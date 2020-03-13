use crate::_pasts_hide::stn::{future::Future, task::{Context, Poll}, pin::Pin};

/// Create a task queue from futures.
///
/// ```
/// async fn async_main() {
///     let hello = async { "Hello" };
///     pasts::task_queue!(task_queue = [hello, async { "World!" }]);
///     assert_eq!((0, "Hello"), task_queue.select().await);
///     assert_eq!((1, "World!"), task_queue.select().await);
/// }
/// 
/// <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(async_main());
/// ```
#[macro_export]
macro_rules! task_queue {
    ($x:ident = [$($y:expr),* $(,)?]) => {
        use $crate::_pasts_hide::stn::{
            pin::Pin,
            future::Future,
        };

        // Allocate buffer for task queue.
        let queue = &mut [
            $(
                {
                    if false { { &$y }; }
                    $crate::_pasts_hide::stn::mem::MaybeUninit::<
                        (bool, Pin<&mut dyn Future<Output = _>>)
                    >::uninit()
                }
            ),*
        ][..];

        // Fill buffer with pinned futures.
        let mut count = 0;
        $(
            // Force move (don't use this identifier from this point on).
            let mut temp_future = { $y };
            // Shadow use to prevent future use that could move it.
            let mut temp_future = &mut temp_future;
            // Safely create Pin 
            queue[count] =
                $crate::_pasts_hide::stn::mem::MaybeUninit::new(
                    (true, $crate::_pasts_hide::new_pin(temp_future))
                );
            count += 1;
        )*

        let mut queue = $crate::_pasts_hide::transmute_slice(queue);

        // Turn task slice into TaskQueue structure
        let mut $x = $crate::TaskQueue::new(queue);
    };
}

/// Asynchronous task queue for running multiple futures on the same thread.
pub struct TaskQueue<'a, T> {
    tasks: &'a mut [(bool, Pin<&'a mut dyn Future<Output = T>>)],
}

impl<'a, T> TaskQueue<'a, T> {
    #[doc(hidden)]
    pub fn new(tasks: &'a mut [(bool, Pin<&'a mut dyn Future<Output = T>>)]) -> Self {
        Self { tasks }
    }

    /// Poll multiple futures concurrently, and return the future that is ready
    /// first.
    ///
    /// # Example
    /// ```
    /// async fn async_main() {
    ///     pasts::task_queue!(task_queue = [async { "Hello" }, async { "World!" }]);
    ///     assert_eq!((0, "Hello"), task_queue.select().await);
    ///     assert_eq!((1, "World!"), task_queue.select().await);
    /// }
    /// 
    /// <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(async_main());
    /// ```
    pub fn select<'b>(&'b mut self) -> Select<'a, 'b, T> {
        Select { task_queue: self }
    }

    /// Get the number of tasks the queue was initialized with.
    pub fn capacity(&self) -> usize {
        self.tasks.len()
    }
}

pub struct Select<'a, 'b, T> {
    task_queue: &'b mut TaskQueue<'a, T>,
}

impl<'a, 'b, T> Future for Select<'a, 'b, T> {
    type Output = (usize, T);
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        for task in 0..self.task_queue.tasks.len() {
            if self.task_queue.tasks[task].0 {
                match self.task_queue.tasks[task].1.as_mut().poll(cx) {
                    Poll::Ready(ret) => {
                        self.task_queue.tasks[task].0 = false;
                        return Poll::Ready((task, ret))
                    },
                    Poll::Pending => {},
                }
            }
        }
        Poll::Pending
    }
}
