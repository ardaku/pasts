#![no_std]

extern crate alloc;

use async_main::{async_main, LocalSpawner};
use pasts::{prelude::*, Loop};

struct State {
    // Spawned tasks
    tasks: [LocalBoxNotify<'static, &'static str>; 2],
}

impl State {
    fn task_done(&mut self, (_id, text): (usize, &str)) -> Poll {
        log::log(text);
        Pending
    }
}

async fn task_one() -> &'static str {
    log::log("Task 1...\0");
    "Hello\0"
}

async fn task_two() -> &'static str {
    log::log("Task 2...\0");
    "World\0"
}

#[async_main]
async fn main(_spawner: LocalSpawner) {
    // create two tasks to spawn
    let task_one = Box::pin(task_one().fuse());
    let task_two = Box::pin(task_two().fuse());

    // == Allocations end ==

    // create array of tasks to spawn
    let state = &mut State {
        tasks: [task_one, task_two],
    };

    Loop::new(state)
        .on(|s| s.tasks.as_mut_slice(), State::task_done)
        .await;
}

mod log {
    use core::ffi::CStr;

    /// Log a message.  Requires trailing null byte.
    pub fn log(text: &str) {
        let text = CStr::from_bytes_with_nul(text.as_bytes()).unwrap();

        #[link(name = "c")]
        extern "C" {
            fn puts(s: *const ()) -> i32;
        }

        unsafe { puts(text.as_ptr().cast()) };
    }
}
