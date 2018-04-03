use private;

use std;
use std::any::*;
use std::collections::HashMap;
use std::marker::PhantomData;

use itertools::Itertools;
use num_traits;

/// A base unit of measure (e.g. meters, euros, ...)
pub trait BaseUnit: Any {
    /// The human-readable name of the unit, e.g. `"m"`
    /// `"s"`, `"EUR"`, etc.
    ///
    /// Used mainly for debugging purposes.
    const NAME: &'static str;
}
impl<T: BaseUnit> private::Sealed for T {}

/// A unit of measure.
///
/// To implement a new Unit, use BaseUnit.
pub trait Unit: private::Sealed {
    fn new<T>(value: T) -> Measure<T, Self> where Self: Sized {
        Measure::new(value)
    }

    /// Return a runtime representation of this unit.
    /// This method is designed for indexing and debugging.
    /// Not particularly fast.
    fn as_runtime() -> RuntimeUnit {
        let mut runtime = RuntimeUnit::new();
        Self::add_to_runtime(&mut runtime, true);
        runtime
    }

    /// Add a compile-type unit to a dynamic unit, either
    /// in positive position (if `positive` is `true`)
    /// or in negative position (if `positive` is `false`).
    ///
    /// Used internally by `as_runtime`, not particularly interesting otherwise.
    fn add_to_runtime(repr: &mut RuntimeUnit, positive: bool);
}
impl<T: BaseUnit> Unit for T {
    fn add_to_runtime(repr: &mut RuntimeUnit, positive: bool) {
        let is_empty = {
            let entry = repr.dimensions.entry(TypeId::of::<T>())
                .or_insert_with(|| (T::NAME.to_string(), 0));
            if positive {
                entry.1 += 1;
            } else {
                entry.1 -= 1;
            }
            entry.1 == 0
        };
        if is_empty {
            repr.dimensions.remove(&TypeId::of::<T>());
        }
    }
}

/// A value with a unit.
#[allow(unused_attributes)]
#[rustc_yaiouom_check_unify_measure]
pub struct Measure<T, U: Unit> {
    value: T,
    unit: PhantomData<U>,
}

impl<T, U: Unit> Measure<T, U> {
    /// Convert from a dimensionless unit.
    ///
    /// ```
    /// use yaiouom::*;
    /// use yaiouom::si::*;
    ///
    /// let one_meter : Measure<_, Meter> = Measure::new(1);
    /// ```
    ///
    /// # Warning
    ///
    /// This function is somewhat unsafe, as you can use it
    /// to build values that are fully polymorphic in unit.
    ///
    /// Future versions will make this constructor private and
    /// hide it behind syntactic sugar.
    pub fn new(value: T) -> Self {
        Self {
            value,
            unit: PhantomData,
        }
    }

    /// Compare two units of measure (**not** their values).
    ///
    /// Out of the box, the Rust type system is not powerful enough to resolve
    /// general equality between two units of measure. So, for instance, it
    /// will not realize that `m * s` and `s * m` are the same unit, or that
    /// `m / m` is the same as the dimensionless unit.
    ///
    /// For instance, the following will fail to type:
    ///
    /// ```compile_fail
    /// use yaiouom::*;
    /// use yaiouom::si::*;
    ///
    /// let one_meter = Meter::new(1);
    /// let one_second = Second::new(1);
    ///
    /// let a = one_meter * one_second;
    /// let b = one_second * one_meter;
    /// assert_eq!(a, b);
    /// // ^^^^^^^^^^^^^^^^^ expected struct `Meter`, found struct `Second`
    /// ```
    ///
    /// To work around this, use `unify()`, as follows:
    ///
    /// ```
    /// use yaiouom::*;
    /// use yaiouom::si::*;
    ///
    /// let one_meter = Meter::new(1);
    /// let one_second = Second::new(1);
    ///
    /// let a = one_meter * one_second;
    /// let b = (one_second * one_meter).unify(); // Delays unit check.
    /// assert_eq!(a, b);
    /// ```
    ///
    /// # Soundness of `unify`
    ///
    /// If you look at the signature of `unify`, you can notice that
    /// it returns a `Measure<T, V>` for all `V`. Despite appearances,
    /// this is sound, for two reasons:
    ///
    ///
    /// ## A refinement type for units of measure
    ///
    /// The companion linter for this crate implements a refinement of
    /// the Rust type system dedicated to units of measure. What this
    /// means, in practice, is that whenever you call `foo.unify()`,
    /// it will check that the returned type is correct. By opposition
    /// to the general Rust type system, it will correctly realize that
    /// `m * s` and `s * m` are the same, or even that `W * m / W` is
    /// `m`, even if `Wß` is a type variable.
    ///
    /// Please don't use `unify()` without the linter :)
    ///
    ///
    /// ## Dynamic checks
    ///
    /// As a fallback, **in debug builds**, each call to `unify` will panic
    /// if type `V` is not equivalent ot type `U`.
    #[allow(unused_attributes)]
    #[rustc_yaiouom_check_unify]
    pub fn unify<V: Unit>(self) -> Measure<T, V> {
        // First, ensure that we can perform conversion.
        debug_assert_eq!(U::as_runtime(), V::as_runtime());
        Measure {
            value: self.value,
            unit: PhantomData,
        }
    }

