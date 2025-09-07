use super::{Os, Target};
use crate::{LocalBoxFuture, Pool};

#[derive(Debug, Default)]
pub(crate) struct ParkCx;

impl Target for Os {
    type ParkCx = ParkCx;

    #[inline(always)]
    fn park(self, _park_cx: &Self::ParkCx) {
        // Spin loop hints aren't useful on the web since nothing blocks, so do
        // nothing instead.
    }

    /// Spawn a local future.
    #[inline(always)]
    fn spawn<P: Pool>(self, _pool: &P, f: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(f);
    }

    /// Spawn a local future.
    #[inline(always)]
    fn spawn_boxed<P: Pool>(self, pool: &P, f: LocalBoxFuture<'static>) {
        self.spawn(pool, f);
    }

    #[inline(always)]
    fn block_on<P: Pool>(
        self,
        pool: &P,
        f: impl Future<Output = ()> + 'static,
    ) {
        self.spawn(pool, f);
    }
}
