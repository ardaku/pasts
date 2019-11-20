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
/// pasts::block_on::<_, pasts::CondvarInterrupt>(example());
/// ```
#[macro_export]
macro_rules! join {
    ($($future:ident),* $(,)?) => {
        {
            use $crate::{let_pin, Wait, Done, select};
            let_pin! { $($future = $future;)* };
            let mut count = 0;
            $( let mut $future = { count += 1; Wait($future) }; )*
            for _ in 0..count { select! { $( _ref = $future => {} ),* } }
            ($(match $future { Done(r) => r, Wait(_) => unreachable!(), } ),* )
        }
    };
}
