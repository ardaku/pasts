use crate::_pasts_hide::stn::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    task::{Context, Poll, Waker},
    thread,
};

struct SharedState {
    waker: Option<Waker>,
}

struct ThreadFuture<R> {
    shared_state: Arc<(Mutex<SharedState>, AtomicBool)>,
    handle: Option<thread::JoinHandle<R>>,
}

impl<R> Future for ThreadFuture<R> {
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Self::Output>
    {
        if !self.shared_state.1.load(Ordering::Relaxed) {
            let mut shared_state = self.shared_state.0.lock().unwrap();

            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(self.handle.take().unwrap().join().unwrap())
        }
    }
}

impl<R> ThreadFuture<R> where R: Send + 'static {
    pub fn new<F>(function: F) -> Self
    where
        F: FnOnce() -> R,
        F: Send + 'static,
    {
        let shared_state = Arc::new((Mutex::new(SharedState {
            waker: None,
        }), AtomicBool::new(false)));

        let thread_shared_state = shared_state.clone();
        let handle = Some(thread::spawn(move || {
            let ret = function();
            let mut shared_state = thread_shared_state.0.lock().unwrap();
            thread_shared_state.1.store(true, Ordering::Relaxed);
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
            ret
        }));

        ThreadFuture { shared_state, handle }
    }
}

/// **std** feature required.  Construct a future from a blocking function.  The
/// function will be run on a separate thread in a dynamically sized thread pool.
pub fn spawn_blocking<F, R>(function: F) -> impl Future<Output = R>
where
    F: FnOnce() -> R,
    F: Send + 'static,
    R: Send + 'static
{
    ThreadFuture::new(function)
}
