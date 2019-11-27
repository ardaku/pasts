use pasts::stn::{
    future::Future,
    task::{Poll, Context},
    pin::Pin,
};

/// return = future => { /* do something with return */ },
macro_rules! select {
    ($($pattern:pat = $var:ident => $branch:expr),* $(,)?) => {
        {
            use pasts::{let_pin, stn::{future::Future, pin::Pin}};
            let_pin! { $( $var = $var; )* }
            struct Selector<'a, T> {
                closure: &'a mut dyn FnMut(&mut Context<'_>) -> Poll<T>,
            }
            impl<'a, T> Future for Selector<'a, T> {
                type Output = T;
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
                    -> Poll<Self::Output>
                {
                    (self.get_mut().closure)(cx)
                }
            }
            let closure = &mut |cx: &mut Context<'_>| {
                $(
                    match Future::poll($var.as_mut(), cx) {
                        Poll::Ready(r) => {
                            let exec = &mut |$pattern| { $branch };
                            return Poll::Ready(exec(r));
                        }
                        _ => { /* not ready yet */ }
                    }
                )*
                Poll::Pending
            };
            Selector { closure }.await
        }
    };
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
        // Inputs
        let a_fut = async { AlwaysPending().await };
        let b_fut = async { two().await };

        // MACRO_START:

        select!(
            a = a_fut => {
                println!("This will never print!");
                Select::One(a)
            },
            b = b_fut => Select::Two(b),
        )
    }

    assert_eq!(pasts::block_on(example()), Select::Two('c'));
}
