Units of measure.

This crate implements a mechanism of units of measure.
It may be used to manipulate all sorts of measures,
including physics/engineering (m, kg, s, A, m * s ^ 1,
...), currencies (EUR, USD, ...), statistics (dollars
per barrel, engineers per lightbulb, dollars per household
per year, ...)

While this is not the first implementation of units of
measure in Rust, this is the first one that is both
extensible (you can trivially add new base units),
compositional (two units defined in different crates
may interact without trouble) and type-safe (the compiler
will inform you if you attempt to mix several incompatible
units of measure without converting them first). However,
before using this crate, please read the rest of these
explanations.

# Example

The following computes a speed in f64 m * s^-1

```rust
extern crate yaiouom;

use yaiouom::*;
use yaiouom::si::*;

// The following builds unsafely with Rust, then yaiouom-checker ensures the safety of `unify`.
fn get_speed(distance: Measure<f64, Meter>, duration: Measure<f64, Second>) -> Measure<f64, Mul<Meter, Inv<Second>>> {
    return (distance / duration).unify();
}

fn main() {
    let distance = Meter::new(100.);
    let duration = Second::new(25.);
    let speed = get_speed(distance, duration);
}
```

If you're curious about the comment and this call to `unify`, you should really
look at the documentation of [`unify`](https://yoric.github.io/yaiouom/yaiouom/struct.Measure.html#method.unify) :)

Note that multiplication is commutative, so this is equivalent to
the following (we change the result of the function).


```rust
extern crate yaiouom;

use yaiouom::*;
use yaiouom::si::*;

// The following builds unsafely with Rust, then yaiouom-checker ensures the safety of `unify`.
fn get_speed(distance: Measure<f64, Meter>, duration: Measure<f64, Second>) -> Measure<f64, Mul<Inv<Second>, Meter>> {
    return (distance / duration).unify();
}
```

or even to the following (we have changed the type of `distance`)

```rust
extern crate yaiouom;

use yaiouom::*;
use yaiouom::si::*;

// The following builds unsafely with Rust, then yaiouom-checker ensures the safety of `unify`.
fn get_speed(distance: Measure<f64, Mul<Second, Mul<Meter, Inv<Second>>>, duration: Measure<f64, Second>) -> Measure<f64, Mul<Inv<Second>, Meter>> {
    return (distance / duration).unify();
}
```

Or, if you wish to be more generic,

```rust
extern crate yaiouom;

use yaiouom::*;
use yaiouom::si::*;


trait Distance: Unit {}
trait Duration: Unit {}

// The following builds unsafely with Rust, then yaiouom-checker ensures the safety of `unify`.
fn get_speed_generic<A: Distance, B: Duration>(distance: Measure<f64, A>, duration: Measure<f64, B>) -> Measure<f64, Mul<A, Inv<B>>> {
    return (distance / duration).unify();
}
```

Or, if you wish to be even more generic,

```rust
extern crate yaiouom;

use std;

use yaiouom::*;
use yaiouom::si::*;


trait Distance: Unit {}
trait Duration: Unit {}

// The following builds unsafely with Rust, then yaiouom-checker ensures the safety of `unify`.
fn get_speed_generic<A, B, T>(distance: Measure<T, A>, duration: Measure<T, B>) -> Measure<T::Output, Mul<A, Inv<B>>>
    where A: Distance,
          B: Duration,
          T: std::ops::Mul<T>
{
    return (distance / duration).unify();
}
```

You can easily add new units of measure:

```rust
struct Kilometer;
impl BaseUnit for Kilometer {
    const NAME: &'static str = "km";
}

fn get_speed_km(distance: Measure<f64, Kilometer>, duration: Measure<f64, Second>) -> Measure<f64, Mul<Kilometer, Inv<Second>>> {
    return (distance / duration).unify();
}
```

On the other hand, if you attempt to write a program that misuses units of measure,
the companion **linter** will inform you of your error:


```rust
struct Kilometer;
impl BaseUnit for Kilometer {
    const NAME: &'static str = "km";
}

fn get_speed_bad_unify(distance: Measure<f64, Kilometer>, duration: Measure<f64, Second>) -> Measure<f64, Mul<Meter, Inv<Second>>> {
    return (distance / duration).unify();
}

// 69 | / fn get_speed_bad(distance: Measure<f64, Kilometer>, duration: Measure<f64, Second>) -> Measure<f64, Mul<Meter, Inv<Second>>> {
// 70 | |     return ((Dimensionless::new(1.) / duration) * distance ).unify();
//    | |            --------------------------------------------------------- in this unification
// 71 | | }
//    | |_^ While examining this function
//    |
//    = note: expected unit of measure: `Kilometer`
//               found unit of measure: `yaiouom::si::Meter`
```

Or, if for some reason you decide to run the code without the linter,


```
thread 'main' panicked at 'assertion failed: `(left == right)`
  left: `km * s^-1`,
 right: `m * s^-1`', src/unit.rs:158:9
```

# Unification and the companion linter

At the time of this writing, the Rust type system is not
powerful enough to permit an extensible, compositional,
type-safe representation of units of measure. For this
reason, other crates implementing units of measure have
needed to make a choice:

- either prevent compositional extensibility;
- or give up on type safety.

This crate uses a different approach, by delegating safety
checks to a specialized checker, `yaiouom-checker`. This
checker extends Rust's type system with a mechanism ensuring
that units of measure are used safely.

If you do not use the checker, you'll end up with a binary
that performs (slow) dynamic unit checking in debug builds,
and no unit checking in optimized builds.

The linter guarantees that you'll never hit such dynamic
panics.

> You really should use the companion linter :) Also, please see
> the documentation of [`unify`](https://yoric.github.io/yaiouom/yaiouom/struct.Measure.html#method.unify).



# Representation of values

Different values have different rules. Many are f32 or f64,
but currency computations, for instance, need to be performed
with either rationals or fixed point arithmetics. Some electrical
measures are actually complex values. Statistics may use integer
values for population, etc.

For this reason, yaiouom does not hardcode a specific representation
of values. A value with a unit is a `Measure<T, U: Unit>`, where
`T` can be any kind of number or number-like value.



# Limitations

As discussed above,

> Please use the companion linter! Also, please see the documentation of
> [`unify`](https://yoric.github.io/yaiouom/yaiouom/struct.Measure.html#method.unify).

This crate attempts to be strictly minimal.

Unit conversion is a complicated thing. We do not attempt to
solve this problem.

Some values cannot be multiplied or divided (e.g. ºC, ºF, pH,
dB). We do not attempt to differentiate between units that can be
multiplied/dived and units that can, although this might happen
in a future version.

Some values have different definitions when subtracted. For instance,
the difference between two dates in seconds is a duration in seconds.
The difference between two ºC temperatures is a value that may be
multiplied or divided. We do not attempt to differentiate between
these things.

# Credits

While this refinement type is much simpler (and more limited)
than the original, it draws heavy inspiration from F#'s type
system.
