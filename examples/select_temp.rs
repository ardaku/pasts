use pasts::stn::{
    future::Future,
    task::{Poll, Context},
    pin::Pin,
};

/// return: type = future => { /* do something with return */ },
macro_rules! select {
    ($($pattern:pat = $var:ident: $typ:ty => $branch:expr),* $(,)?) => {
        {
            use pasts::{
                let_pin,
                stn::{
                    future::Future,
                    pin::Pin,
                },
            };

            let_pin! {
                $(
                    $var = $var;
                )*
            }

            #[allow(non_camel_case_types)]
            enum Which {
                $($var($typ)),*
            }

            struct Selector<'a> {
                $($var: Pin<&'a mut dyn Future<Output = $typ>>),*
            }

            impl<'a> Future for Selector<'a> {
                type Output = Which;

                fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
                    -> Poll<Self::Output>
                {
                    $(
                        match Future::poll(self.$var.as_mut(), cx) {
                            Poll::Ready(r) => {
                                return Poll::Ready(Which::$var(r));
                            }
                            _ => { /* not ready yet */ }
                        }
                    )*

                    // None are ready yet.
                    Poll::Pending
                }
            }

            let selector = Selector {
                $(
                    $var: $var
                ),*
            };

            match selector.await {
                $(
                    Which :: $var ( $pattern ) => { $branch }
                ),*
            }
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
            a = a_fut: i32 => {
                println!("This will never print!");
                Select::One(a)
            },
            b = b_fut: char => Select::Two(b),
        )
    }

    assert_eq!(pasts::block_on(example()), Select::Two('c'));
}
