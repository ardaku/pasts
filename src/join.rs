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
/// use pasts::prelude::*;
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
/// pasts::ThreadInterrupt::block_on(example());
/// ```
#[macro_export]
macro_rules! join {
    ($($future:ident),* $(,)?) => {
        {
            use $crate::{let_pin, Task::{Wait, Done}, select};
            let_pin! { $($future = $future;)* };
            let mut count = 0;
            $( let mut $future = { count += 1; Wait($future) }; )*
            for _ in 0..count { select! { $( _ref = $future => {} ),* } }
            ($(match $future { Done(r) => r, Wait(_) => unreachable!(), } ),* )
        }
    };
}
