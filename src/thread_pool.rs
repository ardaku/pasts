use crate::_pasts_hide::stn::{
    thread, vec, vec::Vec
};

struct Thread {
    // Whether or not thread is busy
    busy: bool,
    //
    
    //
    
}

/// **std** feature required.  A thread pool to offload expensive tasks to from
/// the async loop.
pub struct ThreadPool {
    // Whether or not thread is busy, thread handle.
    threads: Vec<(bool, )>,
}

// A thread that can run futures.
fn futures_thread() {
    loop {
        
    }
}

impl ThreadPool {
    /// Spawn `count` threads for offloading expensive tasks to from the async
    /// loop.
    pub fn new(count: usize) -> Self {
        for i in 0..count {
            thread::spawn(|| {});
        }

        ThreadPool {
            threads: vec![(false, ); count],
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        
    }
}
