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
