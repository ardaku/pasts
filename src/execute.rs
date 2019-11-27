use crate::_pasts_hide::stn::{
    future::Future,
    task::{Context, Poll},
};

#[cfg(feature = "std")]
use crate::_pasts_hide::stn::sync::{
    Condvar, Mutex
};

#[cfg(not(feature = "std"))]
use crate::_pasts_hide::stn::sync::atomic::{AtomicBool, Ordering};

use crate::{let_pin, Wake};

/// Run a future to completion on the current thread.  This will cause the
/// current thread to block.
///
/// ```rust
/// let ret = pasts::block_on(
///     async {
///         /* Do some work, calling .await on futures */
///         "Complete!"
///     }
/// );
/// assert_eq!(ret, "Complete!");
/// ```
pub fn block_on<F: Future>(f: F) -> <F as Future>::Output {
/*    #[cfg(feature = "std")]
    static mut FUTURE_CONDVARS: MaybeUninit<(Mutex<bool>, Condvar)> =
        MaybeUninit::uninit();

    #[cfg(not(feature = "std"))]
    static mut FUTURE_CONDVARS: AtomicBool = AtomicBool::new(true);*/

    #[cfg(feature = "std")]
    pub struct FutureTask(Mutex<bool>, Condvar);

    #[cfg(not(feature = "std"))]
    pub struct FutureTask(AtomicBool);

    impl Wake for FutureTask {
        #[cfg(feature = "std")]
        fn wake_up(&self) {
            *self.0.lock().unwrap() = true;
            self.1.notify_one();
        }

        #[cfg(not(feature = "std"))]
        fn wake_up(&self) {
            self.0.store(true, Ordering::Relaxed);
        }
    }

    let_pin! { future_one = f; }

    #[cfg(feature = "std")]
    let_pin! { task = FutureTask(Mutex::new(true), Condvar::new()); };

    #[cfg(not(feature = "std"))]
    let_pin! { task = FutureTask(AtomicBool::new(true)); };

    // Check for any futures that are ready
    #[cfg(not(feature = "std"))]
    loop {
        if task.0.load(Ordering::Relaxed) {
            // This runs whenever woke.
            let waker = FutureTask::into_waker(&*task);
            let context = &mut Context::from_waker(&waker);
            match future_one.as_mut().poll(context) {
                // Go back to "sleep".
                Poll::Pending => task.0.store(false, Ordering::Relaxed),
                Poll::Ready(ret) => break ret,
            }
        }
    }

    #[cfg(feature = "std")]
    let mut guard = task.0.lock().unwrap();
    #[cfg(feature = "std")]
    loop {
        // Save some processing, by putting the thread to sleep.
        if !(*guard) {
            guard = task.1.wait(guard).unwrap();
        }
        if *guard {
            // This runs whenever woke.
            let task = FutureTask::into_waker(&*task);
            let context = &mut Context::from_waker(&task);
            match future_one.as_mut().poll(context) {
                // Go back to "sleep".
                Poll::Pending => *guard = false,
                Poll::Ready(ret) => break ret,
            }
        }
    }
}

/// Poll two futures concurrently, and return a tuple of returned values from
/// each future.  Only usable inside async functions and blocks.
///
/// Futures that are ready first will be executed first.  This makes
/// `join!(a, b)` faster than the alternative `(a.await, b.await)`.
///
/// ```rust
/// async fn one() -> char {
///     'c'
/// }
///
/// async fn two() -> char {
///     'a'
/// }
///
/// async fn example() {
///     // Joined await on the two futures.
///     let a = one();
///     let b = two();
///     let ret = pasts::join!(a, b);
///     assert_eq!(ret, ('c', 'a'));
/// }
///
/// pasts::block_on(example());
/// ```
#[macro_export]
macro_rules! join {
    ($($y:expr),* $(,)?) => {
        {
            ('c', 'a')
        }
    }
}

/// Poll multiple futures at once, and run the future that is ready first.  Only
/// usable inside async functions and blocks.
///
/// The API is like a match statement: `match_pattern = future => expression`.
/// The expression will be run with the pattern (returned from the future) in
/// scope when the future is the first to complete.  This usage is the same as
/// the one from the futures crate.
///
/// # Example
/// ```rust
/// use core::{
///     pin::Pin,
///     future::Future,
///     task::{Poll, Context},
/// };
///
/// #[derive(Debug, PartialEq)]
/// enum Select {
///     One(i32),
///     Two(char),
/// }
///
/// pub struct AlwaysPending();
///
/// impl Future for AlwaysPending {
///     type Output = i32;
///
///     fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<i32> {
///         Poll::Pending
///     }
/// }
///
/// async fn two() -> char {
///     'c'
/// }
///
/// async fn example() -> Select {
///     let a_fut = AlwaysPending();
///     let b_fut = two();
///
///     pasts::select!(
///         a = a_fut => {
///             println!("This will never print!");
///             Select::One(a)
///         }
///         b = b_fut => Select::Two(b)
///     )
/// }
///
/// assert_eq!(pasts::block_on(example()), Select::Two('c'));
/// ```
#[macro_export] macro_rules! select {
    ($($pattern:pat = $var:ident => $branch:expr $(,)?)*) => {
        {
            use $crate::{let_pin, _pasts_hide::stn::{future::Future, pin::Pin}};
            let_pin! { $( $var = $var; )* }
            struct Selector<'a, T> {
                closure: &'a mut dyn FnMut(&mut Context<'_>) -> Poll<T>,
            }
            impl<'a, T> Future for Selector<'a, T> {
                type Output = T;
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
                    (self.get_mut().closure)(cx)
                }
            }
            Selector { closure: &mut |cx: &mut Context<'_>| {
                $(
                    match Future::poll($var.as_mut(), cx) {
                        Poll::Ready(r) =>
                            return Poll::Ready((&mut |$pattern| $branch)(r)),
                        Poll::Pending => {},
                    }
                )*
                Poll::Pending
            } }.await
        }
    };
}
