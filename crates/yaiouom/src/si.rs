//! International system of units.
use unit::Unit;

pub struct Meter;
impl Unit for Meter {
    fn name() -> String {
        format!("m")
    }
}

pub struct Second;
impl Unit for Second {
    fn name() -> String {
        format!("s")
    }
}

pub struct Kg;
impl Unit for Kg {
    fn name() -> String {
        format!("kg")
    }
}

pub struct Ampere;
impl Unit for Ampere {
    fn name() -> String {
        format!("A")
    }
}
