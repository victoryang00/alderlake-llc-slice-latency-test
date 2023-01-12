extern crate core_affinity;
extern crate lazy_static;
#[cfg(feature = "cat-process")]
extern crate zerocopy;

use lazy_static::lazy_static;
use std::arch::x86_64::_rdtsc;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::thread::Thread;
use std::time::Duration;

// shm get sizeof LLC slice and spawn process with CAT technology
#[cfg(feature = "cat-process")]
use rusty_fork::{fork, rusty_fork_id, ChildWrapper};
#[cfg(feature = "cat-process")]
use shmem_ipc::sharedring::{Receiver, Sender};
#[cfg(feature = "cat-process")]
use std::{fs, io::Read, panic, process};
#[cfg(feature = "cat-process")]
use zerocopy;
#[cfg(feature = "cat-process")]
fn setup_one<T: Copy + zerocopy::FromBytes + zerocopy::ToBytes>(
    chunks: usize,
) -> (Sender<T>, Receiver<T>) {
    let s: Sender<T> = Sender::new(chunks).unwrap();
    let memfd = s.memfd().as_file().try_clone().unwrap();
    let e = s.empty_signal().try_clone().unwrap();
    let f = s.full_signal().try_clone().unwrap();
    let r: Receiver<T> = Receiver::open(chunks, memfd, e, f).unwrap();
    (s, r)
}
#[cfg(feature = "cat-process")]
#[derive(Copy, Clone)]
pub struct pqos_l3ca {
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
    fn pqos_l3ca_set(l3cat_id: u32, max_num_ca: u32, num_ca: *const u32, pq_config: pqos_l3ca);
}

lazy_static! {
    static ref now: AtomicU64 = AtomicU64::new(0);
}

#[cfg(feature = "cat-process")]
fn set_qpos(way_qos: i32) {}

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

#[test]
#[cfg(feature = "cat-process")]
fn get_latency_from_shmem_transfered_between_process() {
    let (r, s) = setup_one::<u8>(8 * 1024);
    let mut x = 0;
    let mut ss: u64 = 0;
    let output = fork(
        "test::get_latency_from_shmem_transfered_between_process",
        rusty_fork_id!(),
        capturing_output,
        || {
            taskset(1);
            set_qpos(5);
            println!(
                "P->E core start: {}",
                unsafe { _rdtsc() } - now.load(Ordering::Acquire)
            );
            r.receive_trusted(|p| {
                ss = ss.wrapping_add(sum(p));
                z
            })
            .unwrap();
        },
        || {
            taskset(16);
            set_qpos(11);
            s.send_trusted(|p| {
                z = std::cmp::min(p.len(), init.len() - x);
                let part = &init[x..(x + z)];
                p[0..z].copy_from_slice(part);
                z
            })
            .unwrap();
            println!(
                "E->P core end: {}",
                unsafe { _rdtsc() } - now.load(Ordering::Acquire)
            );
        },
    )
    .unwrap();

    std::thread::sleep(Duration::from_nanos(1000));
    assert!(
        !output.contains("E->"),
        "Had unexpected output:\n{}",
        output
    );
}
