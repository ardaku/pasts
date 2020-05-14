// Pasts
//
// Copyright (c) 2019-2020 Jeron Aldaron Lau
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0>, or the Zlib License, <LICENSE-ZLIB
// or http://opensource.org/licenses/Zlib>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use core::{
    ptr,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    mem::MaybeUninit,
};

/// Trait for joining a tuple of futures into a single future.
pub trait Join<Z> {
    /// Poll multiple futures concurrently, and return a tuple of returned
    /// values from each future.
    ///
    /// Futures that are ready first will be executed first.  This makes
    /// `(a, b).join().await` faster than the alternative `(a.await, b.await)`.
    ///
    /// ```rust
    /// #![forbid(unsafe_code)]
    ///
    /// use pasts::prelude::*;
    ///
    /// async fn one() -> i32 {
    ///     42
    /// }
    ///
    /// async fn two() -> char {
    ///     'a'
    /// }
    ///
    /// async fn example() {
    ///     // Joined await on the two futures.
    ///     let ret = (one(), two()).join().await;
    ///     assert_eq!(ret, (42, 'a'));
    /// }
    ///
    /// pasts::ThreadInterrupt::block_on(example());
    /// ```
    fn join(self) -> Z;
}

// unsafe: For pinning projections, MaybeUninit return tuple
#[allow(
    unsafe_code,
    missing_debug_implementations,
    missing_copy_implementations
)]
mod tuple {
    use super::*;

