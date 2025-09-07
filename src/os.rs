// Daku
#[cfg_attr(all(target_arch = "wasm32", daku), path = "os/daku.rs")]
// Std
#[cfg_attr(
    all(
        feature = "std",
        not(all(target_arch = "wasm32", any(daku, feature = "web")))
    ),
    path = "os/std.rs"
)]
// Web
#[cfg_attr(all(target_arch = "wasm32", feature = "web"), path = "os/web.rs")]
mod target;

use core::fmt::Debug;

/// Implement `Target for Os` to add platform support for a target.
pub(crate) struct Os;

/// Target platform support
pub(crate) trait Target: Sized {
    type ParkCx: Debug + Default;

    /// Stop doing anything on the CPU or thread until unparked.
    #[inline(always)]
    fn park(self, _park_cx: &Self::ParkCx) {
        // Default implementation doesn't actually park for maximum portability.
        //
        // Hint at spin loop to possibly save CPU time with a short sleep.
        core::hint::spin_loop();
    }

    /// Resume execution on the parked CPU or thread.
    #[inline(always)]
    fn unpark(self, _park_cx: &Self::ParkCx) {}
}
