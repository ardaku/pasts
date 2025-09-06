use alloc::boxed::Box;
use core::pin::Pin;

/// An owned dynamically typed [`Future`] for use in cases where you can’t
/// statically type your result or need to add some indirection.
pub type BoxFuture<'a, T = ()> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// [`BoxFuture`] without the [`Send`] requirement.
pub type LocalBoxFuture<'a, T = ()> = Pin<Box<dyn Future<Output = T> + 'a>>;
