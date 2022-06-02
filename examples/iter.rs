use core::cell::RefCell;

use pasts::prelude::*;

struct MyNotifier;

impl Notifier for MyNotifier {
    type Event = u32;

    fn poll_next(
        self: Pin<&mut Self>,
        _: &mut TaskCx<'_>,
    ) -> Poll<Self::Event> {
        Ready(1)
    }
}

async fn run() {
    let mut count = 0;
    let notifier: &RefCell<_> = &MyNotifier.into();
    for mut i in core::iter::repeat(notifier).map(|i| i.borrow_mut()).take(3) {
        count += i.next().await;
    }
    assert_eq!(count, 3);
}

fn main() {
    pasts::block_on(run());
}
