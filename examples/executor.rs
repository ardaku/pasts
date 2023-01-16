use pasts::Executor;

fn main() {
    Executor::default().block_on(async {
        println!("Hello from a future!");
    });
}
