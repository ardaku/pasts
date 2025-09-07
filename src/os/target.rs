//! Unknown target, fake implementation.
//!
//! This can be used as a template when adding new target support.

use super::{Os, Target};

/// Default parking implementation doesn't need any state
#[derive(Debug, Default)]
pub(crate) struct ParkCx;

impl Target for Os {
    type ParkCx = ParkCx;
}
