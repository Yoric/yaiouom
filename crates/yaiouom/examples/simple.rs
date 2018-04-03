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

/*
// The following has too many generics, Rust can't infer it out of the box.
fn get_speed_generic_2<A: Distance, B: Duration>(distance: Measure<f64, A>, duration: Measure<f64, B>) -> Measure<f64, Mul<A, Inv<B>>> {
    return (distance / duration)
        .unify()
        .unify();
}
*/
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

// The following should compile with Rust but fail with the linter.
struct Kilometer;
impl BaseUnit for Kilometer {
    const NAME: &'static str = "km";
}
fn get_speed_bad(distance: Measure<f64, Kilometer>, duration: Measure<f64, Second>) -> Measure<f64, Mul<Meter, Inv<Second>>> {
    return ((Dimensionless::new(1.) / duration) * distance ).unify();
}

// The same as get_speed_generic_2_annotated_2, but with bad annotations.
fn get_speed_generic_2_annotated_2_bad<A: Distance, B: Duration>(distance: Measure<f64, A>, duration: Measure<f64, B>) -> Measure<f64, Mul<A, Inv<B>>> {
    (distance / duration)
        .unify::<Mul<A, Inv<Second>>>()
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
/*
    fn get_speed_bad(&self) -> Measure<T::Output, Mul<Kilometer, Inv<Second>>> {
        self.distance / self.duration
    }
*/
    fn get_speed_bad_unify(&self) -> Measure<T::Output, Mul<Kilometer, Inv<Second>>> {
        (self.distance / self.duration).unify()
    }
}

fn main() {
    let distance = Meter::new(10.);
    let duration = Second::new(100.);
    let speed_1 = get_speed(distance, duration);
    let speed_2 = get_speed_2(distance, duration);
    print!("Speed: {} / {}", speed_1.as_ref(), speed_2.as_ref());

    let bad_speed = get_speed_bad(Kilometer::new(10.), Second::new(100.));
}