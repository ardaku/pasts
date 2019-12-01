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

macro_rules! join {
    ($($future:ident),* $(,)?) => {
        {
            use pasts::{
                let_pin,
                _pasts_hide::stn::mem::MaybeUninit,
                Task::Wait,
            };

            let_pin! {
                $(
                    $future = Wait($future);
                )*
            };
        }
    };
}

fn main() {
    let ret = pasts::block_on(async {
        pasts::let_pin! {
            one = pasts::Task::Wait(timer(1));
            two = pasts::Task::Wait(timer(2));
        };

        for _ in 0..2 { // Push 2 futures to completion.
            pasts::select! {
                a = one => println!("Finished 1"),
                b = two => println!("Finished 2"),
            }
        }

        (
            one.take().unwrap(),
            two.take().unwrap(),
        )
    });
    println!("Future returned: \"{:?}\"", ret);
}
