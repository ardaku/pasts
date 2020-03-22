#![forbid(unsafe_code)]

use std::cell::Cell;

async fn timer_future(duration: std::time::Duration) {
    pasts::spawn_blocking(move || std::thread::sleep(duration)).await
}

async fn one(context: &mut usize) {
    timer_future(std::time::Duration::new(1, 0)).await;
    println!("One {}", *context);
    *context += 1;
}

async fn two(context: &mut usize) {
    timer_future(std::time::Duration::new(2, 0)).await;
    println!("Two {}", *context);
    *context += 1;
}

async fn example() {
    /*let mut state: usize = 0;

//    pasts::run!(state < 10; one(&mut state), two(&mut state))

    let mut one2 = one(pasts::_pasts_hide::ref_from_ptr(&mut state));
    let mut two2 = two(pasts::_pasts_hide::ref_from_ptr(&mut state));

    pasts::task_queue!(task_queue = [one2, two2]);

    pasts::pin_mut!(one2);
    pasts::pin_mut!(two2);

    while state < 10 {
        let (i, r): (usize, ()) = task_queue.select().await;
        if i == 0 {
            one2.set(one(pasts::_pasts_hide::ref_from_ptr(&mut state)));
//            task_queue.replace(i, one2.as_mut());
        } else if i == 1 {
            two2.set(two(pasts::_pasts_hide::ref_from_ptr(&mut state)));
//            task_queue.replace(i, two2.as_mut());
        }
    }*/
}

fn main() {
    <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(example());
}
