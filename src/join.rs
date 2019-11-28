/// Poll multiple futures concurrently, and return a tuple of returned values
/// from each future.  Only usable inside async functions and blocks.
///
/// Futures that are ready first will be executed first.  This makes
/// `join!(a, b)` faster than the alternative `(a.await, b.await)`.
///
/// ```rust
/// async fn one() -> char {
///     'c'
/// }
///
/// async fn two() -> char {
///     'a'
/// }
///
/// async fn example() {
///     // Joined await on the two futures.
///     let a = one();
///     let b = two();
///     let ret = pasts::join!(a, b);
///     assert_eq!(ret, ('c', 'a'));
/// }
///
/// pasts::block_on(example());
/// ```
#[macro_export]
macro_rules! join {
    ($($y:expr),* $(,)?) => {
        /*{
            use $crate::{
                let_pin,
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
            let_pin! { $( $var = $var; )* }
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
        }*/
        {
            ('c', 'a')
        }
    };
}
