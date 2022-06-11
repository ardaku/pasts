use pasts::{Join, prelude::*};
use core::ffi::CStr;

struct HelloWorld;

impl Notifier for HelloWorld {
    type Event = &'static str;

    fn poll_next(self: Pin<&mut Self>, _e: &mut Exec<'_>) -> Poll<Self::Event> {
        Ready("Hello, world!\0")
    }
}

struct State {
    hello_world: HelloWorld,
}

impl State {
    fn hello_world(&mut self, text: &str) -> Poll<()> {
        crate::log::println(CStr::from_bytes_with_nul(text.as_bytes()).unwrap());

        Ready(())
    }
}

static mut STATE: State = State {
    hello_world: HelloWorld,
};

fn main(_executor: &Executor) -> impl Future<Output = ()> + Unpin {
    // unsafe: Safe because only ever borrowed once
    unsafe {
        Join::new(&mut STATE).on(|s| &mut s.hello_world, State::hello_world)
    }
}
