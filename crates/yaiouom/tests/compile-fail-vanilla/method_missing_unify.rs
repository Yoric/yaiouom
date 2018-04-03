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
    fn get_speed_bad(&self) -> Measure<T::Output, Mul<Kilometer, Inv<Second>>> {
        self.distance / self.duration //~ERROR
    }
}
