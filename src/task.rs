use crate::_pasts_hide::{
    stn::{
        future::Future,
        pin::Pin,
        mem::replace,
    }
};

/// A task that is either not yet ready, or has completed.
pub enum Task<F, O>
    where F: Future<Output = O>
{
    /// Still have to wait for future.
    Wait(F),
    /// Future has completed.
    Done(O),
    /// Future output has been moved.
    Moved,
}

impl<F, O> Task<F, O>
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

    /// Return true if future has completed.
    #[inline(always)]
    pub fn is_done(&self) -> bool {
        !self.is_wait()
    }

    #[inline(always)]
    pub fn as_mut(&mut self) -> Option<&mut F> {
        match *self {
            Task::Wait(ref mut x) => Some(x),
            _ => None,
        }
    }

    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn as_pin_mut(self: Pin<&mut Self>) -> Option<Pin<&mut F>> {
        unsafe {
            Pin::get_unchecked_mut(self).as_mut().map(|x| Pin::new_unchecked(x))
        }
    }

    /// Replace self with `Task::Moved` and return what it was.
    #[allow(unsafe_code)]
    #[inline(always)]
    pub fn take(self: Pin<&mut Self>) -> Self {
        unsafe {
            replace(Pin::get_unchecked_mut(self), Task::Moved)
        }
    }

    /// Get 
    #[inline(always)]
    pub fn unwrap(self) -> O {
        match self {
            Task::Done(output) => output,
            Task::Wait(_) => panic!("unwrap() called on an incomplete task!"),
            Task::Moved => panic!("unwrap() called on a moved task!"),
        }
    }
}
