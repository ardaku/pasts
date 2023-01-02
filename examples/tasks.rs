use pasts::{prelude::*, Join};

struct Exit;

struct App {
    tasks: Vec<BoxNotifier<'static, &'static str>>,
}

impl App {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        println!("Received message from completed task: {val}");

        self.tasks.swap_remove(id);

        if self.tasks.is_empty() {
            Ready(Exit)
        } else {
            Pending
        }
    }
}

#[async_main::async_main(pasts)]
async fn main(_executor: Executor) {
    let mut app = App {
        tasks: vec![
            Box::pin(async { "Hello" }.fuse()),
            Box::pin(async { "World" }.fuse()),
        ],
    };

    Join::new(&mut app)
        .on(|s| &mut s.tasks[..], App::completion)
        .await;
}
