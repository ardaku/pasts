// This example shows task spawning without allocation (requires unsafe).
//
// If you can use an allocator, don't worry about this example.

use pasts::{Join, prelude::*};
use core::{mem::MaybeUninit};

pub mod log;

type StaticTask = Pin<&'static mut dyn Notifier<Event = &'static str>>;

struct State {
    // Spawned tasks
    tasks: [StaticTask; 2],
}

impl State {
    fn task_done(&mut self, (_id, text): (usize, &str)) -> Poll<()> {
        log::log(text);
        Pending
    }
}

static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

async fn task_one() -> &'static str {
    log::log("Task 1...\0");
    "Hello\0"
}

async fn task_two() -> &'static str {
    log::log("Task 2...\0");
    "World\0"
}

unsafe fn pin_static<T: ?Sized>(value: *mut T) -> Pin<&'static mut T> {
    let value: &'static mut T = &mut *value;
    Pin::static_mut(value)
}

async fn main() {
    // create two tasks to spawn
    let mut task_one = task_one().fuse();
    let mut task_two = task_two().fuse();
    // unsafe: Sound because local variables will be available for 'static
    let task_one: StaticTask = unsafe { pin_static(&mut task_one) };
    let task_two: StaticTask = unsafe { pin_static(&mut task_two) };
    // create array of tasks to spawn
    let init = State {
        tasks: [task_one, task_two],
    };
    
    // unsafe: Sound because only ever called/borrowed once
    let state = unsafe { &mut STATE };
    *state = MaybeUninit::new(init);
    // unsafe: Sound because just initialized
    let state = unsafe { state.assume_init_mut() };

    Join::new(state).on(|s| s.tasks.as_mut_slice(), State::task_done).await;
}
