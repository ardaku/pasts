use crate::_pasts_hide::stn::{
    future::Future,
    pin::Pin,
    sync::{Arc, Condvar, Mutex, Once, atomic::{AtomicBool, Ordering}},
    task::{Context, Poll, Waker},
    mem::{MaybeUninit},
    vec::Vec, vec,
    marker::PhantomData,
    ptr,
};

mod thread_pool;

use thread_pool::ThreadPool;

// A task on a thread
trait ThreadTask {
    type Output;

    
}

/*enum ThreadState<R> {
    Working(&'static dyn Fn()),
    Done(R),
}*/

struct Thread<R> {
    mutex: Mutex<Option<R>>,
    condvar: Condvar,
}

// Waiting on thread for tasks.
struct TaskWaiter<R> {
    mutex: Mutex<Option<R>>,
    condvar: Condvar,
}

impl<R> Thread<R> {
    fn join(&self) -> R {
        loop {
            // Lock the mutex.
            let mut guard = self.mutex.lock().unwrap();

            // Return if task has completed.
            if let Some(ret) = guard.take() {
                return ret;
            }

            // Wait until not zero (unlock mutex).
            let _guard = self.condvar.wait(guard).unwrap();
        }
    }
}

struct ThreadFuture {
    shared_state: Arc<(Mutex<Option<Waker>>, AtomicBool)>,
    handle: Option<thread_pool::ThreadHandle>,
}

impl Future for ThreadFuture {
    type Output = (); // R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Self::Output>
    {
        if !self.shared_state.1.load(Ordering::Relaxed) {
            let mut shared_state = self.shared_state.0.lock().unwrap();

            *shared_state = Some(cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(self.handle.take().unwrap().join())
        }
    }
}

impl ThreadFuture {
    pub fn new<F>(function: F) -> Self
    where
        F: FnOnce(),
        F: Send + 'static,
    {
        let shared_state: Arc<(Mutex<Option<Waker>>, AtomicBool)> = Arc::new((Mutex::new(None), AtomicBool::new(false)));

        let thread_shared_state = shared_state.clone();
        let handle = Some(thread_pool().spawn(move || {
            let ret = function();
            let mut shared_state = thread_shared_state.0.lock().unwrap();
            thread_shared_state.1.store(true, Ordering::Relaxed);
            if let Some(waker) = shared_state.take() {
                waker.wake()
            }
            ret
        }));

        ThreadFuture { shared_state, handle }
    }
}

static mut THREAD_POOL: MaybeUninit<Arc<ThreadPool>> = MaybeUninit::uninit();
static START: Once = Once::new();

// Return the global thread pool.
#[allow(unsafe_code)]
fn thread_pool() -> Arc<ThreadPool> {
    START.call_once(|| {
        unsafe {
            ptr::write(THREAD_POOL.as_mut_ptr(), ThreadPool::new());
        }
    });

    unsafe {
        (*THREAD_POOL.as_ptr()).clone()
    }
}

/// **std** feature required.  Construct a future from a blocking function.  The
/// function will be run on a separate thread in a dynamically sized thread pool.
pub fn spawn_blocking<F>(function: F) -> impl Future<Output = ()>
where
    F: FnOnce(),
    F: Send + 'static,
//    R: Send + 'static
{
    ThreadFuture::new(function)
}
