use core::time::Duration;

use async_main::{async_main, LocalSpawner};
use async_std::task::sleep;
use pasts::{notify, prelude::*, Loop};

/// Shared state between tasks on the thread.
struct App<'a> {
    counter: usize,
    one: &'a mut (dyn Notify<Event = ()> + Unpin),
    two: &'a mut (dyn Notify<Event = ()> + Unpin),
}

impl App<'_> {
    fn one(&mut self, (): ()) -> Poll {
        println!("One {}", self.counter);
        self.counter += 1;

        if self.counter > 6 {
            Ready(())
        } else {
            Pending
        }
    }

    fn two(&mut self, (): ()) -> Poll {
        println!("Two {}", self.counter);
        self.counter += 1;

        Pending
    }

    async fn run(&mut self) {
        Loop::new(self)
            .on(|s| &mut s.one, Self::one)
            .on(|s| &mut s.two, Self::two)
            .await;
    }
}

#[async_main]
async fn main(_spawner: LocalSpawner) {
    let sleep = |seconds| sleep(Duration::from_secs_f64(seconds));
    let mut app = App {
        counter: 0,
        one: &mut notify::future_fn(|| Box::pin(sleep(1.0))),
        two: &mut notify::future_fn(|| Box::pin(sleep(2.0))),
    };

    app.run().await;
}
