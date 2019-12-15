/*use crate::_pasts_hide::stn::{
    thread, vec, vec::Vec
};

struct Thread {
}

/// **std** feature required.  A thread pool to offload expensive tasks to from
/// the async loop.
pub struct ThreadPool {
    // Whether or not thread is busy, thread handle.
    threads: Vec<Thread>,
    // Available thread indices.
    unused: Vec<usize>,
}

// A thread that can run futures.
impl ThreadPool {
    /// New thead pool for offloading expensive tasks to from the async loop.
    pub fn new() -> Self {
        // Start with 2 threads, we can add more if needed.
        Self::with_capacity(2)
    }

    /// New thead pool for offloading expensive tasks to from the async loop.
    pub fn with_capacity(num: usize) -> Self {
        let mut threads: Vec::with_capacity(num);
        let mut unused: Vec::with_capacity(num);

        for i in 0..num {
            unused.push(i);

            thread.push(Thread {
                busy: false,
            });

            thread::spawn(move || {
                loop {
                }
            });
        }

        ThreadPool { threads,  }
    }

    /// Spawn a blocking call as a task on this thread pool.
    pub fn spawn() {
        
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        
    }
}*/
