use crate::_pasts_hide::stn::{
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, Once,
    },
    task::{Context, Poll, Waker},
};

mod thread_pool;

use thread_pool::ThreadPool;

struct ThreadFuture<R> {
    shared_state: Arc<(Mutex<Option<Waker>>, AtomicBool)>,
    handle: Option<thread_pool::ThreadHandle>,
    ret: Arc<Mutex<Option<R>>>,
}

impl<R> Future for ThreadFuture<R> {
    type Output = R;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        if !self.shared_state.1.load(Ordering::Relaxed) {
            let mut shared_state = self.shared_state.0.lock().unwrap();

            *shared_state = Some(cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready({
                self.handle.take().unwrap().join();
                self.ret.lock().unwrap().take().unwrap()
            })
        }
    }
}

impl<R> ThreadFuture<R> {
    pub fn new<F>(function: F) -> Self
    where
        F: FnOnce() -> R,
        F: Send + 'static,
        R: Send + 'static,
    {
        let shared_state: Arc<(Mutex<Option<Waker>>, AtomicBool)> =
            Arc::new((Mutex::new(None), AtomicBool::new(false)));

        let thread_shared_state = shared_state.clone();

        let ret = Arc::new(Mutex::new(None));
        let thread_ret = ret.clone();

        let handle = Some(thread_pool().spawn(move || {
            *thread_ret.lock().unwrap() = Some(function());
            let mut shared_state = thread_shared_state.0.lock().unwrap();
            thread_shared_state.1.store(true, Ordering::Relaxed);
            if let Some(waker) = shared_state.take() {
                waker.wake()
            }
        }));

        ThreadFuture {
            shared_state,
            handle,
            ret,
        }
    }
}

static mut THREAD_POOL: MaybeUninit<Arc<ThreadPool>> = MaybeUninit::uninit();
static START: Once = Once::new();

// Return the global thread pool.
#[allow(unsafe_code)]
fn thread_pool() -> Arc<ThreadPool> {
    START.call_once(|| unsafe {
        ptr::write(THREAD_POOL.as_mut_ptr(), ThreadPool::new());
    });

    unsafe { (*THREAD_POOL.as_ptr()).clone() }
}

/// **std** feature required.  Construct a future from a blocking function.  The
/// function will be run on a separate thread in a dynamically sized thread pool.
pub fn spawn_blocking<F, R>(function: F) -> impl Future<Output = R>
where
    F: FnOnce() -> R,
    F: Send + 'static,
    R: Send + 'static,
{
    ThreadFuture::new(function)
}
