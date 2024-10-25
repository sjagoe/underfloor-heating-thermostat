use crate::state::{ElectricityPrice, Temperature};

#[derive(Copy, Clone, Debug)]
pub struct CoreConfig {
    // Minimam allowed temperature set point if heating is on
    pub minimum_temperature: Temperature,

    // Maximum allowed temperature set point under normal conditions
    pub maximum_temperature: Temperature,

    // Emergency heating mode if temperature has dropped too far
    pub turbo_temperature: Temperature,

    // Electricity price at which to turn heating off
    pub maximum_price: ElectricityPrice,
}
