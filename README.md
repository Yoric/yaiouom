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
> the documentation of [`unify`](struct.Measure.html#method.unify).



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
> [`unify`](struct.Measure.html#method.unify).

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