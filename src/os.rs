// Daku
#[cfg_attr(all(target_arch = "wasm32", daku), path = "os/daku.rs")]
// Std
#[cfg_attr(
    all(
        feature = "std",
        not(all(target_arch = "wasm32", any(daku, feature = "web")))
    ),
    path = "os/std.rs"
)]
// Web
#[cfg_attr(all(target_arch = "wasm32", feature = "web"), path = "os/web.rs")]
mod target;

use alloc::{boxed::Box, sync::Arc, task::Wake, vec::Vec};
use core::{fmt::Debug, pin::Pin, task::Context};

use crate::{LocalBoxFuture, Park, Poll, Pool};

/// Implement `Target for Os` to add platform support for a target.
pub(crate) struct Os;

/// Target platform support
pub(crate) trait Target: Sized {
    type ParkCx: Debug + Default;

    /// Stop doing anything on the CPU or thread until unparked.
    #[inline(always)]
    fn park(self, _park_cx: &Self::ParkCx) {
        // Default implementation doesn't actually park for maximum portability.
        //
        // Hint at spin loop to possibly save CPU time with a short sleep.
        core::hint::spin_loop();
    }

    /// Resume execution on the parked CPU or thread.
    #[inline(always)]
    fn unpark(self, _park_cx: &Self::ParkCx) {}

    /// Spawn a local future.
    #[inline(always)]
    fn spawn<P: Pool>(self, pool: &P, f: impl Future<Output = ()> + 'static) {
        self.spawn_boxed(pool, Box::pin(f));
    }

    /// Spawn a local future.
    #[inline(always)]
    fn spawn_boxed<P: Pool>(self, pool: &P, f: LocalBoxFuture<'static>) {
        pool.push(f);
    }

    /// Block on a local future (if the platform allows it, otherwise spawn).
    #[inline(always)]
    fn block_on<P: Pool>(
        self,
        pool: &P,
        f: impl Future<Output = ()> + 'static,
    ) {
        struct Unpark<P: Park>(P);

        impl<P: Park> Wake for Unpark<P> {
            #[inline(always)]
            fn wake(self: Arc<Self>) {
                self.wake_by_ref();
            }

            #[inline(always)]
            fn wake_by_ref(self: &Arc<Self>) {
                self.0.unpark();
            }
        }

        // Box and pin main task
        let f: LocalBoxFuture<'_> = Box::pin(f);
        // Set up the notify
        let tasks = &mut Vec::new();
        // Set up the park, waker, and context
        let unpark = Arc::new(Unpark(<P as Pool>::Park::default()));
        let waker = unpark.clone().into();
        let cx = &mut Context::from_waker(&waker);
        // Which task's turn it is (for basic fairness)
        let mut index = 0;

        // Spawn main task
        tasks.push(f);

        // Run the set of futures to completion.
        while !tasks.is_empty() {
            // Wrap index
            index %= tasks.len();

            // Poll the entire set of futures on wake
            let poll = 'poll: {
                for (i, this) in tasks.iter_mut().skip(index).enumerate() {
                    if let Poll::Ready(()) = Pin::new(this).poll(cx) {
                        break 'poll Poll::Ready(i);
                    }
                }

                for (i, this) in tasks.iter_mut().take(index).enumerate() {
                    if let Poll::Ready(()) = Pin::new(this).poll(cx) {
                        break 'poll Poll::Ready(i);
                    }
                }

                // Take turns which task polls first
                index += 1;
                break 'poll Poll::Pending;
            };
            // If no tasks have completed, then park
            let Poll::Ready(task_index) = poll else {
                // Initiate execution of any spawned tasks - if no new tasks,
                // park
                if !pool.drain(tasks) {
                    unpark.0.park();
                }

                continue;
            };

            // Task has completed, drop it
            drop(tasks.swap_remove(task_index));
            // Drain any spawned tasks into the pool
            pool.drain(tasks);
        }
    }
}
