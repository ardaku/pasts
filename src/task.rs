// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/// Create a future trait objects that implement `Unpin`.
///
/// ```rust
/// use pasts::prelude::*;
///
/// async fn bar() { }
///
/// task!(let task_name = async { "Hello, world" });
/// task! {
///     let foo = async {};
///     let bar = bar();
/// }
/// ```
#[macro_export]
macro_rules! task {
    // unsafe: move value, then shadow so one can't directly access anymore.
    ($(let $x:ident = $y:expr);* $(;)?) => { $(
        let mut $x = $y;
        #[allow(unused_mut)]
        let mut $x = unsafe {
            core::pin::Pin::<&mut dyn core::future::Future<Output = _>>
                ::new_unchecked(&mut $x)
        };
    )* };
    ($x:ident) => {
        core::pin::Pin::<&mut dyn core::future::Future<Output = _>>
            ::new(&mut $x)
    };
}
