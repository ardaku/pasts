mod timerfuture;

#[derive(Debug)]
struct Length(u64);

impl Drop for Length {
    fn drop(&mut self) {
        println!("Dropping {}", self.0);
    }
}

async fn timer(how_long: u64) -> Length {
    timerfuture::TimerFuture::new(std::time::Duration::new(how_long, 0)).await;
    Length(how_long)
}

fn main() {
    let one = timer(1);
    let two = timer(2);

    let ret = <pasts::CondvarInterrupt as pasts::Interrupt>::block_on(async {
        // This will only take two seconds, rather than `(one.await, two.await)`
        // which will take three.
        pasts::join!(one, two)
    });
    println!("Future returned: \"{:?}\"", ret);
}
