# Pasts
Minimal and simpler alternative to the futures crate.

- No required std
- No allocations
- No procedural macros (for faster compile times)
- No dependencies
- No cost (True zero-cost abstractions!)
- No pain (API super easy to learn & use!)
- No unsafe code in pinning macro (allowing you to `forbid(unsafe_code)`)

## Example
This example goes in a loop and prints "One" every second, and "Two" every other
second.  After 10 prints, the program terminates.

```rust
#![forbid(unsafe_code)]

async fn timer_future(duration: std::time::Duration) {
    pasts::spawn_blocking(move || std::thread::sleep(duration)).await
}

async fn one(context: &mut usize) {
    timer_future(std::time::Duration::new(1, 0)).await;
    println!("One {}", *context);
    *context += 1;
}

async fn two(context: &mut usize) {
    timer_future(std::time::Duration::new(2, 0)).await;
    println!("Two {}", *context);
    *context += 1;
}

async fn example() {
    let mut context: usize = 0;

    pasts::run!(context while context < 10; one, two)
}

fn main() {
    <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(example());
}
```
