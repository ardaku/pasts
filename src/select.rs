// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::{
    future::Future, marker::PhantomData, pin::Pin, task::Context, task::Poll,
};

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use crate::DynFut;

#[allow(missing_debug_implementations)]
pub struct SelectFuture<'b, T, I: DynFut<T>>(&'b mut [I], PhantomData<T>);
#[allow(missing_debug_implementations)]
pub struct SelectOptionalFuture<'b, T, I: DynFut<T>>(
    &'b mut [Option<I>],
    PhantomData<T>,
);
#[allow(missing_debug_implementations)]
pub struct SelectBoxedFuture<'b, T>(&'b mut [Pin<Box<dyn Future<Output = T>>>]);

impl<T, I> Future for SelectFuture<'_, T, I>
where
    I: DynFut<T>,
    T: Unpin,
{
    type Output = (usize, T);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let tasks = &mut self.get_mut().0;

        for (task_id, task) in tasks.iter_mut().enumerate() {
            let mut task = task.fut();
            let pin_fut = Pin::new(&mut task);
            let task = pin_fut.poll(cx);
            match task {
                Poll::Ready(ret) => return Poll::Ready((task_id, ret)),
                Poll::Pending => {}
            }
        }

        Poll::Pending
    }
}

impl<T, I> Future for SelectOptionalFuture<'_, T, I>
where
    I: DynFut<T>,
    T: Unpin,
{
    type Output = (usize, T);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let tasks = &mut self.get_mut().0;

        for (task_id, task_opt) in tasks.iter_mut().enumerate() {
            if let Some(ref mut task) = task_opt {
                let mut task = task.fut();
                let pin_fut = Pin::new(&mut task);
                let task = pin_fut.poll(cx);
                match task {
                    Poll::Ready(ret) => {
                        *task_opt = None;
                        return Poll::Ready((task_id, ret));
                    }
                    Poll::Pending => {}
                }
            }
        }

        Poll::Pending
    }
}

impl<T> Future for SelectBoxedFuture<'_, T>
where
    T: Unpin,
{
    type Output = (usize, T);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let tasks = &mut self.get_mut().0;

        for (task_id, task) in tasks.iter_mut().enumerate() {
            let task = Pin::new(task).poll(cx);
            match task {
                Poll::Ready(ret) => return Poll::Ready((task_id, ret)),
                Poll::Pending => {}
            }
        }

        Poll::Pending
    }
}

/// A trait to select on a slice of `Future`s.
///
/// # Select on slice of futures.
///
/// ```rust
/// use pasts::prelude::*;
///
/// async fn async_main() {
///     let mut hello = async { "Hello" };
///     let mut world = async { "World!" };
///     // Hello is ready, so returns with index and result.
///     assert_eq!((0, "Hello"), [hello.fut(), world.fut()].select().await);
/// }
///
/// pasts::spawn(async_main);
/// ```
// Future needs to be unpin to prevent UB because `Future`s can move between
// calls to select after starting (which fills future's RAM with garbage data).
pub trait Select<'a, T, I>
where
    I: DynFut<T>,
{
    /// Poll multiple futures, and return the value from the future that returns
    /// `Ready` first.
    fn select(&'a mut self) -> SelectFuture<'a, T, I>;
}

/// A trait to select on a slice of `Option<Future>`s.
pub trait SelectOptional<'a, T, I>
where
    I: DynFut<T>,
{
    /// Poll multiple futures, and return the value from the future that returns
    /// `Ready` first.
    fn select(&'a mut self) -> SelectOptionalFuture<'a, T, I>;
}

/// A trait to select on a slice of `Pin<Box<Future>>`s.
pub trait SelectBoxed<'a, T> {
    /// Poll multiple futures, and return the value from the future that returns
    /// `Ready` first.
    fn select_boxed(&'a mut self) -> SelectBoxedFuture<'a, T>;
}

impl<'a, T, I> Select<'a, T, I> for [I]
where
    I: DynFut<T>,
{
    fn select(&'a mut self) -> SelectFuture<'a, T, I> {
        SelectFuture(self, PhantomData)
    }
}

impl<'a, T, I> SelectOptional<'a, T, I> for [Option<I>]
where
    I: DynFut<T>,
{
    fn select(&'a mut self) -> SelectOptionalFuture<'a, T, I> {
        SelectOptionalFuture(self, PhantomData)
    }
}

impl<'a, T> SelectBoxed<'a, T> for [Pin<Box<dyn Future<Output = T>>>] {
    fn select_boxed(&'a mut self) -> SelectBoxedFuture<'a, T> {
        SelectBoxedFuture(self)
    }
}
