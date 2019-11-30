mod timerfuture;

async fn timer(how_long: u64) -> u64 {
    timerfuture::TimerFuture::new(std::time::Duration::new(how_long, 0)).await;
    how_long
}

/*
macro_rules! join {
    ($($future:ident),* $(,)?) => {
        {
            use pasts::{
                let_pin,
                _pasts_hide::stn::mem::MaybeUninit,
            };

            let_pin! {
                $(
                    $future = Some($future);
                )
            }
        }
    };
}*/

fn main() {
    pasts::let_pin! {
        one = pasts::Task::Wait(timer(1));
        two = pasts::Task::Wait(timer(2));
    };

    let ret = pasts::block_on(async {
        for _ in 0..2 { // Push 2 futures to completion.
            pasts::select! {
                a = one => println!("Finished 1"),
                b = two => println!("Finished 2"),
            }
        }

        (
            one.unwrap(),
            two.unwrap(),
//            pasts::_pasts_hide::assume_init(one),
//            pasts::_pasts_hide::assume_init(two),
        )
    });
    println!("Future returned: \"{:?}\"", ret);
}
