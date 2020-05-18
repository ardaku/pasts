# Changelog
All notable changes to `pasts` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://jeronlau.tk/semver/).

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
