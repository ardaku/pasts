use pasts::{Task, Poll};

pasts::glue!();

async fn run() {
    let hello: Task<&str> = Box::pin(async { "Hello" });
    let world: Task<&str> = Box::pin(async { "World" });
    let mut tasks = vec![hello, world];

    while !tasks.is_empty() {
        let (idx, val) = tasks.poll().await;
        tasks.remove(idx);
        println!("Received message from completed task: {}", val);
    }
}
