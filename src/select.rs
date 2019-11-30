/// Poll multiple futures concurrently, and run the future that is ready first.
/// Only usable inside async functions and blocks.
///
/// The API is like a match statement:
/// `match_pattern = pinnned_future => expression`.  The expression will be run
/// with the pattern (returned from the future) in scope when the future is the
/// first to complete.  This usage is the similar to the one from the futures
/// crate; Although, neither `default` or `complete` are supported, and rather
/// than using fused futures, this API uses optional futures that are turned to
/// `None` on completion.
///
/// This is the lowest level async control structure.  All other async control
/// structures can be built on top of `select!()`.
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
///     pasts::let_pin! {
///         a_fut = pasts::Task::Wait(AlwaysPending());
///         b_fut = pasts::Task::Wait(two());
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
/// assert_eq!(pasts::block_on(example()), Select::Two('c'));
/// ```
#[macro_export] macro_rules! select {
    ($($pattern:ident = $future:ident => $branch:expr $(,)?)*) => {
        {
            use $crate::{
                let_pin, Task,
                _pasts_hide::stn::{
                    future::Future,
                    pin::Pin,
                    task::{Poll, Context},
                },
            };
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
                    if let Some(future) = $future.as_mut().as_pin_mut() {
                        match Future::poll(future, cx) {
                            Poll::Ready($pattern) => {
                                let ret = { $branch };
                                $future.set(Task::Done($pattern));
                                return Poll::Ready(ret);
                            }
                            Poll::Pending => {}
                        }
                    }
                )*
                Poll::Pending
            } }.await
        }
    };
}
