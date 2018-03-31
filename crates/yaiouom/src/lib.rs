#![feature(rustc_attrs)]

//! A crate implementing a mechanism of units of measure.
//!
//! This crate is designed to be used with the companion
//! linter, which implements a refinement type system
//! that determines whether two units of measure are
//! equivalent, including type inference.

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