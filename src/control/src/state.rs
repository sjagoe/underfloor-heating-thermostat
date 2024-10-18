use fixed::types::I7F9;

// Temperature in degrees celcius
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Temperature(pub i16);

impl Temperature {
    pub fn new(value: i16) -> Self {
        Self(value)
    }
}

impl From<Temperature> for i16 {
    fn from(value: Temperature) -> i16 {
        value.0
    }
}

// Electricity price in cents per kWh
// Can be negative .. that happens, sometimes
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct ElectricityPrice(I7F9);

impl ElectricityPrice {
    pub fn new(value: I7F9) -> Self {
        Self(value)
    }
}

impl From<ElectricityPrice> for I7F9 {
    fn from(value: ElectricityPrice) -> I7F9 {
        value.0
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PowerState {
    On,
    Off,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct State {
    pub power: PowerState,
    pub temperature: Temperature,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SetPoint {
    pub power: PowerState,
    pub temperature: Temperature,
}
