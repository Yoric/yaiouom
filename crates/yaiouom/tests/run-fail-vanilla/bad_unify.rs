// should-fail

extern crate yaiouom;

use yaiouom::*;
use yaiouom::si::*;


// The following should compile with Rust but fail with the linter.
struct Kilometer;
impl BaseUnit for Kilometer {
    const NAME: &'static str = "km";
}
fn get_speed_bad(distance: Measure<f64, Kilometer>, duration: Measure<f64, Second>) -> Measure<f64, Mul<Meter, Inv<Second>>> {
    return ((Dimensionless::new(1.) / duration) * distance ).unify(); //~ERROR
}

fn main() {
    let distance = Kilometer::new(1.0);
    let duration = Second::new(1.0);
    let _ = get_speed_bad(distance, duration);
}