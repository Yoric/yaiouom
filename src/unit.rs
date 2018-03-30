use private;
pub use logics::{ PDimensionless, PInv, PMul };
use std;
use std::marker::PhantomData;

/// A unit of measure.
///
/// This trait is meant to be used mainly with void structs, but can
/// be implemented by any type.
pub trait Unit {}

/// A value with a unit.
pub struct Measure<T, U: Unit> {
    value: T,
    unit: PhantomData<U>,
}

/// A dimensionless value.
///
/// This trait constitutes a proof obligation for the type-checker.
pub trait Dimensionless: Unit + private::Sealed {}

/// The type-level product of two units of measure.
///
/// This trait constitutes a proof obligation for the type-checker.
///
/// This trait is special, insofar as the type-checker ensures
/// that it can only be implemented by the type combinators
/// defined below.
pub trait Mul: Unit + private::Sealed where Self::Left: Unit, Self::Right: Unit {
    type Left;
    type Right;
}

/// The type-level product of two units of measure.
///
/// This trait constitutes a proof obligation for the type-checker.
///
/// This trait is special, insofar as the type-checker ensures
/// that it can only be implemented by the type combinators
/// defined below.
pub trait Inv: Unit + private::Sealed where Self::Inner: Unit {
    type Inner;
}


/// Convert from a dimensionless unit.
///
/// ```
/// use yaioum::*;
///
/// struct Meter;
/// impl Unit for Meter {}
///
/// let one_meter : Measure<_, Meter> = Measure::new(1);
/// ```
impl<T, U: Unit> Measure<T, U> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            unit: PhantomData,
        }
    }

    /// Convert between two value representations (e.g. `u32` vs `u64`)
    /// In the same unit.
    ///
    /// Ideally, this should be an implementation of `From`, but it conflicts
    /// with the reflexive implementation.
    ///
    /// ```
    /// use yaioum::*;
    ///
    /// struct Meter;
    /// impl Unit for Meter {}
    ///
    /// let one_meter_usize : Measure<i32, Meter> = Measure::new(1);
    /// let one_meter_isize : Measure<i64, Meter> = Measure::from(one_meter_usize);
    /// ```
    pub fn from<V>(value: Measure<V, U>) -> Self where T: From<V> {
        Self {
            value: From::from(value.value),
            unit: PhantomData
        }
    }

    /// Convert between two value representations (e.g. `u32` vs `u64`)
    /// In the same unit.
    ///
    /// Ideally, this should be an implementation of `From`, but it conflicts
    /// with the reflexive implementation.
    ///
    /// ```
    /// use yaioum::*;
    ///
    /// struct Meter;
    /// impl Unit for Meter {}
    ///
    /// let one_meter_usize : Measure<i32, Meter> = Measure::new(1);
    /// let one_meter_isize : Measure<i64, Meter> = one_meter_usize.into();
    /// ```
    pub fn into<V>(self) -> Measure<V, U> where T: Into<V> {
        Measure {
            value: Into::into(self.value),
            unit: PhantomData
        }
    }
}

impl<T, U: Unit> AsRef<T> for Measure<T, U> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

//#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
impl<T, U: Unit> Clone for Measure<T, U> where T: Clone {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            unit: PhantomData
        }
    }
}

impl<T, U: Unit> Copy for Measure<T, U> where T: Copy { }

impl<T, U: Unit> PartialEq for Measure<T, U> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.value, &other.value)
    }
}

impl<T, U: Unit> Eq for Measure<T, U> where T: PartialEq { }

impl<T, U: Unit> PartialOrd for Measure<T, U> where T: PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(&self.value, &other.value)
    }
}

impl<T, U: Unit> Ord for Measure<T, U> where T: Ord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&self.value, &other.value)
    }
}

/// Out of the box, one may only add two values with the same unit.
///
/// It remains, however, possible to manually implement Add e.g. for
/// a time and a duration or a point and a vector.
///
/// ```
/// use yaioum::*;
///
/// struct Meter;
/// impl Unit for Meter {}
///
/// let one_meter : Measure<i32, Meter> = Measure::new(1);
/// let two_meters = one_meter + one_meter;
/// assert_eq!(*two_meters.as_ref(), 2);
/// ```
impl<T, U: Unit> std::ops::Add<Self> for Measure<T, U> where T: std::ops::Add<Output = T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Measure {
            value: self.value + rhs.value,
            unit: PhantomData,
        }
    }
}

impl<T, U: Unit> std::ops::Mul<T> for Measure<T, U> where T: std::ops::Mul<T> {
    type Output = Measure<<T as std::ops::Mul>::Output, U>;
    /// Multiply a dimensionless value with a measure.
    ///
    /// ```
    /// use yaioum::*;
    ///
    /// struct Meter;
    /// impl Unit for Meter {}
    ///
    /// let one_meter : Measure<i32, Meter> = Measure::new(1);
    /// let ten_meters : Measure<i32, Meter> = one_meter * 10;
    /// assert_eq!(ten_meters.as_ref(), &10);
    /// ```
    fn mul(self, rhs: T) -> Self::Output {
        Measure {
            value: self.value * rhs,
            unit: PhantomData,
        }
    }
}

// FIXME: We probably want to implement `mul` without `std::ops::Mul`, specify
// syntactic sugar later.
impl<T, U: Unit, V: Unit> std::ops::Mul<Measure<T, V>> for Measure<T, U> where
    T: std::ops::Mul<T>,
{
    type Output = Measure<<T as std::ops::Mul>::Output, PMul<U, V>>;
    /// Multiply two measures
    ///
    /// ```
    /// use yaioum::*;
    ///
    /// struct Meter;
    /// impl Unit for Meter {}
    ///
    /// let two_meters : Measure<i32, Meter> = Measure::new(2);
    /// let four_sq_meters : Measure<i32, PMul<Meter, Meter>> = two_meters * two_meters;
    /// assert_eq!(four_sq_meters.as_ref(), &4);
    /// ```
    fn mul(self, rhs: Measure<T, V>) -> Self::Output {
        Measure {
            value: self.value * rhs.value,
            unit: PhantomData,
        }
    }
}

impl<T, U: Unit, V: Unit> std::ops::Div<Measure<T, V>> for Measure<T, U> where
    T: std::ops::Div<T>,
{
    type Output = Measure<<T as std::ops::Div>::Output, PMul<U, PInv<V>>>;
    /// Divide two measures
    ///
    /// ```
    /// use yaioum::*;
    ///
    /// struct Meter;
    /// impl Unit for Meter {}
    ///
    /// let four_sq_meters : Measure<i32, PMul<Meter, Meter>> = Measure::new(4);
    /// let two_meters : Measure<i32, Meter> = Measure::new(2);
    /// let other_two_meters : Measure<i32, _> = four_sq_meters / two_meters;
    ///
    /// assert_eq!(two_meters.as_ref(), other_two_meters.as_ref());
    /// ```
    fn div(self, rhs: Measure<T, V>) -> Self::Output {
        Measure {
            value: self.value / rhs.value,
            unit: PhantomData,
        }
    }
}