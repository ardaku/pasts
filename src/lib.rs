// Copyright © 2019-2022 The Pasts Contributors.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).
//
//! Minimal and simpler alternative to the futures crate.
//!
//! # Optional Features
//! The **std** feature is enabled by default, disable it to use on `no_std`.
//!
//! # Getting Started
//! This example runs two timers in parallel using the `async-std` crate
//! counting from 0 to 6.  The "one" task will always be run for count 6 and
//! stop the program, although which task will run for count 5 may be either
//! "one" or "two" because they trigger at the same time.
//!
//! Add the following to your *Cargo.toml*:
//!
//! ```toml
//! [dependencies]
//! pasts = "0.8"
//! aysnc-std = "1.0"
//! ```
//!
//! ```rust,no_run
//! use async_std::task::sleep;
//! use core::future::Future;
//! use core::task::Poll;
//! use core::time::Duration;
//! use pasts::{Loop, Past};
//!
//! // Exit type for State.
//! type Exit = ();
//!
//! // Shared state between tasks on the thread.
//! struct State<A: Future<Output = ()>, B: Future<Output = ()>> {
//!     counter: usize,
//!     one: Past<(), (), A>,
//!     two: Past<(), (), B>,
//! }
//!
//! impl<A: Future<Output = ()>, B: Future<Output = ()>> State<A, B> {
//!     fn one(&mut self, _: ()) -> Poll<Exit> {
//!         println!("One {}", self.counter);
//!         self.counter += 1;
//!         if self.counter > 6 {
//!             Poll::Ready(())
//!         } else {
//!             Poll::Pending
//!         }
//!     }
//!
//!     fn two(&mut self, _: ()) -> Poll<Exit> {
//!         println!("Two {}", self.counter);
//!         self.counter += 1;
//!         Poll::Pending
//!     }
//! }
//!
//! async fn run() {
//!     let mut state = State {
//!         counter: 0,
//!         one: Past::new((), |()| sleep(Duration::from_secs_f64(1.0))),
//!         two: Past::new((), |()| sleep(Duration::from_secs_f64(2.0))),
//!     };
//!
//!     Loop::new(&mut state)
//!         .when(|s| &mut s.one, State::one)
//!         .when(|s| &mut s.two, State::two)
//!         .await;
//! }
//!
//! fn main() {
//!     pasts::block_on(run())
//! }
//! ```
#![cfg_attr(not(feature = "std"), no_std)]
#![doc(
    html_logo_url = "https://ardaku.github.io/mm/logo.svg",
    html_favicon_url = "https://ardaku.github.io/mm/icon.svg",
    html_root_url = "https://docs.rs/pasts"
)]
#![forbid(unsafe_code)]
#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]

use alloc::{task::Wake, sync::Arc, rc::Rc};
use core::{cell::Cell, pin::Pin, task::{Context, Poll}, future::Future, marker::PhantomData, sync::{atomic::{Ordering, AtomicUsize}}};

#[cfg(feature = "std")]
thread_local! {
    static LOCAL: Local = Local::default();
}

#[cfg(not(feature = "std"))]
static LOCAL: Local = Local::default();

#[derive(Default)]
struct Local {
    /// Unique identifier for awoken task, zero if none
    which: AtomicUsize,
    /// Pasts executor
    executor: Cell<Option<Rc<dyn Executor>>>,
}

impl Local {
    fn executor(&self) -> Option<Rc<dyn Executor>> {
        let executor = self.executor.take();
        self.executor.set(executor.clone());
        executor
    }
}

// Thread local waker usable on the pasts executor
struct LocalWaker;

