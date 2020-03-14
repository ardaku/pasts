/// Pin to the stack.
///
/// ```
/// let a: u32 = 10;
/// pasts::pin_mut!(a);
/// let _: std::pin::Pin<&mut u32> = a;
/// ```
#[macro_export]
macro_rules! pin_mut {
    ($x:ident) => {
        // Force move (don't use this identifier from this point on).
        let mut $x = $x;
        // Shadow use to prevent future use that could move it.
        let mut $x = $crate::_pasts_hide::new_pin(&mut $x);
    }
}
