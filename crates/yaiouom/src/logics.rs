//! Embedded logics, which may be used to manually prove unification.

use private;
use unit::*;

use std::marker::PhantomData;

#[rustc_yaiouom_combinator_dimensionless]
pub struct PDimensionless;
impl Unit for PDimensionless {
    fn add_to_runtime(_: &mut RuntimeUnit, _: bool) {
        // Nothing to do.
    }
}
impl private::Sealed for PDimensionless {}

/// Exposing type-level product.
#[rustc_yaiouom_combinator_mul]
pub struct PMul<A, B> where A: Unit, B: Unit {
    left: PhantomData<A>,
    right: PhantomData<B>,
}
impl<A: Unit, B: Unit> private::Sealed for PMul<A, B> { }
impl<A: Unit, B: Unit> Unit for PMul<A, B> {
    fn add_to_runtime(repr: &mut RuntimeUnit, positive: bool) {
        A::add_to_runtime(repr, positive);
        B::add_to_runtime(repr, positive);
    }
}

/// Exposing type-level inversion.
#[rustc_yaiouom_combinator_inv]
pub struct PInv<A> where A: Unit {
    inner: PhantomData<A>
}
impl<A: Unit> private::Sealed for PInv<A> { }
impl<A: Unit> Unit for PInv<A> {
    fn add_to_runtime(repr: &mut RuntimeUnit, positive: bool) {
        A::add_to_runtime(repr, !positive);
    }
}
