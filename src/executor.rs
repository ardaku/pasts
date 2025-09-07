use alloc::sync::Arc;
use core::fmt;

use crate::{
    LocalBoxFuture,
    os::{Os, Target},
    pool::{DefaultPool, Pool},
};

/// Pasts' executor.
///
/// # Run a Future
///
/// It's relatively simple to block on a future, and run it to completion:
///
/// ```rust
#[doc = include_str!("../examples/executor.rs")]
/// ```
/// 
/// # Spawn a Future
///
/// You may spawn tasks on an `Executor`.  Only once all tasks have completed,
/// can [`block_on()`](Executor::block_on()) return.
/// ```rust,no_run
#[doc = include_str!("../examples/spawn.rs")]
/// ```
/// 
/// # Recursive `block_on()`
///
/// One cool feature about the pasts executor is that you can run it from within
/// the context of another:
/// ```rust
#[doc = include_str!("../examples/recursive.rs")]
/// ```
/// 
/// Or even resume the executor from within it's own context:
/// ```rust
#[doc = include_str!("../examples/resume.rs")]
/// ```
pub struct Executor<P: Pool = DefaultPool>(Arc<P>);

impl Default for Executor {
    fn default() -> Self {
        Self::new(DefaultPool::default())
    }
}

impl<P: Pool> Clone for Executor<P> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<P: Pool + fmt::Debug> fmt::Debug for Executor<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Executor").field(&self.0).finish()
    }
}

impl<P: Pool> Executor<P> {
    /// Create a new executor that can only spawn tasks from the current thread.
    ///
    /// Custom executors can be built by implementing [`Pool`].
    #[inline(always)]
    pub fn new(pool: P) -> Self {
        Self(Arc::new(pool))
    }

    /// Block on a future and return it's result.
    ///
    /// # Platform-Specific Behavior
    ///
    /// When building with feature _`web`_, spawns task and returns
    /// immediately instead of blocking.
    #[inline(always)]
    pub fn block_on(self, f: impl Future<Output = ()> + 'static) {
        Os.block_on(&*self.0, f);
    }
}

impl<P: Pool> Executor<P> {
    /// Spawn a [`LocalBoxFuture`] on this executor.
    ///
    /// Execution of the [`LocalBoxFuture`] will halt after the first poll that
    /// returns [`Ready`](Poll::Ready).
    #[inline(always)]
    pub fn spawn_future(&self, f: LocalBoxFuture<'static>) {
        Os.spawn_boxed(&*self.0, f);
    }

    /// Box and spawn a future on this executor.
    #[inline(always)]
    pub fn spawn_boxed(&self, f: impl Future<Output = ()> + 'static) {
        Os.spawn(&*self.0, f);
    }
}
