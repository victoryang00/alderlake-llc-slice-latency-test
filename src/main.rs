extern crate core_affinity;
extern crate lazy_static;
#[cfg(feature = "cat-process")]
extern crate zerocopy;
use lazy_static::lazy_static;
use std::arch::x86_64::_rdtsc;
use std::io::Read;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;
// shm get sizeof LLC slice and spawn process with CAT technology
#[cfg(feature = "cat-process")]
use rusty_fork::{fork, rusty_fork_id};
#[cfg(feature = "cat-process")]
use shmem_ipc::sharedring::{Receiver, Sender};
#[cfg(feature = "cat-process")]
use std::process;

#[cfg(feature = "cat-process")]
#[derive(Copy, Clone)]
pub struct PqosL3ca {
    class_id: u32,
    /**< class of service */
    cdp: i32,
    /**< data & code masks used if true */
    u: U,
}
#[cfg(feature = "cat-process")]
#[derive(Copy, Clone)]
pub union U {
    ways_mask: u64,
    /**< bit mask for L3 cache ways */
    s: S,
}
#[cfg(feature = "cat-process")]
#[derive(Copy, Clone)]

struct S {
    data_mask: u64,
    code_mask: u64,
}

#[cfg(feature = "cat-process")]
extern "C" {
    fn pqos_l3ca_set(l3cat_id: u32, max_num_ca: u32, num_ca: *const u32, pq_config: PqosL3ca);
}

lazy_static! {
    static ref now: AtomicU64 = AtomicU64::new(0);
}

#[cfg(feature = "cat-process")]
fn set_qpos(way_qos: i32) {
    let mut num_ca = 1;
    let mut pq_config = PqosL3ca {
        class_id: 0,
        cdp: 0,
        u: U {
            ways_mask: way_qos as u64,
        },
    };
    unsafe {
        pqos_l3ca_set(0, 1, &mut num_ca, pq_config);
    }
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
    println!(
        "E->P core latency: {}",
        unsafe { _rdtsc() } - now.load(Ordering::Acquire)
    );
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
    println!(
        "P->E core latency: {}",
        unsafe { _rdtsc() } - now.load(Ordering::Acquire)
    );

    running.store(false, Ordering::Release);
}
#[cfg(not(feature = "cat-process"))]
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

#[cfg(feature = "cat-process")]
fn capturing_output(cmd: &mut process::Command) {
    // Only actually capture stdout since we can't use
    // wait_with_output() since it for some reason consumes the `Child`.

    cmd.stdout(process::Stdio::piped())
        .stderr(process::Stdio::inherit());
}

#[cfg(feature = "cat-process")]
fn main() {
    let mut s = Sender::new(8 * 1024).unwrap();
    let memfd = s.memfd().as_file().try_clone().unwrap();
    let e = s.empty_signal().try_clone().unwrap();
    let f = s.full_signal().try_clone().unwrap();
    let mut r = Receiver::open(8 * 1024, memfd, e, f).unwrap();
    let mut x = 0;
    let mut ss: u64 = 0;
    now.store(unsafe { _rdtsc() }, Ordering::Release);
    let output = fork(
        "main",
        rusty_fork_id!(),
        capturing_output,
        |child, _| {
            let mut sum: usize = 0;
            let mut res = String::new();
            taskset(1);
            set_qpos(5);
            unsafe {
                r.receive_trusted(|p: &[u8]| {
                    for i in 0..1024 {
                        sum += p[i] as usize;
                    }
                    sum
                })
            }
            .unwrap();
            res = format!(
                "P->E core latency: {}",
                unsafe { _rdtsc() } - now.load(Ordering::Acquire)
            )
            .to_owned();
            // unsafe {
            //     s.send_trusted(|p: &mut [u8]| {
            //         for i in 0..1024 {
            //             sum += p[i] as usize;
            //         }
            //         sum
            //     })
            // }
            // .unwrap();
            // child
            //     .inner_mut()
            //     .stdout
            //     .as_mut()
            //     .unwrap()
            //     .read_to_string(&mut res)
            //     .unwrap();
            // assert!(child.wait().unwrap().success());
            res
        },
        || {
            let mut sum: usize = 0;
            taskset(16);
            set_qpos(11);
            unsafe {
                s.send_trusted(|p: &mut [u8]| {
                    for i in 0..1024 {
                        sum += p[i] as usize;
                    }
                    sum
                })
            }
            .unwrap();
            // unsafe {
            //     r.receive_trusted(|p: &[u8]| {
            //         for i in 0..1024 {
            //             sum += p[i] as usize;
            //         }
            //         sum
            //     })
            // }
            // .unwrap();
            // println!(
            //     "E->P core latency: {}",
            //     unsafe { _rdtsc() } - now.load(Ordering::Acquire)
            // )
        },
    )
    .unwrap();

    std::thread::sleep(Duration::from_nanos(1000));
    println!("{}", output);
}
