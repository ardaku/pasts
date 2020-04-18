# Pasts

#### Minimal and simpler alternative to the futures crate.

[![Build Status](https://api.travis-ci.org/AldaronLau/pasts.svg?branch=master)](https://travis-ci.org/AldaronLau/pasts)
[![Docs](https://docs.rs/pasts/badge.svg)](https://docs.rs/pasts)
[![crates.io](https://img.shields.io/crates/v/pasts.svg)](https://crates.io/crates/pasts)

### Goals/Features
- No required std
- No allocations
- No procedural macros (for faster compile times)
- No dependencies
- No cost (True zero-cost abstractions!)
- No pain (API super easy to learn & use!)
- No unsafe code in pinning macro (allowing you to `forbid(unsafe_code)`)

## Table of Contents
- [Getting Started](#getting-started)
   - [Example](#example)
   - [API](#api)
   - [Features](#features)
- [Upgrade](#upgrade)
- [License](#license)
   - [Contribution](#contribution)


## Getting Started
Add the following to your `Cargo.toml`.

```toml
[dependencies]
pasts = "0.1"
```

### Example
This example goes in a loop and prints "One" every second, and "Two" every other
second.  After 10 prints, the program terminates.

```rust,no_run
#![forbid(unsafe_code)]

use pasts::prelude::*;
use pasts::ThreadInterrupt;

use std::cell::RefCell;

async fn timer_future(duration: std::time::Duration) {
    pasts::spawn_blocking(move || std::thread::sleep(duration)).await
}

async fn one(state: &RefCell<usize>) {
    println!("Starting task one");
    while *state.borrow() < 5 {
        timer_future(std::time::Duration::new(1, 0)).await;
        let mut state = state.borrow_mut();
        println!("One {}", *state);
        *state += 1;
    }
    println!("Finish task one");
}

async fn two(state: &RefCell<usize>) {
    println!("Starting task two");
    loop {
        timer_future(std::time::Duration::new(2, 0)).await;
        let mut state = state.borrow_mut();
        println!("Two {}", *state);
        *state += 1;
    }
}

async fn example() {
    let state = RefCell::new(0);
    let mut task_one = one(&state);
    let mut task_two = two(&state);
    let mut tasks = [
        task_one.dyn_fut(),
        task_two.dyn_fut(),
    ];
    tasks.select().await;
}

fn main() {
    ThreadInterrupt::block_on(example());
}
```

:x
API documentation can be found on [docs.rs](https://docs.rs/pasts).

### Features
Some APIs are only available with the **std** feature enabled.  They are labeled
as such on [docs.rs](https://docs.rs/pasts).

## Upgrade
You can use the
[changelog](https://github.com/AldaronLau/pasts/blob/master/CHANGELOG.md)
to facilitate upgrading this crate as a dependency.

## License
Licensed under either of
 - Apache License, Version 2.0,
   ([LICENSE-APACHE](https://github.com/AldaronLau/pasts/blob/master/LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 - Zlib License,
   ([LICENSE-ZLIB](https://github.com/AldaronLau/pasts/blob/master/LICENSE-ZLIB) or
   https://opensource.org/licenses/Zlib)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

Before contributing, check out the
[contribution guidelines](https://github.com/AldaronLau/pasts/blob/master/CONTRIBUTING.md),
and, as always, make sure to always follow the
[code of conduct](https://github.com/AldaronLau/pasts/blob/master/CODE_OF_CONDUCT.md).
