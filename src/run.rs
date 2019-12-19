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
///     pasts::run!(context while true; one, two)
/// }
///
/// <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(example());
/// ```
#[macro_export]
macro_rules! run {
    ($context:ident while $exit:expr; $($generator:ident),* $(,)?) => {
        {
            use $crate::_pasts_hide::{new_task, ref_from_ptr};

            let cx: &mut _ = &mut $context;
            let cx: *mut _ = cx;

            $(
                let mut __pasts_future = $generator(ref_from_ptr(cx));
                #[allow(unused_mut)]
                let mut $generator = (
                    new_task(&mut __pasts_future).0,
                    $generator,
                );
            )*

            while $exit {
                $crate::select!(
                    $(
                        _a = $generator.0 => {
                            $generator.0.set(($generator.1)(ref_from_ptr(cx)));
                        }
                    ),*
                );
            }
        }
    };
}
