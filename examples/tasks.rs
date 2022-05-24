use core::iter;

use pasts::{prelude::*, AsyncIter, Loop};

enum Exit {
    /// Task has completed, remove it
    Remove(usize),
}

struct State {}

impl State {
    fn completion(
        &mut self,
        next: Option<(usize, Option<&str>)>,
    ) -> Poll<Exit> {
        let (id, val) = next.unwrap();
        let val = val.unwrap();

        println!("Received message from completed task: {val}");

        Ready(Exit::Remove(id))
    }
}

async fn run() {
    let mut state = State {};
    let mut tasks: Vec<BoxAsyncIterator<'static, &str>> = vec![
        Box::pin(AsyncIter::from(iter::once(Box::pin(async { "Hello" })))),
        Box::pin(AsyncIter::from(iter::once(Box::pin(async { "World" })))),
    ];

    while !tasks.is_empty() {
        match Loop::new(&mut state)
            .on(&mut tasks[..], State::completion)
            .await
        {
            Exit::Remove(index) => {
                tasks.swap_remove(index);
            }
        }
    }
}

fn main() {
    pasts::block_on(run());
}
