//! Embedded logics, which may be used to manually prove unification.

use private;
use unit::*;

use std::marker::PhantomData;

#[rustc_yaiouom_combinator_dimensionless]
pub struct PDimensionless;
impl Unit for PDimensionless {
    fn name() -> String {
        "".to_string()
    }
}
impl private::Sealed for PDimensionless {}
impl Dimensionless for PDimensionless {}

/// Exposing type-level product.
#[rustc_yaiouom_combinator_mul]
pub struct PMul<A, B> where A: Unit, B: Unit {
    left: PhantomData<A>,
    right: PhantomData<B>,
}
impl<A: Unit, B: Unit> Unit for PMul<A, B> {
    fn name() -> String {
        format!("{}*{}", A::name(), B::name())
    }
}

/// Exposing type-level inversion.
#[rustc_yaiouom_combinator_inv]
pub struct PInv<A> where A: Unit {
    inner: PhantomData<A>
}
impl<A: Unit> Unit for PInv<A> {
    fn name() -> String {
        format!("({})^-1", A::name())
    }
}
