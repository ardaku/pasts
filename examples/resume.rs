use pasts::Executor;

fn main() {
    let executor = Executor::default();

    executor.clone().block_on(async move {
        println!("Hello from a future!");

        executor.block_on(async {
            println!("Resuming execution from within the executor context!");
        });
    });
}
