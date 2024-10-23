#![no_std]

mod config;
mod state;
mod thermistor;

use config::*;
use state::*;

pub use thermistor::temperature_from_voltage;

pub fn select_temperature(
    config: &CoreConfig,
    current_price: ElectricityPrice,
) -> Temperature {
    let max_price = f32::from(config.maximum_price);
    let price_difference = max_price - f32::from(current_price);

    let scaling_factor = price_difference / max_price;

    let temperature_range = f32::from(config.maximum_temperature) - f32::from(config.minimum_temperature);
    let temperature_delta = temperature_range * scaling_factor;

    let set_temperature = f32::from(config.minimum_temperature) + temperature_delta;

    Temperature::new(set_temperature)
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
        let electricity_price = ElectricityPrice::new(0.0);
        let settings = CoreConfig {
            minimum_temperature: Temperature::new(15.0),
            maximum_temperature: Temperature::new(22.0),
            turbo_temperature: Temperature::new(30.0),
            maximum_price: ElectricityPrice::new(0.30),
        };
        let set_temperature = select_temperature(&settings, electricity_price);
        assert_eq!(set_temperature, settings.maximum_temperature);
    }

    #[test]
    fn test_select_temperature_low_price() {
        let electricity_price = ElectricityPrice::new(0.05);
        let settings = CoreConfig {
            minimum_temperature: Temperature::new(15.0),
            maximum_temperature: Temperature::new(22.0),
            turbo_temperature: Temperature::new(30.0),
            maximum_price: ElectricityPrice::new(0.30),
        };
        let set_temperature = select_temperature(&settings, electricity_price);
        assert_eq!(set_temperature, Temperature::new(20.0));
    }

    #[test]
    fn test_select_temperature_max_price() {
        let electricity_price = ElectricityPrice::new(0.30);
        let settings = CoreConfig {
            minimum_temperature: Temperature::new(15.0),
            maximum_temperature: Temperature::new(20.0),
            turbo_temperature: Temperature::new(30.0),
            maximum_price: ElectricityPrice::new(0.30),
        };
        let set_temperature = select_temperature(&settings, electricity_price);
        assert_eq!(set_temperature, settings.minimum_temperature);
    }

    #[test]
    fn test_get_desired_state_low_price() {
        let electricity_price = ElectricityPrice::new(0.05);
        let current_state = State {
            power: PowerState::On,
            temperature: Temperature::new(18.0),
        };
        let settings = CoreConfig {
            minimum_temperature: Temperature::new(15.0),
            maximum_temperature: Temperature::new(22.0),
            turbo_temperature: Temperature::new(30.0),
            maximum_price: ElectricityPrice::new(0.30),
        };
        let result =
            desired_state(&current_state, &settings, electricity_price);
        let expected = SetPoint {
            power: PowerState::On,
            temperature: Temperature::new(20.0),
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_desired_state_high_price() {
        let electricity_price = ElectricityPrice::new(1.20);
        let current_state = State {
            power: PowerState::On,
            temperature: Temperature::new(18.0),
        };
        let settings = CoreConfig {
            minimum_temperature: Temperature::new(15.0),
            maximum_temperature: Temperature::new(22.0),
            turbo_temperature: Temperature::new(30.0),
            maximum_price: ElectricityPrice::new(0.30),
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
        let electricity_price = ElectricityPrice::new(1.20);
        let current_state = State {
            power: PowerState::On,
            temperature: Temperature::new(12.0),
        };
        let settings = CoreConfig {
            minimum_temperature: Temperature::new(15.0),
            maximum_temperature: Temperature::new(22.0),
            turbo_temperature: Temperature::new(30.0),
            maximum_price: ElectricityPrice::new(0.30),
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
