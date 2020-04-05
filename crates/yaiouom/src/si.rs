//! International system of units.
use unit::BaseUnit;


pub struct Second;
impl BaseUnit for Second {
    const NAME: &'static str = "s";
}

pub struct Meter;
impl BaseUnit for Meter {
    const NAME: &'static str = "m";
}

pub struct Kg;
impl BaseUnit for Kg {
    const NAME: &'static str = "kg";
}

pub struct Ampere;
impl BaseUnit for Ampere {
    const NAME: &'static str = "A";
}

pub struct Kelvin;
impl BaseUnit for Kelvin {
    const NAME: &'static str = "K";
}

pub struct Mole;
impl BaseUnit for Mole {
    const NAME: &'static str = "mol";
}

pub struct Candela;
impl BaseUnit for Candela {
    const NAME: &'static str = "cd";
}
