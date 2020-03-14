/// Poll multiple futures concurrently in a loop.  At completion of each future,
/// the future is regenerated.
///
/// This macro is only usable inside async functions and blocks.
///
/// The following example prints "One" every second, and "Two" every other
/// second.
///
/// ```rust,no_run
/// #![forbid(unsafe_code)]
///
/// async fn timer_future(duration: std::time::Duration) {
///     pasts::spawn_blocking(move || std::thread::sleep(duration)).await
/// }
///
/// async fn one(context: &mut usize) {
///     timer_future(std::time::Duration::new(1, 0)).await;
///     println!("One {}", *context);
///     *context += 1;
/// }
///
/// async fn two(context: &mut usize) {
///     timer_future(std::time::Duration::new(2, 0)).await;
///     println!("Two {}", *context);
///     *context += 1;
/// }
///
/// async fn example() {
///     let mut context: usize = 0;
///
///     pasts::run!(context while context < 10; one, two)
/// }
///
/// <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(example());
/// ```
#[macro_export]
macro_rules! run {
    ($context:ident while $exit:expr; $($generator:ident),* $(,)?) => {
        {
/*            $crate::task_queue!(task_queue = [$({
                let ret: &mut dyn FnMut(_) -> _ = &mut $generator;
                (ret)($context)
            }),*]);

            let re_gen = [$({
                let ret: &mut dyn FnMut(_) -> _ = &mut $generator;
                ret
            }),*];

            while $exit {
                let (i, r) = task_queue.select().await;
                task_queue.replace(i, futures[i]);
            }*/

            use $crate::_pasts_hide::{
                new_task, ref_from_ptr,
                stn::{
                    future::Future,
                    pin::Pin,
                    task::{Poll, Context},
                }
            };

            let state: &mut _ = &mut $context;
            let state: *mut _ = state;

            $(
                let mut __pasts_future = $generator(ref_from_ptr(state));
                #[allow(unused_mut)]
                let mut $generator = (
                    new_task(&mut __pasts_future).0,
                    $generator,
                );
            )*

            while $exit {
                struct __Pasts_Selector<'a> {
                    closure: &'a mut dyn FnMut(&mut Context<'_>) -> Poll<()>,
                }
                impl<'a> Future for __Pasts_Selector<'a> {
                    type Output = ();
                    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                        (self.get_mut().closure)(cx)
                    }
                }
                __Pasts_Selector { closure: &mut |__pasts_cx: &mut Context<'_>| {
                    $(
                        match $generator.0.poll(__pasts_cx) {
                            Poll::Ready(_) => {
                                $generator.0.set(($generator.1)(ref_from_ptr(state)));
                                return Poll::Ready(());
                            }
                            Poll::Pending => {}
                        }
                    )*
                    Poll::Pending
                } }.await
            }
        }
    };
}