    // 0-Tuple
    pub struct Join0();
    impl Future for Join0 {
        type Output = ();
        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
            Poll::Ready(())
        }
    }
    impl Join<Join0> for () {
        fn join(self) -> Join0 {
            Join0()
        }
    }
    // 1-Tuple
    pub struct Join1<T, A: Future<Output = T>>((A,));
    impl<T, A: Future<Output = T>> Future for Join1<T, A> {
        type Output = (T,);
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<(T,)> {
            match unsafe { self.map_unchecked_mut(|s| &mut s.0 .0) }.poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(out) => Poll::Ready((out,)),
            }
        }
    }
    impl<T, A: Future<Output = T>> Join<Join1<T, A>> for (A,) {
        fn join(self) -> Join1<T, A> {
            Join1(self)
        }
    }
    // 2-Tuple
    pub struct Join2<T, A: Future<Output = T>, U, B: Future<Output = U>>(
        (A, B),
        (bool, bool),
        MaybeUninit<(T, U)>,
    );
    impl<T, A, U, B> Future for Join2<T, A, U, B>
    where
        A: Future<Output = T>,
        B: Future<Output = U>,
    {
        type Output = (T, U);
        fn poll(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<(T, U)> {
            let mut complete = true;
            if self.1 .0 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .0) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).0 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .0 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .1 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .1) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).1 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .1 = false }
                } else {
                    complete = false;
                }
            }
            if complete {
                Poll::Ready(unsafe { ptr::read(self.2.as_ptr()) })
            } else {
                Poll::Pending
            }
        }
    }
    impl<T, A, U, B> Join<Join2<T, A, U, B>> for (A, B)
    where
        A: Future<Output = T>,
        B: Future<Output = U>,
    {
        fn join(self) -> Join2<T, A, U, B> {
            Join2(self, (true, true), MaybeUninit::uninit())
        }
    }
    // 3-Tuple
    pub struct Join3<
        T,
        A: Future<Output = T>,
        U,
        B: Future<Output = U>,
        V,
        C: Future<Output = V>,
    >((A, B, C), (bool, bool, bool), MaybeUninit<(T, U, V)>);
    impl<T, A, U, B, V, C> Future for Join3<T, A, U, B, V, C>
    where
        A: Future<Output = T>,
        B: Future<Output = U>,
        C: Future<Output = V>,
    {
        type Output = (T, U, V);
        fn poll(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<(T, U, V)> {
            let mut complete = true;
            if self.1 .0 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .0) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).0 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .0 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .1 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .1) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).1 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .1 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .2 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .2) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).2 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .2 = false }
                } else {
                    complete = false;
                }
            }
            if complete {
                Poll::Ready(unsafe { ptr::read(self.2.as_ptr()) })
            } else {
                Poll::Pending
            }
        }
    }
    impl<T, A, U, B, V, C> Join<Join3<T, A, U, B, V, C>> for (A, B, C)
    where
        A: Future<Output = T>,
        B: Future<Output = U>,
        C: Future<Output = V>,
    {
        fn join(self) -> Join3<T, A, U, B, V, C> {
            Join3(self, (true, true, true), MaybeUninit::uninit())
        }
    }
    // 4-Tuple
    pub struct Join4<
        T,
        A: Future<Output = T>,
        U,
        B: Future<Output = U>,
        V,
        C: Future<Output = V>,
        W,
        D: Future<Output = W>,
    >(
        (A, B, C, D),
        (bool, bool, bool, bool),
        MaybeUninit<(T, U, V, W)>,
    );
    impl<T, A, U, B, V, C, W, D> Future for Join4<T, A, U, B, V, C, W, D>
    where
        A: Future<Output = T>,
        B: Future<Output = U>,
        C: Future<Output = V>,
        D: Future<Output = W>,
    {
        type Output = (T, U, V, W);
        fn poll(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<(T, U, V, W)> {
            let mut complete = true;
            if self.1 .0 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .0) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).0 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .0 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .1 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .1) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).1 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .1 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .2 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .2) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).2 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .2 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .3 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .3) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).3 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .3 = false }
                } else {
                    complete = false;
                }
            }
            if complete {
                Poll::Ready(unsafe { ptr::read(self.2.as_ptr()) })
            } else {
                Poll::Pending
            }
        }
    }
    impl<T, A, U, B, V, C, W, D> Join<Join4<T, A, U, B, V, C, W, D>>
        for (A, B, C, D)
    where
        A: Future<Output = T>,
        B: Future<Output = U>,
        C: Future<Output = V>,
        D: Future<Output = W>,
    {
        fn join(self) -> Join4<T, A, U, B, V, C, W, D> {
            Join4(self, (true, true, true, true), MaybeUninit::uninit())
        }
    }
    // 5-Tuple
    pub struct Join5<
        T,
        A: Future<Output = T>,
        U,
        B: Future<Output = U>,
        V,
        C: Future<Output = V>,
        W,
        D: Future<Output = W>,
        X,
        E: Future<Output = X>,
    >(
        (A, B, C, D, E),
        (bool, bool, bool, bool, bool),
        MaybeUninit<(T, U, V, W, X)>,
    );
    impl<T, A, U, B, V, C, W, D, X, E> Future
        for Join5<T, A, U, B, V, C, W, D, X, E>
    where
        A: Future<Output = T>,
        B: Future<Output = U>,
        C: Future<Output = V>,
        D: Future<Output = W>,
        E: Future<Output = X>,
    {
        type Output = (T, U, V, W, X);
        fn poll(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<(T, U, V, W, X)> {
            let mut complete = true;
            if self.1 .0 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .0) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).0 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .0 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .1 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .1) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).1 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .1 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .2 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .2) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).2 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .2 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .3 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .3) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).3 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .3 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .4 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .4) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).4 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .4 = false }
                } else {
                    complete = false;
                }
            }
            if complete {
                Poll::Ready(unsafe { ptr::read(self.2.as_ptr()) })
            } else {
                Poll::Pending
            }
        }
    }
    impl<T, A, U, B, V, C, W, D, X, E> Join<Join5<T, A, U, B, V, C, W, D, X, E>>
        for (A, B, C, D, E)
    where
        A: Future<Output = T>,
        B: Future<Output = U>,
        C: Future<Output = V>,
        D: Future<Output = W>,
        E: Future<Output = X>,
    {
        fn join(self) -> Join5<T, A, U, B, V, C, W, D, X, E> {
            Join5(self, (true, true, true, true, true), MaybeUninit::uninit())
        }
    }
    // 6-Tuple
    pub struct Join6<
        T,
        A: Future<Output = T>,
        U,
        B: Future<Output = U>,
        V,
        C: Future<Output = V>,
        W,
        D: Future<Output = W>,
        X,
        E: Future<Output = X>,
        Y,
        F: Future<Output = Y>,
    >(
        (A, B, C, D, E, F),
        (bool, bool, bool, bool, bool, bool),
        MaybeUninit<(T, U, V, W, X, Y)>,
    );
    impl<T, A, U, B, V, C, W, D, X, E, Y, F> Future
        for Join6<T, A, U, B, V, C, W, D, X, E, Y, F>
    where
        A: Future<Output = T>,
        B: Future<Output = U>,
        C: Future<Output = V>,
        D: Future<Output = W>,
        E: Future<Output = X>,
        F: Future<Output = Y>,
    {
        type Output = (T, U, V, W, X, Y);
        fn poll(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<(T, U, V, W, X, Y)> {
            let mut complete = true;
            if self.1 .0 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .0) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).0 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .0 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .1 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .1) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).1 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .1 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .2 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .2) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).2 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .2 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .3 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .3) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).3 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .3 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .4 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .4) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).4 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .4 = false }
                } else {
                    complete = false;
                }
            }
            if self.1 .5 {
                let f =
                    unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.0 .5) };
                if let Poll::Ready(out) = f.poll(cx) {
                    unsafe {
                        (*self.as_mut().get_unchecked_mut().2.as_mut_ptr()).5 =
                            out
                    }
                    unsafe { self.as_mut().get_unchecked_mut().1 .5 = false }
                } else {
                    complete = false;
                }
            }
            if complete {
                Poll::Ready(unsafe { ptr::read(self.2.as_ptr()) })
            } else {
                Poll::Pending
            }
        }
    }
    impl<T, A, U, B, V, C, W, D, X, E, Y, F>
        Join<Join6<T, A, U, B, V, C, W, D, X, E, Y, F>> for (A, B, C, D, E, F)
    where
        A: Future<Output = T>,
        B: Future<Output = U>,
        C: Future<Output = V>,
        D: Future<Output = W>,
        E: Future<Output = X>,
        F: Future<Output = Y>,
    {
        fn join(self) -> Join6<T, A, U, B, V, C, W, D, X, E, Y, F> {
            Join6(
                self,
                (true, true, true, true, true, true),
                MaybeUninit::uninit(),
            )
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn join6() {
        let future = async {
            (
                async { 1i32 },
                async { 'a' },
                async { 4.0f32 },
                async { "boi" },
                async { [4i32, 6i32] },
                async { (2i32, 'a') },
            )
                .join()
                .await
        };

        assert_eq!(
            crate::ThreadInterrupt::block_on(future),
            (1, 'a', 4.0, "boi", [4, 6], (2, 'a'))
        );
    }
}
