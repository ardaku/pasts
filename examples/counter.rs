use core::time::Duration;

use async_main::{async_main, LocalSpawner};
use async_std::task::sleep;
use pasts::{notify, prelude::*};

// Shared state between tasks on the thread.
struct App {
    counter: usize,
}

impl App {
    fn one(&mut self, (): ()) -> bool {
        println!("One {}", self.counter);
        self.counter += 1;

        if self.counter > 6 {
            false
        } else {
            true
        }
    }

    fn two(&mut self, (): ()) -> bool {
        println!("Two {}", self.counter);
        self.counter += 1;

        true
    }
}

#[async_main]
async fn main(_spawner: LocalSpawner) {
    let sleep = |seconds| sleep(Duration::from_secs_f64(seconds));
    let ref mut one = notify::future_fn(|| Box::pin(sleep(1.0)));
    let ref mut two = notify::future_fn(|| Box::pin(sleep(2.0)));
    let ref mut app = App { counter: 0 };

    while notify::select([
        &mut one.map(|()| App::one as fn(&mut App, ()) -> bool),
        &mut two.map(|()| App::two as _),
    ])
    .next()
    .await(app, ())
    {}
}
