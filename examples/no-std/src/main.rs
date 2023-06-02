//! Use pasts on no-std, specifically targeting x86_64-unknown-linux-gnu (may
//! work on others).  Requires nightly for `eh_personality` lang item (tested
//! with `rustc 1.72.0-nightly (871b59520 2023-05-31)`).
//!                                                                             
//! ```shell
//!
//! cargo +nightly run                                                           
//! ```

#![no_std]
#![no_main]
#![feature(lang_items)]

extern crate alloc;

#[global_allocator]
static _GLOBAL_ALLOCATOR: rlsf::SmallGlobalTlsf = rlsf::SmallGlobalTlsf::new();

#[lang = "eh_personality"]
extern "C" fn _eh_personality() {}

#[no_mangle]
extern "C" fn _Unwind_Resume() {}

fn print(string: &str) {
    use core::ffi::{c_int, c_void};

    #[link(name = "c")]
    extern "C" {
        fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
    }

    unsafe { write(0, string.as_ptr().cast(), string.len()) };
}

#[panic_handler]
fn yeet(panic: &::core::panic::PanicInfo<'_>) -> ! {
    print(&panic.to_string());
    print("\n");

    loop {}
}

//// End no-std specific boilerplate ////

use alloc::string::ToString;

use async_main::{async_main, LocalSpawner};
use pasts::{prelude::*, Loop};

struct State {
    // Spawned tasks
    tasks: [LocalBoxNotify<'static, &'static str>; 2],
}

impl State {
    fn task_done(&mut self, (_id, text): (usize, &str)) -> Poll {
        print(text);
        print("\n");
        Pending
    }
}

async fn task_one() -> &'static str {
    print("Task 1...\n");
    "Hello"
}

async fn task_two() -> &'static str {
    print("Task 2...\n");
    "World"
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

mod main {
    #[no_mangle]
    extern "C" fn main() -> ! {
        super::main();

        loop {}
    }
}
