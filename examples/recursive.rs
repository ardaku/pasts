use pasts::Executor;

fn main() {
    Executor::default().block_on(async {
        Executor::default().block_on(async {
            println!("Hello from the future running on the inner executor!");
        });

        println!("Hello from the future running on the outer executor!");
    });
}
