extern crate core_affinity;

use std::{
    alloc::{alloc, dealloc, Layout},
    thread, sync::{Arc, Mutex},
};

use core_affinity::CoreId;

// shm get sizeof LLC slice and spawn process with CAT technology

fn main() {
    // Retrieve the IDs of all active CPU cores.
    // let core_ids = core_affinity::get_core_ids().unwrap();
    let signal = Arc::new(Mutex::new(0));
    let core_ids = core_affinity::get_core_ids().unwrap();

    let core_ids = vec![core_ids[0],core_ids[16]];
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
                            signal.lock();
                            unsafe {
                                let layout = Layout::new::<[u16; 100]>();
                                let ptr = alloc(layout);

                                *(ptr as *mut u16) = 42;
                            }


                        }
                        16 => { // E core affinity
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
