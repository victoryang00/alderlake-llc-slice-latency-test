extern crate core_affinity;

use crossbeam_epoch::{Atomic, Guard, Shared};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    alloc::{alloc, Layout},
    sync::Arc,
    thread,
};
// shm get sizeof LLC slice and spawn process with CAT technology

fn main() {
    // Retrieve the IDs of all active CPU cores.
    // let core_ids = core_affinity::get_core_ids().unwrap();
    let shared_array = Arc::new(RwLock::new([0u8; 8 * 1024]));

    let ptr = AtomicBool::new(false);
    let core_ids = core_affinity::get_core_ids().unwrap();
    let mut now = std::time::Instant::now();

    let core_ids = vec![core_ids[0], core_ids[16]];
    dbg!(core_ids.clone());
    // Create a thread for each active CPU core.
    let handles = core_ids
        .into_iter()
        .map(|id| {
            thread::spawn(move || {
                // Pin this thread to a single CPU core.
                let res = core_affinity::set_for_current(id);
                if res {
                    match id.id {
                        0 => {
                            // P core affinity
                            for i in 0..8 * 1024 {
                                shared_array[i] = 1;
                            }
                            now = std::time::Instant::now();
                            ptr.store(true, Ordering::Release);
                            // wait for E core to finish
                            while ptr.load(Ordering::SeqCst) {
                                for i in 0..8 * 1024 {
                                    shared_array[i] = 1;
                                }
                            }
                            println!("E->P core latency: {:?}", now.elapsed());
                            ptr.store(false, Ordering::Release);
                        }
                        16 => {
                            // E core affinity
                            while ptr.load(Ordering::SeqCst) {
                                for i in 0..8 * 1024 {
                                    shared_array[i] = 0;
                                }
                            }
                            println!("P->E core latency: {:?}", now.elapsed());
                            ptr.store(false, Ordering::Release);
                        }
                        _ => {}
                    }
                }
            })
        })
        .collect::<Vec<_>>();

    for handle in handles.into_iter() {
        handle.join().unwrap();
    }
}
