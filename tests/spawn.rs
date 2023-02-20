extern crate whisk;

use pasts::Executor;
use whisk::Channel;

#[test]
fn spawn_inside_block_on() {
    let executor = Executor::default();
    let channel = Channel::new();
    let sender = channel.clone();

    executor.clone().block_on(async move {
        executor.spawn_boxed(async move {
            sender.send(0xDEADBEEFu32).await;
        });
    });

    Executor::default().block_on(async move {
        assert_eq!(0xDEADBEEFu32, channel.recv().await);
    });
}
