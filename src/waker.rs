use alloc::sync::Arc;
use core::marker::PhantomData;
use core::mem;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::task::{RawWaker, RawWakerVTable, Waker};

pub trait Woke: Send + Sync + Sized {
    fn wake_by_ref(&self);

    fn into_waker(waker: *const Self) -> Waker {
        unsafe {
            Waker::from_raw(RawWaker::new(waker as *const (), waker_vtable::<Self>()))
        }
    }
}

pub fn waker_vtable<W: Woke>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(
        clone_raw::<W>,
        wake_raw::<W>,
        wake_by_ref_raw::<W>,
        drop_raw::<W>,
    )
}

pub fn waker<W: Woke>(wake: Arc<W>) -> Waker {
    let ptr = Arc::into_raw(wake) as *const ();

    unsafe { Waker::from_raw(RawWaker::new(ptr, waker_vtable::<W>())) }
}

unsafe fn increase_refcount<T: Woke>(data: *const ()) {
    let arc = mem::ManuallyDrop::new(Arc::<T>::from_raw(data as *const T));
    let _arc_clone: mem::ManuallyDrop<_> = arc.clone();
}

unsafe fn clone_raw<T: Woke>(data: *const ()) -> RawWaker {
    increase_refcount::<T>(data);
    RawWaker::new(data, waker_vtable::<T>())
}

unsafe fn wake_raw<T: Woke>(data: *const ()) {
    Woke::wake_by_ref(&*(data as *const T));
}

unsafe fn wake_by_ref_raw<T: Woke>(data: *const ()) {
    // Retain Arc, but don't touch refcount by wrapping in ManuallyDrop
    Woke::wake_by_ref(&*(data as *const T));
}

unsafe fn drop_raw<T: Woke>(data: *const ()) {
    drop(Arc::<T>::from_raw(data as *const T))
}
