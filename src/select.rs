/// Poll multiple futures concurrently, and run the future that is ready first.
///
/// This macro is only usable inside async functions and blocks.
///
/// The API is like a match statement:
/// `match_pattern = pinnned_future => expression`.  The expression will be run
/// with the pattern (returned from the future) in scope when the future is the
/// first to complete.  This usage is the similar to the one from the futures
/// crate; Although, neither `default` or `complete` are supported, and rather
/// than using fused futures, this API uses [`Task`](enum.Task.html)s that are
/// turned to [`Done`](enum.Task.html#variant.Done) on completion.
///
/// This is the lowest level async control structure.  All other async control
/// structures can be built on top of [`select!()`](macro.select.html).
///
/// # Example
/// ```rust
/// #![forbid(unsafe_code)]
///
/// use core::{
///     pin::Pin,
///     future::Future,
///     task::{Poll, Context},
/// };
///
/// use pasts::prelude::*;
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
///     pasts::tasks! {
///         a_fut = AlwaysPending();
///         b_fut = two();
///     };
///
///     let ret = pasts::select!(
///         a = a_fut => {
///             println!("This will never print!");
///             Select::One(a)
///         }
///         b = b_fut => Select::Two(b)
///     );
///
///     assert!(a_fut.is_wait());
///     assert!(b_fut.is_done());
///
///     ret
/// }
///
/// assert_eq!(
///     pasts::ThreadInterrupt::block_on(example()),
///     Select::Two('c')
/// );
/// ```
#[macro_export]
macro_rules! select {
    ($($pattern:ident = $future:expr => $branch:expr $(,)?)*) => {
        {
            use $crate::{
                Task,
                _pasts_hide::stn::{
                    future::Future,
                    pin::Pin,
                    task::{Poll, Context},
                },
            };
            struct __Pasts_Selector<'a, T> {
                closure: &'a mut dyn FnMut(&mut Context<'_>) -> Poll<T>,
            }
            impl<'a, T> Future for __Pasts_Selector<'a, T> {
                type Output = T;
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
                    (self.get_mut().closure)(cx)
                }
            }
            __Pasts_Selector { closure: &mut |__pasts_cx: &mut Context<'_>| {
                $(
                    match $future.poll(__pasts_cx) {
                        Poll::Ready($pattern) => {
                            let ret = { $branch };
                            return Poll::Ready(ret);
                        }
                        Poll::Pending => {}
                    }
                )*
                Poll::Pending
            } }.await
        }
    };
}
