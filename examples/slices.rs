include!(concat!(env!("OUT_DIR"), "/main.rs"));

use pasts::{prelude::*, Join};

struct Exit;

struct App<'a> {
    tasks: &'a mut [BoxNotifier<'static, &'static str>],
}

impl App<'_> {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        println!("Received message from {id}, completed task: {val}");

        Ready(Exit)
    }

    async fn main(_executor: Executor) {
        let tasks: &mut [BoxNotifier<'_, _>] = &mut [
            Box::pin(async { "Hello" }.fuse()),
            Box::pin(async { "World" }.fuse()),
        ];
        let mut app = App { tasks };

        // First task will complete first.
        Join::new(&mut app).on(|s| s.tasks, App::completion).await;
    }
}
