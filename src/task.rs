// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

// Compensate for Box not being in the prelude on no-std.
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

/// A boxed, pinned future.
///
/// You should use this in conjunction with
/// [`Loop::poll`](crate::Loop::poll) if you need a dynamic number of tasks (for
/// instance, a web server).
///
/// # Example
/// This example spawns two tasks on the same thread, and then terminates when
/// they both complete.
///
/// ```
/// use core::task::Poll;
/// use pasts::{Executor, Loop, Task};
///
/// type Exit = ();
///
/// struct State {
///     tasks: Vec<Task<&'static str>>,
/// }
///
/// impl State {
///     fn completion(&mut self, id: usize, val: &str) -> Poll<Exit> {
///         self.tasks.remove(id);
///         println!("Received message from completed task: {}", val);
///         if self.tasks.is_empty() {
///             Poll::Ready(())
///         } else {
///             Poll::Pending
///         }
///     }
/// }
///
/// async fn run() {
///     let mut state = State {
///         tasks: vec![Box::pin(async { "Hello" }), Box::pin(async { "World" })],
///     };
///
///     Loop::new(&mut state)
///         .poll(|s| &mut s.tasks, State::completion)
///         .await;
/// }
///
/// fn main() {
///     Executor::default().block_on(run());
/// }
/// ```
pub type Task<T> = core::pin::Pin<Box<dyn core::future::Future<Output = T>>>;
