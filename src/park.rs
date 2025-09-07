use crate::os::{Os, Target};

/// Trait for implementing the parking / unparking threads.
pub trait Park: Default + Send + Sync + 'static {
    /// The park routine; should put the processor or thread to sleep in order
    /// to save CPU cycles and power, until the hardware tells it to wake up.
    fn park(&self);

    /// Wake the processor or thread.
    fn unpark(&self);
}

#[derive(Debug, Default)]
pub struct DefaultPark(<Os as Target>::ParkCx);

impl Park for DefaultPark {
    // Park the current thread.
    #[inline(always)]
    fn park(&self) {
        Os.park(&self.0);
    }

    // Unpark the parked thread
    #[inline(always)]
    fn unpark(&self) {
        Os.unpark(&self.0);
    }
}
