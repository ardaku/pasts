/// Poll multiple futures concurrently in a loop.  At completion of each future,
/// the future is regenerated.  This can be used as a simple scheduler / event
/// loop for tasks.
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
///     pasts::tasks!(context while context < 10; [one, two])
/// }
///
/// <pasts::ThreadInterrupt as pasts::Interrupt>::block_on(example());
/// ```
#[macro_export]
macro_rules! tasks {
    ($cx:ident while $exit:expr; [ $($gen:ident),* $(,)? ] $(,)?) => {{
        // Create 2 copies of mutable references to futures.
        $(
            let a = &mut $gen($crate::_pasts_hide::ref_from_ptr(&mut $cx));
            let b = $crate::_pasts_hide::ref_from_ptr(a);
            let $gen = (a, b, $gen);
        )*
        // Create generically-typed futures array using first copy.
        $(
            let temp: core::pin::Pin<&mut dyn core::future::Future<Output = _>> = $crate::_pasts_hide::new_pin($gen.0);
            let mut $gen = (temp, $gen.1, $gen.2);
        )*
        let mut tasks_count = 0;
        let mut tasks = [
            $(
                {
                    let temp = &mut $gen.0;
                    tasks_count += 1;
                    temp
                }
            ),*
        ];
        // Create uniquely-typed futures using second copy.
        $(
            let mut $gen = ($crate::_pasts_hide::new_pin($gen.1), $gen.2);
        )*

        while $exit {
            use $crate::Select;

            let (i, ()): (usize, ()) = tasks.select().await;

            tasks_count = 0;
            $({
                if i == tasks_count {
                    $gen.0.set(($gen.1)(pasts::_pasts_hide::ref_from_ptr(&mut $cx)));
                }
                tasks_count += 1;
            })*
        }
    }};

    ($cx:ident; [ $($gen:ident),* $(,)? ] $(,)?) => {{
        tasks!(true, $($generator),*)
    }};
}
