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
            };

            let_pin! {
                $(
                    $future = Some($future);
                )*
            };
        }
    };
}

fn main() {
    let ret = pasts::block_on(async {
        pasts::let_pin! {
            one = timer(1);
            two = timer(2);
        };

        let mut one = pasts::Wait(one);
        let mut two = pasts::Wait(two);

        for _ in 0..2 { // Push 2 futures to completion.
            pasts::select! {
                a = one => println!("Finished 1"),
                b = two => println!("Finished 2"),
            }
        }

        (
            match one {
                pasts::Done(r) => r,
                pasts::Wait(_) => unreachable!(),
            },
            match two {
                pasts::Done(r) => r,
                pasts::Wait(_) => unreachable!(),
            },
        )
    });
    println!("Future returned: \"{:?}\"", ret);
}
