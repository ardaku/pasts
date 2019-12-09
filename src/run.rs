/*/// Poll multiple futures concurrently in a loop.  At completion of each future,
/// the future is regenerated.  Only usable inside async functions and blocks.
///
/// ```rust
/// /*use pasts::prelude::*;
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
/// // Run futures.
/// pasts::run!(
///     // Use Condvar Interrupt.
///     pasts::CondvarInterrupt,
///     // `async fn`s to recreate futures.
///     example(),
/// );*/
/// ```
#[macro_export]
macro_rules! run {
    ($interrupt:ty, $($future:expr),* $(,)?) => {
        {
            use $crate::{let_pin, Task::{Wait, Done}, select};
            let_pin! { $($future = $future;)* };
            let mut count = 0;
            $( let mut $future = { count += 1; Wait($future) }; )*
            for _ in 0..count { select! { $( _ref = $future => {} ),* } }
            ($(match $future { Done(r) => r, Wait(_) => unreachable!(), } ),* )
        }
    };
}*/
