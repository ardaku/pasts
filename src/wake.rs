use core::task::{RawWaker, RawWakerVTable, Waker};

/// A static waker.  Implement on a unit struct, and modify one of:
/// - A static mutable atomic (for no_std)
/// - A static mutable condvar (for threads to sleep while waiting for waker,
///   requires std)
pub trait Wake: Send + Sync + Sized {
    /// This function should either modify a condvar or mutable atomic to let
    /// the asynchronous loop wake up.
    unsafe fn wake_up();

    /// Get a `Waker` from type that implements `Wake`.
    unsafe fn into_waker(waker: *const Self) -> Waker {
        Waker::from_raw(RawWaker::new(waker as *const (), vtable::<Self>()))
    }
}

fn vtable<W: Wake>() -> &'static RawWakerVTable {
    unsafe fn clone<T: Wake>(_data: *const ()) -> RawWaker {
        RawWaker::new(_data, vtable::<T>())
    }

    unsafe fn wake<T: Wake>(_data: *const ()) {
        ref_wake::<T>(_data)
    }

    unsafe fn ref_wake<T: Wake>(_data: *const ()) {
        T::wake_up();
    }

    unsafe fn drop<T: Wake>(_data: *const ()) { }

    &RawWakerVTable::new(clone::<W>, wake::<W>, ref_wake::<W>, drop::<W>)
}
