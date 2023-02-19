use core::time::Duration;

use async_main::{async_main, LocalSpawner};
use async_std::task::sleep;
use pasts::{prelude::*, Join};

// Exit type for App.
struct Exit;

// Shared state between tasks on the thread.
struct App<'a> {
    counter: usize,
    one: &'a mut (dyn Notify<Event = ()> + Unpin),
    two: &'a mut (dyn Notify<Event = ()> + Unpin),
}

impl App<'_> {
    fn one(&mut self, (): ()) -> Poll<Exit> {
        println!("One {}", self.counter);
        self.counter += 1;

        if self.counter > 6 {
            Ready(Exit)
        } else {
            Pending
        }
    }

    fn two(&mut self, (): ()) -> Poll<Exit> {
        println!("Two {}", self.counter);
        self.counter += 1;

        Pending
    }
}

#[async_main]
async fn main(_spawner: LocalSpawner) {
    let sleep = |seconds| sleep(Duration::from_secs_f64(seconds));
    let mut app = App {
        counter: 0,
        one: &mut pasts::future_fn(|| Box::pin(sleep(1.0))),
        two: &mut pasts::future_fn(|| Box::pin(sleep(2.0))),
    };

    Join::new(&mut app)
        .on(|s| s.one, App::one)
        .on(|s| s.two, App::two)
        .await;
}
