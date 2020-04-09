//! International system of units.
use unit::BaseUnit;

/// Base unit of time
pub struct Second;
impl BaseUnit for Second {
    const NAME: &'static str = "s";
}

/// Base unit of length
pub struct Meter;
impl BaseUnit for Meter {
    const NAME: &'static str = "m";
}

/// Base unit of mass
pub struct Kg;
impl BaseUnit for Kg {
    const NAME: &'static str = "kg";
}

/// Base unit of electrical current
pub struct Ampere;
impl BaseUnit for Ampere {
    const NAME: &'static str = "A";
}

/// Base unit of temperature
pub struct Kelvin;
impl BaseUnit for Kelvin {
    const NAME: &'static str = "K";
}

/// Base unit for amount of substance
pub struct Mole;
impl BaseUnit for Mole {
    const NAME: &'static str = "mol";
}

/// Base unit of luminous intensity
pub struct Candela;
impl BaseUnit for Candela {
    const NAME: &'static str = "cd";
}
