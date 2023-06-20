use std::time;

use anyhow::Result;
use crossbeam_utils::thread;

pub trait Runnable: Sync {
    fn run(&self) -> Result<()>;
}

pub fn run_in_parallel(runnables: Vec<&dyn Runnable>) {
    thread::scope(|s| {
        for runnable in runnables {
            s.spawn(move |_| {
                runnable.run().unwrap();
            });
        }
    })
    .unwrap();
}

pub fn sleep_millis(millis: u64) {
    let wait_duration = time::Duration::from_millis(millis);
    std::thread::sleep(wait_duration);
}
