/// Trait for implementing the parking / unparking threads.
pub trait Park: Default + Send + Sync + 'static {
    /// The park routine; should put the processor or thread to sleep in order
    /// to save CPU cycles and power, until the hardware tells it to wake up.
    fn park(&self);

    /// Wake the processor or thread.
    fn unpark(&self);
}

#[cfg(not(feature = "std"))]
#[derive(Copy, Clone, Debug, Default)]
#[non_exhaustive]
pub struct DefaultPark();

#[cfg(feature = "std")]
#[derive(Debug)]
#[non_exhaustive]
pub struct DefaultPark(std::sync::atomic::AtomicBool, std::thread::Thread);

#[cfg(feature = "std")]
impl Default for DefaultPark {
    fn default() -> Self {
        Self(
            std::sync::atomic::AtomicBool::new(true),
            std::thread::current(),
        )
    }
}

impl Park for DefaultPark {
    // Park the current thread.
    #[inline(always)]
    fn park(&self) {
        // Only park with std; There is no portable parking for no-std.
        #[cfg(feature = "std")]
        while self.0.swap(true, std::sync::atomic::Ordering::Relaxed) {
            std::thread::park();
        }

        // Hint at spin loop to possibly short sleep on no-std to save CPU time.
        #[cfg(not(feature = "std"))]
        core::hint::spin_loop();
    }

    // Unpark the parked thread
    #[inline(always)]
    fn unpark(&self) {
        // Only unpark on std; Since no-std doesn't park, it's already unparked.
        #[cfg(feature = "std")]
        if self.0.swap(false, std::sync::atomic::Ordering::Relaxed) {
            self.1.unpark();
        }
    }
}
