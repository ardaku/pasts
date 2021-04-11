use core::task::Poll;
use pasts::{EventLoop, Loop, Task};

type Exit = ();

struct State {
    tasks: Vec<Task<&'static str>>,
}

impl State {
    fn completion(&mut self, (id, val): (usize, &str)) -> Poll<Exit> {
        self.tasks.remove(id);
        println!("Received message from completed task: {}", val);
        if self.tasks.is_empty() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }

    fn event_loop(&mut self, elts: EventLoop<Self, Exit>) -> impl Loop<Exit> {
        elts.poll(&mut self.tasks, Self::completion)
    }
}

async fn run() {
    let mut tasks = State {
        tasks: vec![Box::pin(async { "Hello" }), Box::pin(async { "World" })],
    };

    EventLoop::run(&mut tasks, State::event_loop).await;
}

fn main() {
    pasts::block_on(run())
}
