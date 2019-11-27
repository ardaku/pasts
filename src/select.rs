/// Poll multiple futures concurrently, and run the future that is ready first.
/// Only usable inside async functions and blocks.
///
/// The API is like a match statement: `match_pattern = future => expression`.
/// The expression will be run with the pattern (returned from the future) in
/// scope when the future is the first to complete.  This usage is the same as
/// the one from the futures crate.
///
/// # Example
/// ```rust
/// use core::{
///     pin::Pin,
///     future::Future,
///     task::{Poll, Context},
/// };
///
/// #[derive(Debug, PartialEq)]
/// enum Select {
///     One(i32),
///     Two(char),
/// }
///
/// pub struct AlwaysPending();
///
/// impl Future for AlwaysPending {
///     type Output = i32;
///
///     fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<i32> {
///         Poll::Pending
///     }
/// }
///
/// async fn two() -> char {
///     'c'
/// }
///
/// async fn example() -> Select {
///     let a_fut = AlwaysPending();
///     let b_fut = two();
///
///     pasts::select!(
///         a = a_fut => {
///             println!("This will never print!");
///             Select::One(a)
///         }
///         b = b_fut => Select::Two(b)
///     )
/// }
///
/// assert_eq!(pasts::block_on(example()), Select::Two('c'));
/// ```
#[macro_export] macro_rules! select {
    ($($pattern:pat = $var:ident => $branch:expr $(,)?)*) => {
        {
            use $crate::{
                let_pin,
                _pasts_hide::stn::{
                    future::Future,
                    pin::Pin,
                    task::{Poll, Context},
                },
            };
            let_pin! { $( $var = $var; )* }
            struct Selector<'a, T> {
                closure: &'a mut dyn FnMut(&mut Context<'_>) -> Poll<T>,
            }
            impl<'a, T> Future for Selector<'a, T> {
                type Output = T;
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
                    (self.get_mut().closure)(cx)
                }
            }
            Selector { closure: &mut |cx: &mut Context<'_>| {
                $(
                    match Future::poll($var.as_mut(), cx) {
                        Poll::Ready(r) =>
                            return Poll::Ready((&mut |$pattern| $branch)(r)),
                        Poll::Pending => {},
                    }
                )*
                Poll::Pending
            } }.await
        }
    };
}
