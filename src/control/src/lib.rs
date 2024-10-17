#![no_std]

use anyhow::Result;

// Temperature in degrees celcius
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Temperature(i16);

impl Default for Temperature {
    fn default() -> Self {
        Temperature(0)
    }
}

// Electricity price in cents per kWh
// Can be negative .. that happens, sometimes
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct ElectricityPrice(i16);

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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PowerState {
    On,
    Off,
}

impl Default for PowerState {
    fn default() -> Self {
        PowerState::Off
    }
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

pub fn desired_state(
    current_state: &State,
    config: &CoreConfig,
    current_price: ElectricityPrice,
) -> Result<SetPoint> {
    // Temperature below our low point
    if current_state.temperature < config.minimum_temperature {
        // Turn on heating high to recover low point
        let point = SetPoint {
            power: PowerState::On,
            temperature: config.turbo_temperature,
        };
        return Ok(point);
    }

    // Electricity price too high
    if current_price > config.maximum_price {
        // Turn on heating high to recover low point
        let point = SetPoint {
            power: PowerState::Off,
            temperature: config.minimum_temperature,
        };
        return Ok(point);
    }

    let state = SetPoint {
        power: PowerState::On,
        temperature: config.minimum_temperature,
    };
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let electricity_price = ElectricityPrice(5);
        let current_state = State {
            power: PowerState::On,
            temperature: Temperature(18),
        };
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(21),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(30),
        };
        let result =
            desired_state(&current_state, &settings, electricity_price).expect("should work");
        let expected = SetPoint {
            power: PowerState::On,
            temperature: Temperature(5),
        };
        assert_eq!(result, expected);
    }
}
