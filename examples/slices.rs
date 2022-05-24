use core::iter;

use pasts::{prelude::*, AsyncIter, Loop};

type Exit = ();

struct State {}

impl State {
    fn completion(&mut self, x: Option<(usize, Option<&str>)>) -> Poll<Exit> {
        let (id, val) = x.expect("All futures have completed");
        let val = val.unwrap();
        println!("Received message from {id}, completed task: {val}");
        Ready(())
    }
}

async fn run() {
    let mut state = State {};
    let tasks: &mut [BoxAsyncIterator<'static, &str>] = &mut [
        Box::pin(AsyncIter::from(iter::once(Box::pin(async { "Hello" })))),
        Box::pin(AsyncIter::from(iter::once(Box::pin(async { "World" })))),
    ];

    // First task will complete first.
    Loop::new(&mut state).on(tasks, State::completion).await;
}

fn main() {
    pasts::block_on(run())
}
