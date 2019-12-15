use crate::_pasts_hide::stn::{
    boxed::Box,
    sync::{Arc, Condvar, Mutex},
    thread, vec,
    vec::Vec,
};

// A thread that may or may not be running a task for the async loop.
struct JoinHandle {
    mutex: Mutex<Option<()>>,
    condvar: Condvar,
}

// A thread that may or may not be running a task for the async loop.
struct Thread {
    mutex: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    condvar: Condvar,
}

pub(super) struct ThreadPool {
    // Vec of available threads.
    available: Mutex<Vec<ThreadHandle>>,
}

pub(super) struct ThreadHandle {
    join: Arc<JoinHandle>,
    thread: Arc<Thread>,
    pool: Arc<ThreadPool>,
}

impl ThreadHandle {
    pub(super) fn join(self) {
        loop {
            // Lock the mutex.
            let mut guard = self.join.mutex.lock().unwrap();

            // Return if task has completed.
            if guard.take().is_some() {
                break;
            }

            // Wait until not zero (unlock mutex).
            let _guard = self.join.condvar.wait(guard).unwrap();
        }

        // Add back to the stack, as task has completed.
        let pool = self.pool.clone();
        let mut stack = pool.available.lock().unwrap();

        (*stack).push(self);
    }
}

impl ThreadPool {
    /// Create a new ThreadPool.
    pub fn new() -> Arc<Self> {
        Arc::new(ThreadPool {
            available: Mutex::new(vec![]),
        })
    }

    // Take a thread off of the pool stack, and start execution.
    pub fn spawn<F>(self: Arc<Self>, function: F) -> ThreadHandle
    where
        F: FnOnce() + Send + 'static,
    {
        let mut stack = self.available.lock().unwrap();

        if let Some(thread) = (*stack).pop() {
            *thread.thread.mutex.lock().unwrap() = Some(Box::new(function));
            thread.thread.condvar.notify_one();

            thread
        } else {
            let data = Arc::new(Thread {
                mutex: Mutex::new(None),
                condvar: Condvar::new(),
            });
            let thread_data = data.clone();

            let join_handle = Arc::new(JoinHandle {
                mutex: Mutex::new(None),
                condvar: Condvar::new(),
            });
            let thread_join_handle = join_handle.clone();

            *data.mutex.lock().unwrap() = Some(Box::new(function));
            data.condvar.notify_one();

            thread::spawn(move || {
                // Wait for tasks.
                loop {
                    // Lock the mutex.
                    let mut guard = thread_data.mutex.lock().unwrap();

                    // Return if task has completed.
                    if let Some(closure) = guard.take() {
                        closure();
                        *thread_join_handle.mutex.lock().unwrap() = Some(());
                        thread_join_handle.condvar.notify_one();
                    }

                    // Wait until not zero (unlock mutex).
                    let _guard = thread_data.condvar.wait(guard).unwrap();
                }
            });

            ThreadHandle {
                thread: data,
                join: join_handle,
                pool: self.clone(),
            }
        }
    }
}
