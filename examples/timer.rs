mod timerfuture;

fn main() {
    let ret = pasts::block_on::<_, pasts::CondvarInterrupt>(
        async {
            println!("Waiting 2 seconds…");
            timerfuture::TimerFuture::new(std::time::Duration::new(2, 0)).await;
            println!("Waited 2 seconds.");
            "Complete!"
        },
    );
    println!("Future returned: \"{}\"", ret);
}
