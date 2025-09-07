use super::{Os, Target};

#[derive(Debug, Default)]
pub(crate) struct ParkCx;

impl Target for Os {
    type ParkCx = ParkCx;

    #[inline(always)]
    fn park(self, _park_cx: &Self::ParkCx) {
        // Spin loop hints aren't useful on the web since nothing blocks, so do
        // nothing.
    }
}
