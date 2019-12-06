mod timerfuture;

fn main() {
    let ret = pasts::block_until(async {
        println!("Waiting 2 seconds…");
        timerfuture::TimerFuture::new(std::time::Duration::new(2, 0)).await;
        println!("Waited 2 seconds.");
        "Complete!"
    }, pasts::ATOMIC_INTERRUPT);
    println!("Future returned: \"{}\"", ret);
}
