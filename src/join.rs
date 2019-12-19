/// Poll multiple futures concurrently, and return a tuple of returned values
/// from each future.
///
/// This macro is only usable inside async functions and blocks.
///
/// Futures that are ready first will be executed first.  This makes
/// `join!(a, b)` faster than the alternative `(a.await, b.await)`.
///
/// ```rust
/// #![forbid(unsafe_code)]
///
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
/// <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(example());
/// ```
#[macro_export]
macro_rules! join {
    ($($future:ident),* $(,)?) => {
        {
            use $crate::{
                tasks, select, _pasts_hide::{stn::mem::MaybeUninit, join}
            };
            let mut count = 0;
            $(
                // Force move.
                let mut $future = $future;
                // Shadow to prevent future use.
                #[allow(unused_mut)]
                let mut $future = $crate::_pasts_hide::new_task(&mut $future);

                count += 1;
            )*
            for _ in 0..count {
                select! {
                    $( ret = $future.0 => $future.1 = MaybeUninit::new(ret) ),*
                }
            }
            ($(join($future.1)),*)
        }
    };
}