    /// Convert between two value representations (e.g. `u32` vs `u64`)
    /// in the same unit.
    ///
    /// Ideally, this should be an implementation of `From`, but it conflicts
    /// with the reflexive implementation.
    ///
    /// ```
    /// use yaiouom::*;
    /// use yaiouom::si::*;
    ///
    /// let one_meter_usize = Meter::new(1 as i32);
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
    /// use yaiouom::*;
    /// use yaiouom::si::*;
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

    /// Extract a runtime representation of a unit.
    ///
    /// This runtime representation is designed mainly for debugging purposes.
    ///
    /// ```
    /// use yaiouom::*;
    /// use yaiouom::si::*;
    ///
    /// let one_meter_usize : Measure<i32, Meter> = Measure::new(1);
    /// assert_eq!(one_meter_usize.as_runtime().to_string(), "m");
    /// ```
    ///
    /// # Performance note
    ///
    /// This method is fine for debugging, but should not be used in a tight loop.
    pub fn as_runtime(&self) -> RuntimeUnit {
        U::as_runtime()
    }
}

impl<T, A: Unit> Measure<T, Mul<A, A>> where T: num_traits::float::Float {
    /// Compute the square root of a value.
    pub fn sqrt(self) -> Measure<T, A> {
        Measure {
            value: self.value.sqrt(),
            unit: PhantomData,
        }
    }
}

impl<T, U: Unit> num_traits::ops::inv::Inv for Measure<T, U> where T: num_traits::ops::inv::Inv {
    type Output = Measure<T::Output, Inv<U>>;
    /// Unary operator for retrieving the multiplicative inverse, or reciprocal, of a value.
    fn inv(self) -> Self::Output {
        Measure {
            value: self.value.inv(),
            unit: PhantomData,
        }
    }
}

impl<T, U: Unit> AsRef<T> for Measure<T, U> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

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

impl<T, U: Unit> std::ops::Neg for Measure<T, U> where T: std::ops::Neg {
    type Output = Measure<T::Output, U>;
    fn neg(self) -> Self::Output {
        Measure {
            value: self.value.neg(),
            unit: PhantomData,
        }
    }
}

impl<T, U: Unit> num_traits::identities::Zero for Measure<T, U> where T: num_traits::identities::Zero {
    fn zero() -> Self {
        Self {
            value: T::zero(),
            unit: PhantomData,
        }
    }
    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}

/// Out of the box, one may only add two values with the same unit.
///
/// It remains, however, possible to manually implement Add e.g. for
/// a time and a duration or a point and a vector.
///
/// ```
/// use yaiouom::*;
/// use yaiouom::si::*;
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
    /// use yaiouom::*;
    /// use yaiouom::si::*;
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

impl<T, U: Unit, V: Unit> std::ops::Mul<Measure<T, V>> for Measure<T, U> where
    T: std::ops::Mul<T>,
{
    type Output = Measure<<T as std::ops::Mul>::Output, Mul<U, V>>;
    /// Multiply two measures
    ///
    /// ```
    /// use yaiouom::*;
    /// use yaiouom::si::*;
    ///
    /// let two_meters : Measure<i32, Meter> = Measure::new(2);
    /// let four_sq_meters : Measure<i32, Mul<Meter, Meter>> = two_meters * two_meters;
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
    type Output = Measure<<T as std::ops::Div>::Output, Mul<U, Inv<V>>>;
    /// Divide two measures
    ///
    /// ```
    /// use yaiouom::*;
    /// use yaiouom::si::*;
    ///
    /// let four_sq_meters : Measure<i32, Mul<Meter, Meter>> = Measure::new(4);
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


impl<T, U: Unit> std::ops::Div<T> for Measure<T, U> where T: std::ops::Div<T> {
    type Output = Measure<<T as std::ops::Div>::Output, U>;
    /// Divide a dimensionless value by a measure.
    ///
    /// ```
    /// use yaiouom::*;
    /// use yaiouom::si::*;
    ///
    /// let ten_meters : Measure<i32, Meter> = Measure::new(10);
    /// let one_meter : Measure<i32, Meter> = ten_meters / 10;
    /// assert_eq!(one_meter.as_ref(), &1);
    /// ```
    fn div(self, rhs: T) -> Self::Output {
        Measure {
            value: self.value / rhs,
            unit: PhantomData,
        }
    }
}

impl<T, U: Unit> std::iter::Sum for Measure<T, U> where T: std::iter::Sum {
    fn sum<I: std::iter::Iterator<Item = Self>>(iter: I) -> Self {
        let sum = iter.map(|m| m.value)
            .sum();
        Measure {
            value: sum,
            unit: PhantomData,
        }
    }
}


impl<T, U: Unit> std::iter::Product for Measure<T, U> where T: std::iter::Product {
    fn product<I: std::iter::Iterator<Item = Self>>(iter: I) -> Self {
        let product = iter.map(|m| m.value)
            .product();
        Measure {
            value: product,
            unit: PhantomData,
        }
    }
}

impl<T, U: Unit> std::fmt::Debug for Measure<T, U> where T: std::fmt::Debug {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{:?}{}", self.value, U::as_runtime().to_string())
    }
}

