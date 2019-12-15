/*use core::{
    pin::Pin,
    future::Future,
    task::{Poll, Context},
};

use pasts::prelude::*;

type AsyncFn = async fn() -> AsyncFn;

#[derive(Debug, PartialEq)]
enum Select {
    One(i32),
    Two(char),
}

pub struct AlwaysPending();

impl Future for AlwaysPending {
    type Output = i32;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<i32> {
        Poll::Pending
    }
}

async fn always_pending() -> AsyncFn {
    AlwaysPending().await

    always_pending
}

async fn every_second() -> AsyncFn {
    // TODO

    every_second
}

async fn example() -> Select {
    pasts::tasks! {
        a_fut = AlwaysPending();
        b_fut = two();
    };

    let mut a_fut = Wait(a_fut);
    let mut b_fut = Wait(b_fut);

    let ret = pasts::select!(
        a = a_fut => {
            println!("This will never print!");

            Select::One(a)
        }
        b = b_fut => {
            Select::Two(b)
        }
    );

    assert!(a_fut.is_wait());
    assert!(b_fut.is_done());

    ret
}*/

fn main() {
    /*    assert_eq!(
        pasts::CondvarInterrupt::block_on(example()),
        Select::Two('c')
    );*/
}
