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

    pasts::run!(true, context, one, two)
}

fn main() {
    <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(example());
}
