//! Embedded logics, which may be used to manually prove unification.

use private;
use unit::*;

use std::marker::PhantomData;

pub struct PDimensionless;
impl Unit for PDimensionless {}
impl private::Sealed for PDimensionless {}
impl Dimensionless for PDimensionless {}

/// Exposing type-level product.
pub struct PMul<A, B> where A: Unit, B: Unit {
    left: PhantomData<A>,
    right: PhantomData<B>,
}
impl<A: Unit, B: Unit> Unit for PMul<A, B> {}
impl<A: Unit, B: Unit> private::Sealed for PMul<A, B> {}
impl<A: Unit, B: Unit> Mul for PMul<A, B> {
    type Left = A;
    type Right = B;
}

/// Exposing type-level inversion.
pub struct PInv<A> where A: Unit {
    inner: PhantomData<A>
}
impl<A: Unit> Unit for PInv<A> {}
impl<A: Unit> private::Sealed for PInv<A> {}
impl<A: Unit> Inv for PInv<A> {
    type Inner = A;
}

/// Exposing type-level commutativity
pub struct PComm<A> where A: Mul {
    inner: PhantomData<A>
}
impl<A: Mul> Unit for PComm<A> {}
impl<A: Mul> private::Sealed for PComm<A> {}
impl<A: Mul> Mul for PComm<A> {
    type Left  = A::Right;
    type Right = A::Left;
}

/// Exposing type-level associativity
pub struct PAssoc<A: Unit, B: Mul> {
    left: PhantomData<A>,
    right: PhantomData<B>,
}
impl<A: Unit, B: Mul> Unit for PAssoc<A, B> {}
impl<A: Unit, B: Mul> private::Sealed for PAssoc<A, B> {}
impl<A: Unit, B: Mul> Mul for PAssoc<A, B> {
    type Left = PMul<A, B::Left>;
    type Right = B::Right;
}

/// Inverse
impl<A: Unit, B: Inv<Inner = A>> Dimensionless for PMul<A, B> {

}

/// Neutral element is its own inverse
impl<A: Dimensionless> Dimensionless for PInv<A> {

}


/// Exposing neutrality of Id
pub struct PId<A: Mul> where A::Left : Dimensionless {
    inner: PhantomData<A>,
}
impl<A: Mul> Unit for PId<A> where A::Left : Dimensionless  { }
impl<A: Mul> private::Sealed for PId<A> where A::Left : Dimensionless  { }
impl<A: Mul> Mul for PId<A> where A::Left : Dimensionless, A::Right: Mul  {
    type Left = <<A as Mul>::Right as Mul>::Left;
    type Right = <<A as Mul>::Right as Mul>::Right;
}
impl<A: Mul> Inv for PId<A> where A::Left : Dimensionless, A::Right: Inv  {
    type Inner = <<A as Mul>::Right as Inv>::Inner;
}
