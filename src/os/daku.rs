use super::{Os, Target};

#[derive(Debug, Default)]
pub(crate) struct ParkCx;

impl Target for Os {
    type ParkCx = ParkCx;
}
