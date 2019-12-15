use crate::_pasts_hide::stn::{future::Future, pin::Pin};

/// A task that is either not yet ready, or has completed.
pub enum Task<'a, F, O>
where
    F: Future<Output = O>,
{
    /// Still have to wait for future.
    Wait(Pin<&'a mut F>),
    /// Future has completed.
    Done(O),
}

impl<'a, F, O> Task<'a, F, O>
where
    F: Future<Output = O>,
{
    /// Return true if still have to wait for future.
    #[inline(always)]
    pub fn is_wait(&self) -> bool {
        match self {
            Self::Wait(_) => true,
            _ => false,
        }
    }

    /// Return true if future has returned.
    #[inline(always)]
    pub fn is_done(&self) -> bool {
        !self.is_wait()
    }

    /// Get return value of completed future.  Panics if future is not complete
    /// yet.
    #[inline(always)]
    pub fn unwrap(self) -> O {
        match self {
            Task::Done(output) => output,
            Task::Wait(_) => panic!("unwrap() called on an incomplete task!"),
        }
    }
}

/// Turn `Future`s into `Task::Wait`s.
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
        let mut $x = $crate::Task::Wait($crate::_pasts_hide::new_pin(&mut $x));
    )* };
}
