# Changelog
All notable changes to `pasts` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://jeronlau.tk/semver/).

## [0.13.0] - 2023-01-16
### Added
 - Added `Spawn` trait for spawning both `Send` futures and local tasks.
 - Re-export of `Spawn` in the `prelude`.
 - `Executor::block_on()`
 - Defaults for `T = ()` on both `BoxNotifier` and `Notifier`
 - *`std`* feature, enabled by default

### Changed
 - `prelude::Poll` is now a type alias with generic default set to unit
 - Recommend using `async_main` crate in docs
 - `core::task::Context` is now re-exported as `Task` instead of `Exec` in
   prelude
 - `Local` type alias is renamed to `LocalBoxNotifier`
 - `Task` type alias is renamed to `BoxNotifier`
 - `Executor` no longer executes on `Drop`; must call `block_on()` instead

### Removed
 - `Sleep`, use `Pool` and `Park` instead
 - *`no-std`* feature, for no-std environments disable *`std`* instead

### Fixed
 - Infinite recursion in `impl<N: Notifier, ..> Notifier for Box<N>`

## [0.12.0] - 2022-07-31
### Added
 - **`no-std`** feature

### Changed
 - `Executor::new()` now takes `impl Into<Arc<I>>` instead of `I`
 - `Executor::spawn()` no longer requires `Unpin` futures
 - `Sleep` trait now requires `Send + Sync + 'static`
 - Started using `core::hint::spin_loop()` for default no-std executor

### Removed
 - **`std`** feature - to use pasts on no-std environments use the new
   **`no-std`** feature instead

## [0.11.0] - 2022-06-10
### Added
 - `Sleep` trait for implementing custom executors
 - `Notifier` trait (like `AsyncIterator`, but infinite)
 - `Poller` struct for compatibility with futures
 - `Fuse` trait for turning `Future`s into `Notifier`s
 - `Executor` struct for custom executors
 - `Loop` struct for a notifier created from future producers
 - `Box`, `Future`, `Pin`, `Exec` (alias to `core::task::Context`), `Executor`,
   `Fuse`, `Local`, `Task` and `Notifier` to prelude.
 - `Local` type alias for `!Send` boxed `Notifier`s
 - `Task` type alias for `Send` boxed `Notifier`s

### Changed
 - `Loop` renamed to `Join`
 - `Join::on()` now takes a closure for the notifier
 - `Task` got split into many different specialized types 

### Removed
 - `poll_next_fn()` in favor of new `Poller` type
 - `block_on()` - all futures must be spawned locally now (this change was made
   to support the same behavior on web assembly as other platforms)
 - `BlockOn` trait in favor of new `Executor` struct
 - `Executor` trait in favor of using new `Sleep` trait in combination with the
   `Wake` trait from the std library.

## [0.10.0] - 2022-05-07
### Added
 - More documentation
 - `poll_next_fn()`

### Changed
 - Completely reimplemented `Task` so it doesn't always require allocation or
   `Send` (it should be more obvious which methods require allocation now)
 - `Loop::on()` accepts different types for the second parameter

## [0.9.0] - 2022-03-27
### Added
 - A `prelude` module containing a `core::task::Poll::{self, Pending, Ready}`
   re-export
 - `Loop::on()`
 - `BlockOn` trait
 - `Task::new()`
 - `Task::poll_next()`

### Changed
 - Replaced `Loop::when` and `Loop::poll` with `Loop::on()`
 - Move `block_on_pinned()` and `block_on` out of `Executor` and into their own
   `BlockOn` trait
 - `Task` is no longer an alias, but its own type

### Removed
 - `Loop::when()` - use `Loop::on()`
 - `Loop::poll()` - use `Loop::on()`

## [0.8.0] - 2021-06-18
### Added
 - `Loop` struct to replace `wait!()` and `exec!()`.
 - `Task` type definition for dynamically spawning tasks.
 - `Executor` trait for implementing custom executors on no-std.
 - `Past` struct for executing `!Unpin` futures.

### Changed
 - Removed all unsafe!
 - Executor no longer terminates program upon future completion.
 - Executor now uses thread parking instead of condvars internally.

### Removed
 - `exec!()` macro - use `Loop::when()` instead.
 - `join!()` macro - use `Loop::poll()` instead.
 - `race!()` macro - use `Loop::poll()` instead.
 - `wait!()` macro - use `Loop::when()` instead.

## [0.7.4] - 2021-01-08
### Fixed
 - Executor never going to sleep, wasting CPU cycles.

## [0.7.3] - 2021-01-07
### Fixed
 - Executor freezing up and not being recoverable (happenned sometimes when two
   asynchronous tasks simultaneously woke the executor).

## [0.7.2] - 2020-12-29
### Fixed
 - Links in README

## [0.7.1] - 2020-12-29
### Fixed
 - Category slug.

## [0.7.0] - 2020-12-29
### Added
 - `block_on()` function to execute a future on a thread.

