#![no_std]

use fixed::types::I7F9;
use fixed::traits::LossyInto;

// Temperature in degrees celcius
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Temperature(i16);

// Electricity price in cents per kWh
// Can be negative .. that happens, sometimes
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct ElectricityPrice(I7F9);

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

pub fn select_temperature(
    config: &CoreConfig,
    current_price: ElectricityPrice,
) -> Temperature {
    let price_difference = config.maximum_price.0 - current_price.0;

    let scaling_factor = price_difference / config.maximum_price.0;

    let temperature_range = config.maximum_temperature.0 - config.minimum_temperature.0;
    let temperature_delta: I7F9 = temperature_range * scaling_factor;

    let native_delta: i16 = temperature_delta.lossy_into();
    let set_temperature = config.minimum_temperature.0 + native_delta;

    Temperature(set_temperature)
}

pub fn desired_state(
    current_state: &State,
    config: &CoreConfig,
    current_price: ElectricityPrice,
) -> SetPoint {
    // Temperature below our low point
    if current_state.temperature < config.minimum_temperature {
        // Turn on heating high to recover low point
        return SetPoint {
            power: PowerState::On,
            temperature: config.turbo_temperature,
        };
    }

    // Electricity price too high
    if current_price > config.maximum_price {
        // Turn on heating high to recover low point
        return SetPoint {
            power: PowerState::Off,
            temperature: config.minimum_temperature,
        };
    }

    let set_temperature = select_temperature(config, current_price);
    SetPoint {
        power: PowerState::On,
        temperature: set_temperature,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_temperature_zero_price() {
        let electricity_price = ElectricityPrice(I7F9::lit("0"));
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(22),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(I7F9::lit("0.30")),
        };
        let set_temperature = select_temperature(&settings, electricity_price);
        assert_eq!(set_temperature, settings.maximum_temperature);
    }

    #[test]
    fn test_select_temperature_low_price() {
        let electricity_price = ElectricityPrice(I7F9::lit("0.05"));
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(22),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(I7F9::lit("0.30")),
        };
        let set_temperature = select_temperature(&settings, electricity_price);
        assert_eq!(set_temperature, Temperature(20));
    }

    #[test]
    fn test_select_temperature_max_price() {
        let electricity_price = ElectricityPrice(I7F9::lit("0.30"));
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(20),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(I7F9::lit("0.30")),
        };
        let set_temperature = select_temperature(&settings, electricity_price);
        assert_eq!(set_temperature, settings.minimum_temperature);
    }

    #[test]
    fn test_get_desired_state_low_price() {
        let electricity_price = ElectricityPrice(I7F9::lit("0.05"));
        let current_state = State {
            power: PowerState::On,
            temperature: Temperature(18),
        };
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(22),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(I7F9::lit("0.30")),
        };
        let result =
            desired_state(&current_state, &settings, electricity_price);
        let expected = SetPoint {
            power: PowerState::On,
            temperature: Temperature(20),
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_desired_state_high_price() {
        let electricity_price = ElectricityPrice(I7F9::lit("1.20"));
        let current_state = State {
            power: PowerState::On,
            temperature: Temperature(18),
        };
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(22),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(I7F9::lit("0.30")),
        };
        let result =
            desired_state(&current_state, &settings, electricity_price);
        let expected = SetPoint {
            power: PowerState::Off,
            temperature: settings.minimum_temperature,
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_desired_state_low_temperature() {
        let electricity_price = ElectricityPrice(I7F9::lit("1.20"));
        let current_state = State {
            power: PowerState::On,
            temperature: Temperature(12),
        };
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(22),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(I7F9::lit("0.30")),
        };
        let result =
            desired_state(&current_state, &settings, electricity_price);
        let expected = SetPoint {
            power: PowerState::On,
            temperature: settings.turbo_temperature,
        };
        assert_eq!(result, expected);
    }
}
