use crate::_pasts_hide::stn::task::{RawWaker, RawWakerVTable, Waker};

/// Implement this trait to turn a unit struct into a waker.  Your
/// implementation should modify one of:
/// - A static mutable atomic (for no_std)
/// - A static mutable condvar (for threads to sleep while waiting for waker,
///   requires std)
pub trait Wake: Send + Sync + Sized {
    /// This function should either modify a condvar or mutable atomic to let
    /// the asynchronous loop wake up.
    fn wake_up(&self);

    /// Get a `Waker` from type that implements `Wake`.
    fn into_waker(waker: *const Self) -> Waker {
        new_waker(waker)
    }
}

#[allow(unsafe_code)]
fn new_waker<W: Wake>(waker: *const W) -> Waker {
    unsafe fn clone<T: Wake>(data: *const ()) -> RawWaker {
        RawWaker::new(data, vtable::<T>())
    }

    unsafe fn wake<T: Wake>(data: *const ()) {
        ref_wake::<T>(data)
    }

    unsafe fn ref_wake<T: Wake>(data: *const ()) {
        T::wake_up(&*(data as *const T));
    }

    unsafe fn drop<T: Wake>(_data: *const ()) {}

    unsafe fn vtable<W: Wake>() -> &'static RawWakerVTable {
        &RawWakerVTable::new(clone::<W>, wake::<W>, ref_wake::<W>, drop::<W>)
    }

    unsafe {
        Waker::from_raw(RawWaker::new(waker as *const (), vtable::<W>()))
    }
}
