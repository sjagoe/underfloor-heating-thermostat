#![no_std]

use anyhow::Result;

// Temperature in degrees celcius
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Temperature(i16);

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
    let price_difference: f64 = (config.maximum_price.0 - current_price.0) as f64;

    let scaling_factor = price_difference / config.maximum_price.0 as f64;

    let temperature_range = config.maximum_temperature.0 - config.minimum_temperature.0;
    let temperature_delta = (temperature_range as f64) * scaling_factor;

    let set_temperature = config.minimum_temperature.0 + (temperature_delta as i16);

    Temperature(set_temperature)
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

    let set_temperature = select_temperature(config, current_price);
    let state = SetPoint {
        power: PowerState::On,
        temperature: set_temperature,
    };
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_temperature_zero_price() {
        let electricity_price = ElectricityPrice(0);
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(22),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(30),
        };
        let set_temperature = select_temperature(&settings, electricity_price);
        assert_eq!(set_temperature, settings.maximum_temperature);
    }

    #[test]
    fn test_select_temperature_low_price() {
        let electricity_price = ElectricityPrice(5);
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(22),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(30),
        };
        let set_temperature = select_temperature(&settings, electricity_price);
        assert_eq!(set_temperature, Temperature(20));
    }

    #[test]
    fn test_select_temperature_max_price() {
        let electricity_price = ElectricityPrice(30);
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(20),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(30),
        };
        let set_temperature = select_temperature(&settings, electricity_price);
        assert_eq!(set_temperature, settings.minimum_temperature);
    }

    #[test]
    fn test_get_desired_state_low_price() {
        let electricity_price = ElectricityPrice(5);
        let current_state = State {
            power: PowerState::On,
            temperature: Temperature(18),
        };
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(22),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(30),
        };
        let result =
            desired_state(&current_state, &settings, electricity_price).expect("should work");
        let expected = SetPoint {
            power: PowerState::On,
            temperature: Temperature(20),
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_desired_state_high_price() {
        let electricity_price = ElectricityPrice(120);
        let current_state = State {
            power: PowerState::On,
            temperature: Temperature(18),
        };
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(22),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(30),
        };
        let result =
            desired_state(&current_state, &settings, electricity_price).expect("should work");
        let expected = SetPoint {
            power: PowerState::Off,
            temperature: settings.minimum_temperature,
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_desired_state_low_temperature() {
        let electricity_price = ElectricityPrice(120);
        let current_state = State {
            power: PowerState::On,
            temperature: Temperature(12),
        };
        let settings = CoreConfig {
            minimum_temperature: Temperature(15),
            maximum_temperature: Temperature(22),
            turbo_temperature: Temperature(30),
            maximum_price: ElectricityPrice(30),
        };
        let result =
            desired_state(&current_state, &settings, electricity_price).expect("should work");
        let expected = SetPoint {
            power: PowerState::On,
            temperature: settings.turbo_temperature,
        };
        assert_eq!(result, expected);
    }
}
