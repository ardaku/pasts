use pasts::{prelude::*, Loop, Past};

type Exit = ();

struct State {}

impl State {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        println!("Received message from {}, completed task: {}", id, val);
        Ready(())
    }
}

async fn run() {
    let mut state = State {};
    let mut tasks = vec![
        Past::pin(|| async { "Hello" }),
        Past::pin(|| async { "World" }),
    ];

    // First task will complete first.
    Loop::new(&mut state)
        .on(tasks.as_mut_slice(), State::completion)
        .await;
}

fn main() {
    pasts::block_on(run())
}
