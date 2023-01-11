extern crate core_affinity;
extern crate lazy_static;

use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
// shm get sizeof LLC slice and spawn process with CAT technology

lazy_static! {
    static ref now: Arc<std::time::Instant> = Arc::new(std::time::Instant::now());
}

fn taskset(cpuid: usize) {
    core_affinity::set_for_current(core_affinity::CoreId { id: cpuid });
}

fn p_core_thread(running: Arc<AtomicBool>, shared_array: *mut u8) {
    taskset(0);
    let shared_array = unsafe { std::slice::from_raw_parts_mut(shared_array, 1024 as usize) };
    for i in 0..1024 {
        shared_array[i] = 1;
    }
    running.store(true, Ordering::Release);
    // wait for E core to finish
    while running.load(Ordering::SeqCst) {
        for i in 0..1024 {
            shared_array[i] = 1;
        }
        break;
    }
    println!("E->P core latency: {:?}", now.elapsed());
    running.store(false, Ordering::Release);
}

fn e_core_thread(running: Arc<AtomicBool>, shared_array: *mut u8) {
    taskset(16);
    let shared_array = unsafe { std::slice::from_raw_parts_mut(shared_array, 1024 as usize) };
    while running.load(Ordering::SeqCst) {
        for i in 0..1024 {
            shared_array[i] = 0;
        }
        break;
    }
    println!("P->E core latency: {:?}", now.elapsed());

    running.store(false, Ordering::Release);
}

fn main() {
    // Retrieve the IDs of all active CPU cores.
    // let core_ids = core_affinity::get_core_ids().unwrap();
    let mut shared_array = [0u8; 8 * 1024];
    let mut handles = vec![];
    for _ in 0..10 {
        let running = Arc::new(AtomicBool::new(false));

        let p_th = {
            let running = running.clone();
            std::thread::spawn(move || p_core_thread(running, shared_array.as_mut_ptr()))
        };
        let e_th = {
            let running = running.clone();

            std::thread::spawn(move || e_core_thread(running, shared_array.as_mut_ptr()))
        };

        handles.push(p_th.join().unwrap());
        handles.push(e_th.join().unwrap());
    }
}
