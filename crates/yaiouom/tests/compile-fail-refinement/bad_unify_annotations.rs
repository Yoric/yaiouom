extern crate yaiouom;

use yaiouom::*;
use yaiouom::si::*;

trait Distance: Unit {}
trait Duration: Unit {}
impl Distance for Meter {}
impl Duration for Second {}

fn get_speed_bad<A: Distance, B: Duration>(distance: Measure<f64, A>, duration: Measure<f64, B>) -> Measure<f64, Mul<A, Inv<B>>> {
    (distance / duration)  //~ERROR
        .unify::<Mul<A, Inv<Second>>>() //~^ERROR
        .unify()
}


fn main() {
    let distance = Meter::new(1.0);
    let duration = Second::new(1.0);
    let _ = get_speed_bad(distance, duration);
}