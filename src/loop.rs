use crate::{prelude::*, Notify};

pub trait Stateful<S, T>: Unpin {
    fn state(&mut self) -> &mut S;

    fn poll(&mut self, _: &mut Task<'_>) -> Poll<Poll<T>> {
        Pending
    }
}

#[derive(Debug)]
pub struct Never<'a, S>(&'a mut S);

impl<S, T> Stateful<S, T> for Never<'_, S> {
    fn state(&mut self) -> &mut S {
        self.0
    }
}

/// Composable asynchronous event loop.
///
/// # Selecting on Futures:
/// Select first completed future.
///
/// ```rust
#[doc = include_str!("../examples/slices.rs")]
/// ```
/// 
/// # Task spawning
/// Spawns tasks in a [`Vec`](alloc::vec::Vec), and removes them as they complete.
/// ```rust
#[doc = include_str!("../examples/tasks.rs")]
/// ```
#[derive(Debug)]
pub struct Loop<S: Unpin, T, F: Stateful<S, T>> {
    other: F,
    _phantom: core::marker::PhantomData<(S, T)>,
}

impl<'a, S: Unpin, T> Loop<S, T, Never<'a, S>> {
    /// Create an empty event loop.
    pub fn new(state: &'a mut S) -> Self {
        let other = Never(state);
        let _phantom = core::marker::PhantomData;

        Loop { other, _phantom }
    }
}

impl<S: Unpin, T, F: Stateful<S, T>> Loop<S, T, F> {
    /// Register an event handler.
    pub fn on<N: Notify + Unpin + ?Sized>(
        self,
        noti: impl for<'a> FnMut(&'a mut S) -> &'a mut N + Unpin,
        then: fn(&mut S, N::Event) -> Poll<T>,
    ) -> Loop<S, T, impl Stateful<S, T>> {
        let other = self.other;
        let _phantom = core::marker::PhantomData;
        let other = Looper { other, noti, then };

        Loop { other, _phantom }
    }
}

impl<S: Unpin, T: Unpin, F: Stateful<S, T>> Future for Loop<S, T, F> {
    type Output = T;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, t: &mut Task<'_>) -> Poll<T> {
        while let Ready(output) = Pin::new(&mut self.other).poll(t) {
            if let Ready(output) = output {
                return Ready(output);
            }
        }

        Pending
    }
}

struct Looper<S, T, E, F: Stateful<S, T>, P> {
    other: F,
    noti: P,
    then: fn(&mut S, E) -> Poll<T>,
}

impl<S, T, E, F, N, P> Stateful<S, T> for Looper<S, T, E, F, P>
where
    F: Stateful<S, T>,
    N: Notify<Event = E> + Unpin + ?Sized,
    P: for<'a> FnMut(&'a mut S) -> &'a mut N + Unpin,
{
    #[inline]
    fn state(&mut self) -> &mut S {
        self.other.state()
    }

    #[inline]
    fn poll(&mut self, t: &mut Task<'_>) -> Poll<Poll<T>> {
        let state = self.other.state();
        let poll = Pin::new((self.noti)(state)).poll_next(t);

        if let Ready(out) = poll.map(|x| (self.then)(state, x)) {
            Ready(out)
        } else {
            self.other.poll(t)
        }
    }
}
