use alloc::sync::Arc;
use core::marker::PhantomData;
use core::mem;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use core::task::{RawWaker, RawWakerVTable, Waker};

pub trait Woke: Send + Sync {
    fn wake_by_ref(arc_self: &Arc<Self>);
}

pub fn waker_vtable<W: Woke>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(
        clone_arc_raw::<W>,
        wake_arc_raw::<W>,
        wake_by_ref_arc_raw::<W>,
        drop_arc_raw::<W>,
    )
}

pub fn waker<W>(wake: Arc<W>) -> Waker
where
    W: Woke,
{
    let ptr = Arc::into_raw(wake) as *const ();

    unsafe { Waker::from_raw(RawWaker::new(ptr, waker_vtable::<W>())) }
}

unsafe fn increase_refcount<T: Woke>(data: *const ()) {
    let arc = mem::ManuallyDrop::new(Arc::<T>::from_raw(data as *const T));
    let _arc_clone: mem::ManuallyDrop<_> = arc.clone();
}

unsafe fn clone_arc_raw<T: Woke>(data: *const ()) -> RawWaker {
    increase_refcount::<T>(data);
    RawWaker::new(data, waker_vtable::<T>())
}

unsafe fn wake_arc_raw<T: Woke>(data: *const ()) {
    let arc: Arc<T> = Arc::from_raw(data as *const T);
    Woke::wake_by_ref(&arc);
}

unsafe fn wake_by_ref_arc_raw<T: Woke>(data: *const ()) {
    // Retain Arc, but don't touch refcount by wrapping in ManuallyDrop
    let arc = mem::ManuallyDrop::new(Arc::<T>::from_raw(data as *const T));
    Woke::wake_by_ref(&arc);
}

unsafe fn drop_arc_raw<T: Woke>(data: *const ()) {
    drop(Arc::<T>::from_raw(data as *const T))
}

#[derive(Debug)]
pub struct WakerRef<'a> {
    waker: ManuallyDrop<Waker>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> WakerRef<'a> {
    pub fn new(waker: &'a Waker) -> Self {
        let waker = ManuallyDrop::new(unsafe { core::ptr::read(waker) });
        WakerRef {
            waker,
            _marker: PhantomData,
        }
    }

    pub fn new_unowned(waker: ManuallyDrop<Waker>) -> Self {
        WakerRef {
            waker,
            _marker: PhantomData,
        }
    }
}

impl Deref for WakerRef<'_> {
    type Target = Waker;

    fn deref(&self) -> &Waker {
        &self.waker
    }
}

#[inline]
pub fn waker_ref<W>(wake: &Arc<W>) -> WakerRef<'_>
where
    W: Woke,
{
    let ptr = (&**wake as *const W) as *const ();

    let waker = ManuallyDrop::new(unsafe {
        Waker::from_raw(RawWaker::new(ptr, waker_vtable::<W>()))
    });
    WakerRef::new_unowned(waker)
}
