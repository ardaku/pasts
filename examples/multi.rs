mod timerfuture;

async fn timer(how_long: u64) -> u64 {
    timerfuture::TimerFuture::new(std::time::Duration::new(how_long, 0)).await;
    how_long
}

fn main() {
    let one = timer(1);
    let two = timer(2);

    let mut one_ret = None;
    let mut two_ret = None;

    let ret = pasts::block_on(async {
        for _ in 0..2 { // Push 2 futures to completion.
            pasts::select! {
                a = one => one_ret = Some(a),
                b = two => two_ret = Some(b),
            }
        }

        (one_ret.unwrap(), two_ret.unwrap())
    });
    println!("Future returned: \"{:?}\"", ret);
}
