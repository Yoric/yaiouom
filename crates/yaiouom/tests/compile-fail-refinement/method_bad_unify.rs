extern crate yaiouom;

use yaiouom::*;
use yaiouom::si::*;

struct Kilometer;
impl BaseUnit for Kilometer {
    const NAME: &'static str = "km";
}


struct Foo<T> where T: std::ops::Div<T> + Copy {
    distance: Measure<T, Meter>,
    duration: Measure<T, Second>
}

impl<T> Foo<T> where T: std::ops::Div<T> + Copy {
    fn get_speed_bad_unify(&self) -> Measure<T::Output, Mul<Kilometer, Inv<Second>>> {
        (self.distance / self.duration).unify() //~ERROR
    }
}


fn main() {
    let foo = Foo {
        distance: Meter::new(1.0),
        duration: Second::new(1.0),
    };
    let _ = foo.get_speed_bad_unify();
}