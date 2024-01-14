#![no_std]
#![no_main]

extern crate alloc;

use core::time::Duration;

use alloc::sync::Arc;
use pros::{prelude::*, sync::Mutex, task::delay};
use spin::mutex;

#[derive(Default)]
pub struct Robot;
#[async_trait]
impl SyncRobot for Robot {
    fn opcontrol(&mut self) -> pros::Result {
        println!("basic example");

        let mutex = Arc::new(Mutex::new(0u32));

        pros::task::spawn({
            let mutex = mutex.clone();
            move || {
                let mut guard = mutex.lock_blocking().unwrap();
                println!("mutex: {:?}", mutex);
                println!("{}", *guard);
                delay(Duration::from_secs(1));
                *guard += 1;
                drop(guard)
            }
        });

        let guard = mutex.lock_blocking().unwrap();
        println!("mutex: {:?}", mutex);
        println!("{}", *guard);
        drop(guard);

        Ok(())
    }
}
sync_robot!(Robot);
