/// Pin a variable to a location in the stack.
///
/// ```rust
/// #![forbid(unsafe_code)]
///
/// pasts::let_pin! {
///     var = "Hello, world";
/// };
/// let _: core::pin::Pin<&mut &str> = var;
/// ```
#[macro_export]
macro_rules! let_pin {
    ($($x:ident = $y:expr);* $(;)?) => { $(
        // Force move.
        let mut $x = $y;
        // Shadow to prevent future use.
        #[allow(unused_mut)]
        let mut $x = $crate::_pasts_hide::new_pin(&mut $x);
    )* };
}
