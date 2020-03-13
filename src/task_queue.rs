use crate::_pasts_hide::stn::{future::Future, task::{Context, Poll}, pin::Pin};

/// Create a task queue from futures.
///
/// ```
/// async fn async_main() {
///     pasts::task_queue!(task_queue = [async { "Hello" }, async { "World!" }]);
///     assert_eq!((0, "Hello"), task_queue.select().await);
///     assert_eq!((1, "World!"), task_queue.select().await);
/// }
/// 
/// <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(async_main());
/// ```
#[macro_export]
macro_rules! task_queue {
    ($x:ident = [$($y:expr),* $(,)?]) => {
        // Allocate buffer for task queue.
        let __pasts_task_queue: &mut [Option<$crate::_pasts_hide::stn::pin::Pin<&mut dyn $crate::_pasts_hide::stn::future::Future<Output = _>>>] = &mut [
            $(
                {
                    {&$y};
                    None
                }
            ),*
        ];

        // Fill buffer with pinned futures.
        let mut __pasts_task_queue_count = 0;
        $(
            // Force move (don't use this identifier from this point on).
            let mut __pasts_temp_future = { $y };
            // Shadow use to prevent future use that could move it.
            let mut __pasts_temp_future = &mut __pasts_temp_future;
            // Safely create Pin 
            __pasts_task_queue[__pasts_task_queue_count] =
                Some(
                    $crate::_pasts_hide::new_pin(__pasts_temp_future)
                );
            __pasts_task_queue_count += 1;
        )*

        // Make task slice
        __pasts_task_queue_count = 0;
        let __pasts_task_queue: &mut [(bool, $crate::_pasts_hide::stn::pin::Pin<&mut dyn $crate::_pasts_hide::stn::future::Future<Output = _>>)] = &mut [
            $(
                (true, {
                    {&$y};
                    let ret = __pasts_task_queue[__pasts_task_queue_count].take().unwrap();
                    __pasts_task_queue_count += 1;
                    ret
                })
            ),*
        ];

        // Turn task slice into TaskQueue structure
        let mut $x = $crate::TaskQueue::new(__pasts_task_queue);
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

    /// Wait for the first task to complete.
    /// Returns a future that may be `.await`ed.
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
