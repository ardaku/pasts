use crate::_pasts_hide::{
    stn::{
        future::Future,
        pin::Pin,
//        mem::replace,
    }
};

/// A task that is either not yet ready, or has completed.
pub enum Task<'a, F, O>
    where F: Future<Output = O>
{
    /// Still have to wait for future.
    Wait(Pin<&'a mut F>),
    /// Future has completed.
    Done(O),
}

impl<'a, F, O> Task<'a, F, O>
    where F: Future<Output = O>
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