/*
impl<T, U: Unit> num_traits::Float for Measure<T, U> where T: num_traits::Float {

}
*/

/// Runtime representation of a unit.
///
/// Used mainly for debug assertions and for debug formatting.
#[derive(PartialEq, Eq)]
pub struct RuntimeUnit {
    dimensions: HashMap<TypeId, (String, i32)>,
}
impl std::fmt::Debug for RuntimeUnit {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.to_string())
    }
}
impl RuntimeUnit {
    fn new() -> Self {
        Self {
            dimensions: HashMap::new()
        }
    }

    /// Display a RuntimeUnit as a string.
    ///
    /// Positives come before negatives, but otherwise, the order of elements
    /// is not specified. A dimensionless unit returns `""`.
    ///
    /// ```
    /// use yaiouom::*;
    /// use yaiouom::si::*;
    ///
    /// let unit_str = Mul::<Meter, Inv<Second>>::as_runtime().to_string();
    /// assert_eq!(&unit_str, "m * s^-1");
    ///
    /// let unit_str_2 = Mul::<Inv<Second>, Meter>::as_runtime().to_string();
    /// assert_eq!(&unit_str_2, "m * s^-1");
    ///
    /// let unit_str_3 = Mul::<Inv<Second>, Mul<Inv<Second>, Meter>>::as_runtime().to_string();
    /// assert_eq!(&unit_str_3, "m * s^-2");
    ///
    /// let unit_str_4 = Mul::<Inv<Ampere>, Mul<Inv<Second>, Meter>>::as_runtime().to_string();
    /// assert!(["m * s^-1 * A^-1", "m * A^-1 * s^-1"].iter().any(|x| *x == &unit_str_4));
    /// ```
    ///
    /// # Performance note
    ///
    /// This method is fine for debugging, but should not be used in a tight loop.
    pub fn to_string(&self) -> String {
        // First display the positive values.
        let positives = self.dimensions.values()
            .filter_map(|x| match x.1 {
                0 => panic!(),
                1 => Some(x.0.clone()),
                n if n > 1 => Some(format!("{}^{}", x.0, n)),
                _ => None
            });
        // Then display the negative values.
        let negatives = self.dimensions.values()
            .filter_map(|x| match x.1 {
                0 => panic!(),
                n if n <= -1 => Some(format!("{}^{}", x.0, n)),
                _ => None
            });
        format!("{}", positives.chain(negatives)
            .format(" * "))
    }
}

/// A unit without dimension.
#[allow(unused_attributes)]
#[rustc_yaiouom_combinator_dimensionless]
pub struct Dimensionless;
impl Unit for Dimensionless {
    fn add_to_runtime(_: &mut RuntimeUnit, _: bool) {
        // Nothing to do.
    }
}
impl private::Sealed for Dimensionless {}
impl<T> From<T> for Measure<T, Dimensionless> {
    fn from(value: T) -> Self {
        Self {
            value,
            unit: PhantomData,
        }
    }
}
impl<T> Measure<T, Dimensionless> {
    pub fn unwrap(self) -> T {
        self.value
    }
}

/// The product of two units of measure.
///
/// Note that Rust's type system is not powerful enough
/// to automatically realize that `Mul<A, B> == Mul<B, A>`.
///
/// See the documentation of [unify](struct.Measure.html#method.unify) for details
/// on how to work around this limitation.
#[allow(unused_attributes)]
#[rustc_yaiouom_combinator_mul]
pub struct Mul<A, B> where A: Unit, B: Unit {
    left: PhantomData<A>,
    right: PhantomData<B>,
}
impl<A: Unit, B: Unit> private::Sealed for Mul<A, B> { }
impl<A: Unit, B: Unit> Unit for Mul<A, B> {
    fn add_to_runtime(repr: &mut RuntimeUnit, positive: bool) {
        A::add_to_runtime(repr, positive);
        B::add_to_runtime(repr, positive);
    }
}

/// The inverse of a unit of measure.
///
/// Note that Rust's type system is not powerful enough
/// to automatically realize that `Mul<A, Inv<A>> == Dimensionless`.
///
/// See the documentation of [unify](struct.Measure.html#method.unify) for details
/// on how to work around this limitation.
#[allow(unused_attributes)]
#[rustc_yaiouom_combinator_inv]
pub struct Inv<A> where A: Unit {
    inner: PhantomData<A>
}
impl<A: Unit> private::Sealed for Inv<A> { }
impl<A: Unit> Unit for Inv<A> {
    fn add_to_runtime(repr: &mut RuntimeUnit, positive: bool) {
        A::add_to_runtime(repr, !positive);
    }
}

