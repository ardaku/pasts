// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

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
    () => {{
        ()
    }};
    ($a:expr) => {{
        $a.await
    }};
    ($a:expr, $b:expr) => {{
        $crate::__pasts_join_internal!((a, ra, $a), (b, rb, $b))
    }};
    ($a:expr, $b:expr, $c:expr) => {{
        $crate::__pasts_join_internal!((a, ra, $a), (b, rb, $b), (c, rc, $c))
    }};
    ($a:expr, $b:expr, $c:expr, $d:expr) => {{
        $crate::__pasts_join_internal!(
            (a, ra, $a),
            (b, rb, $b),
            (c, rc, $c),
            (d, rd, $d)
        )
    }};
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr) => {{
        $crate::__pasts_join_internal!(
            (a, ra, $a),
            (b, rb, $b),
            (c, rc, $c),
            (d, rd, $d),
            (e, re, $e)
        )
    }};
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr) => {{
        $crate::__pasts_join_internal!(
            (a, ra, $a),
            (b, rb, $b),
            (c, rc, $c),
            (d, rd, $d),
            (e, re, $e),
            (f, rf, $f)
        )
    }};
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr,
        $g:expr) => {{
        $crate::__pasts_join_internal!(
            (a, ra, $a),
            (b, rb, $b),
            (c, rc, $c),
            (d, rd, $d),
            (e, re, $e),
            (f, rf, $f),
            (g, rg, $g)
        )
    }};
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr,
        $g:expr, $h:expr) => {{
        $crate::__pasts_join_internal!(
            (a, ra, $a),
            (b, rb, $b),
            (c, rc, $c),
            (d, rd, $d),
            (e, re, $e),
            (f, rf, $f),
            (g, rg, $g),
            (h, rh, $h),
        )
    }};
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr,
        $g:expr, $h:expr, $i:expr) => {{
        $crate::__pasts_join_internal!(
            (a, ra, $a),
            (b, rb, $b),
            (c, rc, $c),
            (d, rd, $d),
            (e, re, $e),
            (f, rf, $f),
            (g, rg, $g),
            (h, rh, $h),
            (i, ri, $i)
        )
    }};
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr,
        $g:expr, $h:expr, $i:expr, $j:expr) => {{
        $crate::__pasts_join_internal!(
            (a, ra, $a),
            (b, rb, $b),
            (c, rc, $c),
            (d, rd, $d),
            (e, re, $e),
            (f, rf, $f),
            (g, rg, $g),
            (h, rh, $h),
            (i, ri, $i),
            (j, rj, $j)
        )
    }};
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr,
        $g:expr, $h:expr, $i:expr, $j:expr, $k:expr) => {{
        $crate::__pasts_join_internal!(
            (a, ra, $a),
            (b, rb, $b),
            (c, rc, $c),
            (d, rd, $d),
            (e, re, $e),
            (f, rf, $f),
            (g, rg, $g),
            (h, rh, $h),
            (i, ri, $i),
            (j, rj, $j),
            (k, rk, $k)
        )
    }};
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr,
        $g:expr, $h:expr, $i:expr, $j:expr, $k:expr, $l:expr) => {{
        $crate::__pasts_join_internal!(
            (a, ra, $a),
            (b, rb, $b),
            (c, rc, $c),
            (d, rd, $d),
            (e, re, $e),
            (f, rf, $f),
            (g, rg, $g),
            (h, rh, $h),
            (i, ri, $i),
            (j, rj, $j),
            (k, rk, $k),
            (l, rl, $l),
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pasts_join_internal {
    ($(($ai:ident, $ra:ident, $a_expr:expr)),*) => {{
        $( let mut $ra: Option<_> = None; )*
        {
            $( let $ai: &mut dyn core::future::Future<Output = _>
                = &mut async { $ra = Some($a_expr.await) }; )*
            let tasks = &mut [$((Some($ai))),*][..];
            for _ in 0..tasks.len() {
                $crate::Select::select(tasks).await;
            }
        }
        ($($ra.take().unwrap()),*)
    }}
}
