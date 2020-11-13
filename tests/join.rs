use pasts::prelude::*;

#[test]
fn join6() {
    pasts::spawn(|| async {
        let task = pasts::spawn(|| async {
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
        });
        assert_eq!(task.await, (1, 'a', 4.0, "boi", [4, 6], (2, 'a')));
    });
}
