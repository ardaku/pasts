use crate::_pasts_hide::stn::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// A task that is either working or has completed.
pub struct Task<'a, F, O>
where
    F: Future<Output = O>,
{
    /// Future waiting on.
    future: Pin<&'a mut F>,
    /// True if future is still valid.
    valid: bool,
}

impl<'a, F, O> Task<'a, F, O>
where
    F: Future<Output = O>,
{
    // Create a new task (in Wait state) from a pinned future.
    pub(crate) fn new(future: Pin<&'a mut F>) -> Self {
        Task {
            future,
            valid: true,
        }
    }

    /// Set a new future for this task.
    #[inline(always)]
    pub fn set(&mut self, future: F) {
        self.future.set(future);
        self.valid = true;
    }

    /// Poll the future associated with this task if it hasn't completed.
    #[inline(always)]
    pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<O> {
        if self.valid {
            match Future::poll(self.future.as_mut(), cx) {
                Poll::Ready(f) => {
                    self.valid = false;
                    Poll::Ready(f)
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }

    /// Return true if still have to wait for future.
    #[inline(always)]
    pub fn is_wait(&self) -> bool {
        self.valid
    }

    /// Return true if future has completed.
    #[inline(always)]
    pub fn is_done(&self) -> bool {
        !self.is_wait()
    }
}

/// Turn `Future`s into `Task`s.
///
/// ```rust
/// #![forbid(unsafe_code)]
///
/// pasts::tasks! {
///     task = async { "Hello, world" };
/// };
///
/// assert!(task.is_wait());
/// ```
#[macro_export]
macro_rules! tasks {
    ($($x:ident = $y:expr);* $(;)?) => { $(
        // Force move.
        let mut $x = $y;
        // Shadow to prevent future use.
        #[allow(unused_mut)]
        let mut $x = $crate::_pasts_hide::new_task(&mut $x).0;
    )* };
}