impl Wake for LocalWaker {
    #[inline(always)]
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    #[inline(always)]
    fn wake_by_ref(self: &Arc<Self>) {
        // Set thread-local waker pointer
        let ptr = Arc::as_ptr(self) as usize;
        
        #[cfg(feature = "std")]
        LOCAL.with(|local| local.which.store(ptr, Ordering::SeqCst));

        #[cfg(not(feature = "std"))]
        LOCAL.which.store(ptr, Ordering::SeqCst);

        // Wake thread-local executor
        #[cfg(feature = "std")]
        LOCAL.with(|local| local.executor().as_ref().unwrap().wake());

        #[cfg(not(feature = "std"))]
        LOCAL.executor().as_ref().unwrap().wake();
    }
}
/*
struct ForeignWaker {
    waker: Waker,
    index: Arc<AtomicUsize>,
}

impl Wake for ForeignWaker {
    #[inline(always)]
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    #[inline(always)]
    fn wake_by_ref(self: &Arc<Self>) {
        // Set foreign waker pointer
        self.index.store(Arc::as_ptr(self) as usize, Ordering::SeqCst);
        // Wake foreign-executor
        self.waker.wake_by_ref();
    }
}
*/
/*

/// Create waker of the appropriate type
///  - `LocalWaker` when on the pasts executor
///  - `ForeignWaker` when not on the pasts executor
fn context(cx: &mut Context<'_>) -> Waker {
    if LOCAL.with(|local| local.executor().is_some()) {
        Arc::new(LocalWaker).into()
    } else {
        let waker = cx.waker().clone();
        Arc::new(ForeignWaker { waker, index }).into()
    }
}

*/






/*
trait StatefulFuture<O, S> {
    fn poll(&mut self, cx: &mut Context<'_>, state: &mut S) -> Poll<O>;
}

/// A stateful future that runs two `Past`s in parallel
struct DualPast<A, B, F, G, T, U, S, R, M, N>
    where
        F: Future<Output = T>,
        G: Future<Output = U>,

{
    past_a: Past<T, F, A, M>,
    past_b: Past<U, G, B, N>,
    fut_a: F,
    fut_b: G,
    fn_a: fn(&mut S, T) -> Poll<R>,
    fn_b: fn(&mut S, U) -> Poll<R>,
}

impl<A, B, F, G, T, U, S, R> DualPast<A, B, F, G, T, U, S, R>
{
    fn new(past_a: A, past_b: B, fn_a: F, fn_b: G) -> Self {
        let fut_a = past_a.next();
        let fut_b = past_b.next();

        Self {
            past_a,
            past_b,
            fut_a,
            fut_b,
            fn_a,
            fn_b,
        }
    }
}

impl<A, B, F, G, T, U, S, R> StatefulFuture<Poll<R>, S>
    for DualPast<A, B, F, G, T, U, S, R>
{
    #[inline(always)]
    fn poll(&mut self, cx: &mut Context<'_>, state: &mut S) -> Poll<Poll<R>> {
        match Pin::new(&mut self.fut_a).poll(cx) {
            Poll::Pending => match Pin::new(&mut self.fut_b).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(output) => {
                    self.fut_b = self.past_b.next();
                    Poll::Ready(self.fn_a(state, output))
                }
            },
            Poll::Ready(output) => {
                self.fut_a = self.past_a.next();
                Poll::Ready(self.fn_b(state, output))
            }
        }
    }
}
*/

/*
/// Trait for joining [`Past`]s with a shared state.
pub trait Join {
}

impl<T> Join for &mut T { }



/// Asynchronous event loop builder.
#[derive(Debug)]
pub struct LoopBuilder<'a, S, T, J> {
    state: &'a mut S,
    join: J,
    _phantom: PhantomData<Pin<Box<T>>>,
}

impl<'a, S, T> LoopBuilder<'a, S, T, core::future::Pending<T>> {
    /// Create a new asynchronous event loop with associated state.
    #[inline(always)]
    pub fn new(state: &'a mut S) -> Self {
        let join = core::future::pending();
        let _phantom = PhantomData;

        Self {
            state,
            join,
            _phantom,
        }
    }
}

impl<'a, S, T, J> LoopBuilder<'a, S, T, J>
    where J: Future<Output = T> + Unpin
{
    /// Set an event callback.
    #[inline(always)]
    pub fn on<P, O, F, R>(self, past: P, callback: fn(&mut S, O) -> Poll<T>)
        -> LoopBuilder<'a, S, T, impl Future<Output = T>>
    where
        P: Into<Past<O, F, R>>,
        F: Future<Output = O> + Send + Unpin,
    {
        let past = past.into();

        self
    }
}

impl<S, T, J: Future<Output = T>> Future for LoopBuilder<'_, S, T, J>
    where J: Future<Output = T> + Unpin
{
    type Output = T;

    #[inline(always)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        Pin::new(&mut self.join).poll(cx)
    }
}
*/

extern crate alloc;

mod exec;
mod past;
mod race;
mod task;

pub use exec::{block_on, Executor, BlockOn};
pub use past::Past;
pub use race::Loop;
pub use task::Task;
