// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

/// Creates platform-specific entry point, that executes an asynchronous
/// function called `run!()`.
#[macro_export]
macro_rules! glue {
    ($($f:expr),* $(,)?) => {
        #[cfg(not(target_arch = "wasm32"))]
        fn main() {
            $crate::block_on(run());
        }

        #[cfg(target_arch = "wasm32")]
        fn main() {
            $crate::block_on(run());
        }
    };
}
