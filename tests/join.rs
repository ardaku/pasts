use pasts::prelude::*;

#[test]
fn join6() {
    static EXECUTOR: pasts::CvarExec = pasts::CvarExec::new();
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
        EXECUTOR.block_on(future),
        (1, 'a', 4.0, "boi", [4, 6], (2, 'a'))
    );
}
