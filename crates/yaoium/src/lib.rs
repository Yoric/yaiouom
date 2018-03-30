/// Seal mechanism, to ensure that we cannot implement private traits
/// from outside this module.
mod private {
    /// A trait that cannot be implemented outside this module.
    pub trait Sealed {}
}

pub mod logics;

mod unit;
pub use unit::*;

pub mod si;