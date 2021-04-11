// Pasts
// Copyright © 2019-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use core::future::Future;
use core::pin::Pin;

#[cfg(any(target_arch = "wasm32", not(feature = "std")))]
use alloc::boxed::Box;

/// A boxed, pinned future.
///
/// # Example
/// This example spawns two tasks on the same thread, and then terminates when
/// they both complete.
///
/// ```
/// use core::task::Poll;
/// use pasts::{Task, Exec, Loop};
///
/// type Exit = ();
///
/// struct State {
///     tasks: Vec<Task<&'static str>>,
/// }
///
/// impl State {
///     fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
///         self.tasks.remove(id);
///         println!("Received message from completed task: {}", val);
///         if self.tasks.is_empty() {
///             Poll::Ready(())
///         } else {
///             Poll::Pending
///         }
///     }
///
///     fn event_loop(&mut self, exec: Exec<Self, Exit>) -> impl Loop<Exit> {
///         exec.poll(&mut self.tasks, Self::completion)
///     }
/// }
///
/// async fn run() {
///     let mut tasks = State {
///         tasks: vec![
///             Box::pin(async { "Hello" }),
///             Box::pin(async { "World" }),
///         ]
///     };
///
///     pasts::event_loop(&mut tasks, State::event_loop).await;
/// }
///
/// fn main() {
///     pasts::block_on(run())
/// }
/// ```
pub type Task<T> = Pin<Box<dyn Future<Output = T>>>;
