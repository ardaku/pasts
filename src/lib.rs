#![no_std]
extern crate alloc;

mod waker;

pub use waker::{waker_ref, Woke};
