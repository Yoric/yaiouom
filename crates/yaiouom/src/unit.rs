use private;
pub use logics::{ PDimensionless, PInv, PMul };

use std;
use std::any::*;
use std::collections::HashMap;
use std::marker::PhantomData;

use itertools::Itertools;

/// A base unit of measure (e.g. meters, euros, ...)
pub trait BaseUnit: Any {
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

    /// Unify two measures.
    ///
    /// # Warning
    ///
    /// With out-of-the-box Rust, this method is unsafe wrt units of measure.
    /// The companion linter is necessary to check that the call to `unify`
    /// is safe.
    ///
    /// # Future
    ///
    /// For a future version, we may introduce a dynamically typed implementation
    /// of `unify`, for users of vanilla Rust in debug mode.
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

    /// ```
    /// use yaiouom::*;
    /// use yaiouom::si::*;
    ///
    /// let one_meter_usize : Measure<i32, Meter> = Measure::new(1);
    /// assert_eq!(one_meter_usize.as_runtime().to_string(), "m");
    /// ```
    pub fn as_runtime(&self) -> RuntimeUnit {
        U::as_runtime()
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

// FIXME: We probably want to implement `mul` without `std::ops::Mul`, specify
// syntactic sugar later.
impl<T, U: Unit, V: Unit> std::ops::Mul<Measure<T, V>> for Measure<T, U> where
    T: std::ops::Mul<T>,
{
    type Output = Measure<<T as std::ops::Mul>::Output, PMul<U, V>>;
    /// Multiply two measures
    ///
    /// ```
    /// use yaiouom::*;
    /// use yaiouom::si::*;
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
    /// use yaiouom::*;
    /// use yaiouom::si::*;
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
    /// let unit_str = PMul::<Meter, PInv<Second>>::as_runtime().to_string();
    /// assert_eq!(&unit_str, "m * s^-1");
    ///
    /// let unit_str_2 = PMul::<PInv<Second>, Meter>::as_runtime().to_string();
    /// assert_eq!(&unit_str_2, "m * s^-1");
    /// ```
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
