//! International system of units.
use unit::BaseUnit;

pub struct Meter;
impl BaseUnit for Meter {
    const NAME: &'static str = "m";
}

pub struct Second;
impl BaseUnit for Second {
    const NAME: &'static str = "s";
}

pub struct Kg;
impl BaseUnit for Kg {
    const NAME: &'static str = "kg";
}

pub struct Ampere;
impl BaseUnit for Ampere {
    const NAME: &'static str = "A";
}
