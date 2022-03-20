mod timer {
    const SECOND: core::time::Duration = core::time::Duration::from_secs(1);

    /// Timer device handle.
    pub struct Timer(
        core::marker::PhantomData<*mut ()>,
        core::marker::PhantomPinned,
    );

    impl Timer {
        pub fn new() -> Self {
            Self(core::marker::PhantomData, core::marker::PhantomPinned)
        }
    }

    impl IntoIterator for &mut Timer {
        type IntoIter =
            core::iter::RepeatWith<Box<dyn FnMut() -> SealedFuture>>;
        type Item = SealedFuture;

        fn into_iter(self) -> Self::IntoIter {
            core::iter::repeat_with(Box::new(|| {
                SealedFuture(Box::pin(async {
                    async_std::task::sleep(SECOND).await
                }))
            }))
        }
    }

    /// Sealed: not re-exported
    pub struct SealedFuture(
        core::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>>,
    );

    impl core::future::Future for SealedFuture {
        type Output = ();

        fn poll(
            mut self: core::pin::Pin<&mut Self>,
            cx: &mut core::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            self.0.as_mut().poll(cx)
        }
    }
}

use pasts::Past;
use timer::Timer;

async fn async_main() {
    let mut timer = Past::from(&mut Timer::new());

    for _ in 0..3 {
        println!("Waiting 1 second...");
        timer.next().await;
    }

    println!("Waited 3 seconds!");
}

fn main() {
    pasts::block_on(async_main());
}
