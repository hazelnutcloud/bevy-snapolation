#![feature(div_duration)]
pub mod snapshot_interpolation;
pub mod vault;

pub mod prelude {
    use super::*;
    pub use snapshot_interpolation::SnapshotInterpolation;
    pub use vault::Vault;
}