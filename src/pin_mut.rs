// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/// Pin a future to the stack.
///
/// ```rust
/// #![forbid(unsafe_code)]
/// 
/// use core::pin::Pin;
/// use core::future::Future;
///
/// let a = async { "Hello, world!" };
/// pasts::pin_fut!(a);
/// // Or alternatively,
/// pasts::pin_fut!(a = async { "Hello, world!" });
///
/// let a: Pin<&mut dyn Future<Output = &str>> = a;
/// ```
#[macro_export]
macro_rules! pin_fut {
    ($x:ident) => {
        // Force move (don't use this identifier from this point on).
        let mut $x = $x;
        // Shadow use to prevent future use that could move it.
        let mut $x: core::pin::Pin<&mut dyn core::future::Future<Output = _>>
            = $crate::_pasts_hide::new_pin(&mut $x);
    };

    ($x:ident = $y:expr) => {
        // Force move (don't use this identifier from this point on).
        let mut $x = $y;
        // Shadow use to prevent future use that could move it.
        let mut $x: core::pin::Pin<&mut dyn core::future::Future<Output = _>>
            = $crate::_pasts_hide::new_pin(&mut $x);
    };
}
