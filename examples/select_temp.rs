use pasts::stn::{
    future::Future,
    task::{Poll, Context},
    pin::Pin,
};

struct SelectorFuture<'a, T> {
    futures: &'a mut [Pin<&'a mut dyn Future<Output = T>>],
}

impl<'a, T> Future for SelectorFuture<'a, T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Self::Output>
    {
        for i in 0..self.futures.len() {
            match Future::poll(self.futures[i].as_mut(), cx) {
                Poll::Ready(r) => {
                    return Poll::Ready(r);
                }
                _ => { /* not ready yet */ }
            }
        }
        Poll::Pending
    }
}

fn main() {
    #[derive(Debug, PartialEq)]
    enum Select {
        One(i32),
        Two(char),
    }

    // An always pending future.
    pub struct AlwaysPending();

    impl Future for AlwaysPending {
        type Output = i32;

        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output>
        {
            Poll::Pending
        }
    }

    async fn two() -> char {
        'c'
    }

    async fn example() -> Select {
        pasts::let_pin! {
            future_a = &mut async { Select::One(AlwaysPending().await) };
            future_b = &mut async { Select::Two(two().await) };
        };
        let mut futures: [Pin<&mut dyn Future<Output = Select>>; 2] = [
             future_a, future_b,
        ];

        let joined_future = SelectorFuture {
            futures: &mut futures[..],
        };

        let ret = joined_future.await;

        ret
    }

    assert_eq!(pasts::block_on(example()), Select::Two('c'));
}
