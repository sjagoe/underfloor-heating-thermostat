use serde::{Deserialize, Serialize};

// Temperature in degrees celcius
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Deserialize, Serialize)]
pub struct Temperature(f32);

impl Temperature {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}

impl From<Temperature> for f32 {
    fn from(value: Temperature) -> f32 {
        value.0
    }
}

// Electricity price in cents per kWh
// Can be negative .. that happens, sometimes
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Deserialize, Serialize)]
pub struct ElectricityPrice(f32);

impl ElectricityPrice {
    pub fn new(value: f32) -> Self {
        Self(value)
    }
}

impl From<ElectricityPrice> for f32 {
    fn from(value: ElectricityPrice) -> f32 {
        value.0
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PowerState {
    On,
    Off,
}