### Changed
 - Renamed `poll!()` to `race!()`
 - Separated non-array functionality of `poll!()` into new macro: `wait!().
 - `join!()` no longer requires the futures to be pinned, as it can pin them
   itself.
 - `exec!()` macro can no longer start multiple threads, but you can use it on
   multiple threads simultaneously.  `exec!()` no longer takes a future, but
   an asynchronous expression that gets run in an infinite loop.

### Removed
 - `prelude` module.
 - `task!()` macro.
 - `Task` type.

## [0.6.0] - 2020-11-22
### Added
 - `Task` type alias: `Pin<&'a mut (dyn Future<Output = T> + Unpin)>`
 - `task!()` macro to create a `Task` from an async block or function.
 - `exec!()` macro to execute futures from synchronous code, supporting
   parallelization when the **std** feature is enabled, and not on WASM.
 - `poll!()` macro to create a future that returns ready when the first of a
   list of futures returns ready.
 - `join!()` macro to concurrently push multiple futures to completion.

### Removed
 - `DynFuture` and `DynFut` as they were unsound; you may now use the `task!()`
   macro for the same effect.
 - `spawn()` as it was also unsound due to the fact it made executors that did
   not have a `'static` lifetime nor did reference counting; you may now use the
   `exec!()` macro instead.
 - `JoinHandle` - though there are no replacements for this API, you can use
   `exec!()` to create a thread pool with `num_cpus` to execute tasks.
 - `Join` trait - use `join!()` macro, which can take any number of arguments
   rather than being limited to six.
 - `Select` trait - use `poll!()` macro, which automatically changes your
   futures into `Task`s.  It was renamed to `poll!()` because it works
   differently than `select!()` seen elsewhere in the async ecosystem.
 - `SelectBoxed` - no longer needed, `poll!()` works on `Box`es
 - `SelectOptional` - you may now use task queues (`Vec<Task>`), and remove
   tasks from the vec on completion.
 - A lot of unsafe code, and also lines of code (less than 250 instead of over
   1000).

### Fixed
 - At least two unsoundness issues.

## [0.5.0] - 2020-11-14
### Added
 - `spawn()` function to start a non-blocking task that may run on a separate
   thread.
 - `JoinHandle` struct that lets you `.await` on the termination of a task
   started with `spawn()`.
 - `SelectBoxed` and `SelectOptional` traits to reduce boilerplate

### Changed
 - Now the `alloc` crate is required.

### Removed
 - `Executor` trait and `CvarExec` implementation, now you should use `spawn()`
   instead.
 - `spawn_blocking()`, now you should transition to using non-blocking system
   APIs
 - `DynBoxFut`, as it is now completely useless because `DynFut` works on boxes.

## [0.4.0] - 2020-05-17
### Added
 - `DynBoxFut` which can be enabled with the new **alloc** feature.  Useful for
   converting future trait objects into the `DynFuture` type.  Note that enabling
   **std** automatically enables the **alloc** feature.

### Changed
 - Rename `ThreadInterrupt` to `CvarExec`.
 - Rename `Interrupt` to `Executor`.  No longer requires `new()` to be
   implemented, and `block_on` is now a method rather than an associated
   function.  It is still recommended to implement `new()`, and do it as a `const
   fn`.  `wait_for()` method is renamed to `wait_for_event()` and is now marked
   `unsafe` in order to guarantee soundness.  `interrupt` method is now
   `trigger_event()` and marked `unsafe` for the same reason.  An `is_used()`
   method is now required as well.  Executors must now have a static lifetime;
   This is in order to fix the `block_on()` bug mentioned below.

### Fixed
 - After return of `block_on()`, `Waker`s from that executor containing pointers
   to free'd memory, and dereferencing them on `.wake()`.  This unsound behavior
   is no longer possible without `unsafe` code.

## [0.3.0] - 2020-05-06
### Changed
 - `Join` trait now takes `self` instead of `&mut self`, fixes UB
 - Internals of `Select` no longer contain unsafe code.

### Fixed
 - `.join()` allowing for moving pinned futures.

## [0.2.0] - 2020-05-05
### Changed
 - Simplified `select!()` implementation.  This also reduces the amount of bounds
   checking.
 - `Select` trait now requires that `Future`s are `Unpin`, this fixes a bug that
   allowed for pinned futures to be moved between calls to `.select()`.

### Fixed
 - `.select()` allowing for moving pinned futures.

### Contributors
Thanks to everyone who contributed to make this version of pasts possible!

 - [AldaronLau](https://github.com/AldaronLau)
 - [Darksonn](https://github.com/Darksonn)

## [0.1.0] - 2020-05-03
### Added
 - `Join` trait to replace `join!()`
 - `Select` trait to replace `select!()`
 - `DynFut` trait for converting `Future`s into `DynFuture`s.  This lets you put
   your futures into arrays.
 - `prelude` module for traits.

### Removed
 - All macros
 - `Task`

## [0.0.1] - 2019-12-19
### Added
 - `join!()` similar to macro from `futures` crate.
 - `select!()` similar to macro from `futures` crate.
 - `run!()` a macro that builds an asynchronous loop.
 - `task!()` a pinning macro, which unlike `pin-util`'s `pin_mut!()` doesn't
   insert unsafe code.
 - `Task` - an abstraction over a pinned future, that disallows attempting to run
   futures after completion.
 - `ThreadInterrupt` - a condvar-based interrupt (requires std library feature to
   be enabled).
 - `Interrupt` - a safe way to define asynchronous waking in the executor.
 - `spawn_blocking` - like `tokio`'s `spawn_blocking`, creates a future from a
   closure by running it on a dynamically sized thread pool (also requires std
   library feature to be enabled).
