extern crate yaiouom;

use yaiouom::*;
use yaiouom::si::*;

// The following should build unsafely with Rust, then yaiouom-driver will ensure the safety of `unify`.
fn get_speed(distance: Measure<f64, Meter>, duration: Measure<f64, Second>) -> Measure<f64, Mul<Meter, Inv<Second>>> {
    return (distance / duration).unify();
}

// The following should build unsafely with Rust, then yaiouom-driver will ensure the safety of `unify`.
fn get_speed_2(distance: Measure<f64, Meter>, duration: Measure<f64, Second>) -> Measure<f64, Mul<Meter, Inv<Second>>> {
    return ((Dimensionless::new(1.) / duration) * distance ).unify();
}

// The following should build safely with vanilla Rust.
fn get_average<U: Unit>(number: usize) -> Measure<f64, U> {
    let total = (1..number)
        .map(|x| x as f64)
        .map(U::new)
        .fold(U::new(0.), std::ops::Add::add);
    total / (number as f64)
}

// The following should build safely with vanilla Rust.
fn get_average_2<U: Unit + std::iter::Sum<U>>(number: usize) -> Measure<f64, U> {
    let total : Measure<_, _> = (1..number)
        .map(|x| x as f64)
        .map(U::new)
        .sum();
    total / (number as f64)
}

trait Distance: Unit {}
trait Duration: Unit {}

// The following should build unsafely with Rust, then yaiouom-driver will ensure the safety of `unify`.
fn get_speed_generic<A: Distance, B: Duration>(distance: Measure<f64, A>, duration: Measure<f64, B>) -> Measure<f64, Mul<A, Inv<B>>> {
    return (distance / duration).unify();
}

// The same one, with annotations. Should build with Rust and yaiouom-driver.
fn get_speed_generic_2_annotated<A: Distance, B: Duration>(distance: Measure<f64, A>, duration: Measure<f64, B>) -> Measure<f64, Mul<A, Inv<B>>> {
    let a : Measure<_, Mul<A, Inv<B>>> = (distance / duration)
        .unify();
    return a
        .unify();
}

// The same one, with annotations. Should build with Rust and yaiouom-driver.
fn get_speed_generic_2_annotated_2<A: Distance, B: Duration>(distance: Measure<f64, A>, duration: Measure<f64, B>) -> Measure<f64, Mul<A, Inv<B>>> {
    (distance / duration)
        .unify::<Mul<A, Inv<B>>>()
        .unify()
}

// Using type aliases.
type MeterAlias = Meter;
fn get_speed_alias(distance: Measure<f64, MeterAlias>, duration: Measure<f64, Second>) -> Measure<f64, Mul<Meter, Inv<Second>>> {
    return ((Dimensionless::new(1.) / duration) * distance ).unify();
}

struct Foo<T> where T: std::ops::Div<T> + Copy {
    distance: Measure<T, Meter>,
    duration: Measure<T, Second>
}
impl<T> Foo<T> where T: std::ops::Div<T> + Copy {
    fn get_speed(&self) -> Measure<T::Output, Mul<Meter, Inv<Second>>> {
        self.distance / self.duration
    }
    fn get_speed_2(&self) -> Measure<T::Output, Mul<Inv<Second>, Meter>> {
        (self.distance / self.duration).unify()
    }
}


fn main() {
    // We just want to check that everything compiles.
}