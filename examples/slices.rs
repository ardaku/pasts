use pasts::{Task, Poll};

pasts::glue!();

async fn run() {
    let hello: Task<&str> = Box::pin(async { "Hello" });
    let world: Task<&str> = Box::pin(async { "World" });
    let mut array = [hello, world];
    // Hello is ready, so returns with index and result.
    assert_eq!((0, "Hello"), array.poll().await);
}
