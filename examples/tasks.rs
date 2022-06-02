use pasts::{prelude::*, BoxTask, Join, Task};

struct Exit;

struct State {
    tasks: Vec<BoxTask<'static, &'static str>>,
}

impl State {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        println!("Received message from completed task: {val}");

        self.tasks.swap_remove(id);

        if self.tasks.is_empty() { Ready(Exit) } else { Pending }
    }
}

async fn run() {
    let mut state = State {
        tasks: vec![
            Task::new(async { "Hello" }).into(),
            Task::new(async { "World" }).into(),
        ],
    };

    Join::new(&mut state).on(|s| &mut s.tasks[..], State::completion).await;
}

fn main() {
    pasts::block_on(run());
}
