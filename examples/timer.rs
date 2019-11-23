use {
    pasts::{waker_ref, Woke},
    std::{
        future::Future,
        pin::Pin,
        sync::mpsc::{sync_channel, Receiver, SyncSender},
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
        thread,
        time::Duration,
    },
};

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });

        TimerFuture { shared_state }
    }
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

// Example executor, since const generics aren't stable yet - create with macro
// passing in max_capacity.
struct Executor {
    // List of futures.
    // futures: [impl Future<Output = ()> + 'static; 1],
    // 

    // Old thing
    ready_queue: Receiver<Arc<Task>>,
}

#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
    fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = Box::pin(future);
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

struct Task {
    pub future:
        Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
    pub task_sender: SyncSender<Arc<Task>>,
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

impl Woke for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

impl Executor {
    fn run(&self) {
        println!("Starting executor!");
        while let Ok(task) = self.ready_queue.recv() { // Blocking call
            println!("Executor Loop!");
            let mut future_slot = task.future.lock().unwrap();
            println!("Looking for future…");
            if let Some(mut future) = future_slot.take() {
                let waker = waker_ref(&task);
                println!("Got Waker");
                let context = &mut Context::from_waker(&*waker);
                println!("Got Context");
                if let Poll::Pending = future.as_mut().poll(context) {
                    *future_slot = Some(future);
                }
                println!("Poll Finished");
            }
        }
    }
}

/// Pins a value on the stack.
///
/// ```
/// # use pin_utils::pin_mut;
/// # use core::pin::Pin;
/// # struct Foo {}
/// let foo = Foo { /* ... */ };
/// pin_mut!(foo);
/// let _: Pin<&mut Foo> = foo;
/// ```
#[macro_export]
macro_rules! pin_mut {
    ($($x:ident),* $(,)?) => { $(
        // Move the value to ensure that it is owned
        let mut $x = $x;
        // Shadow the original binding so that it can't be directly accessed
        // ever again.
        #[allow(unused_mut)]
        let mut $x = unsafe {
            std::pin::Pin::new_unchecked(&mut $x)
        };
    )* }
}

fn main() {
    // TODO: Eventually generate executor (this function) with macro.
    use std::sync::atomic::{AtomicBool, Ordering};

    static mut FUTURE_ONE_CONDVAR: AtomicBool = AtomicBool::new(true);

    pub struct FutureOneTask();

    impl Woke for FutureOneTask {
        fn wake_by_ref(_arc_self: &Arc<Self>) {
            unsafe {
                FUTURE_ONE_CONDVAR.store(true, Ordering::Relaxed);
            }

            /*let cloned = arc_self.clone();
            arc_self
                .task_sender
                .send(cloned)
                .expect("too many tasks queued");*/
        }
    }

    let task = Arc::new(FutureOneTask());
    let mut future_one = async {
        println!("Waiting 2 seconds…");
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("Done!");
    };
    pin_mut!(future_one);

    // Check for any futures that are ready

    loop {
        if unsafe { FUTURE_ONE_CONDVAR.load(Ordering::Relaxed) } {
            // This runs whenever woke.
            let waker = waker_ref(&task);
            println!("Got Waker");
            let context = &mut Context::from_waker(&*waker);
            println!("Got Context");
            if let Poll::Pending = future_one.as_mut().poll(context) {
                // Go back to "sleep".
                unsafe {
                    FUTURE_ONE_CONDVAR.store(false, Ordering::Relaxed);
                }
            } else {
                println!("Complete!");
                break;
            }
            println!("Poll Finished");
        }
    }

    // // // 

/*    let (executor, spawner) = new_executor_and_spawner();
    spawner.spawn(async {
        println!("howdy!");
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");
    });
    drop(spawner);
    executor.run();*/
}
