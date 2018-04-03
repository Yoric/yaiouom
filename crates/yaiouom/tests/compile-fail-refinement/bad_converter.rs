extern crate yaiouom;
extern crate num_traits;

use yaiouom::*;
use yaiouom::si::*;

use std::marker::PhantomData;

fn convert_2d<A: Unit, B: Unit, F>(convert_1d: F, input: Measure<f64, Mul<A, A>>) -> Measure<f64, Mul<B, B>>
    where F: Fn(Measure<f64, A>) -> Measure<f64, B>
{
    let b = convert_1d(input.sqrt());
    b * b
}

fn convert_inv<A: Unit, B: Unit, F>(convert_1d: F, input: Measure<f64, Inv<A>>) -> Measure<f64, Inv<B>>
    where F: Fn(Measure<f64, A>) -> Measure<f64, B>
{
    use num_traits::ops::inv::Inv;
    convert_1d(input.inv().unify())
        .inv()
        .unify()
}

struct Converter<A: Unit, B: Unit, F: Fn(Measure<f64, A>) -> Measure<f64, B>> {
    left: PhantomData<A>,
    right: PhantomData<B>,
    convert_1d: F,
}
impl<A, B, F> Converter<A, B, F> where A: Unit, B: Unit, F: Fn(Measure<f64, A>) -> Measure<f64, B> {
    fn convert_2d(&self, input: Measure<f64, Mul<A, A>>) -> Measure<f64, Mul<B, B>> {
        let b = (self.convert_1d)(input.sqrt());
        (b * b)
    }
    fn convert_inv(&self, input: Measure<f64, Inv<A>>) -> Measure<f64, Inv<B>> {
        use num_traits::ops::inv::Inv;
        (self.convert_1d)(input.inv().unify())
            .inv()
            .unify()
    }
}

fn main() {
    // Let's make sure that it doesn't cause a panic.
    let result_1 = convert_2d(|x| x, Mul::<Meter, Meter>::new(1.0));
    let converter = Converter {
        left: PhantomData,
        right: PhantomData,
        convert_1d: |x| x,
    };
    let result_2 = converter.convert_2d(Mul::<Meter, Meter>::new(1.0));
    assert_eq!(result_1, result_2);

    let converter = Converter {
        left: PhantomData,
        right: PhantomData,
        convert_1d: |x: Measure<f64, Second>| x,
    };
    let result_3 = convert_inv(|x| x, Second::new(2.).unify()); //~ERROR
    let result_4 = converter.convert_inv(Second::new(2.).unify()); //~ERROR
    assert_eq!(result_3, result_4);
}
