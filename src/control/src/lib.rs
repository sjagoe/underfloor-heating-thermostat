#![no_std]

mod config;
mod state;
mod thermistor;

pub use config::CoreConfig;
pub use state::{ElectricityPrice, PowerState, Temperature};
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct State {
    pub power: PowerState,
    pub temperature: Temperature,
}

impl State {
    pub fn from_current_state(
        config: &CoreConfig,
        current_temperature: Temperature,
        current_price: ElectricityPrice,
    ) -> State {
        if current_temperature > config.turbo_temperature {
            return State {
                power: PowerState::Off,
                temperature: config.minimum_temperature,
            };
        }

        // Temperature below our low point
        if current_temperature < config.minimum_temperature {
            // Turn on heating high to recover low point
            return State {
                power: PowerState::On,
                temperature: config.turbo_temperature,
            };
        }

        // Electricity price too high
        if current_price > config.maximum_price {
            // Turn on heating high to recover low point
            return State {
                power: PowerState::Off,
                temperature: config.minimum_temperature,
            };
        }

        let set_temperature = select_temperature(config, current_price);
        State {
            power: PowerState::On,
            temperature: set_temperature,
        }
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

        let min = Temperature::new(20.8);
        let max = Temperature::new(20.9);
        assert!(min < set_temperature, "{:?} < {:?}", min, set_temperature);
        assert!(set_temperature < max, "{:?} < {:?}", set_temperature, max);
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
        let current_temperature = Temperature::new(18.0);
        let settings = CoreConfig {
            minimum_temperature: Temperature::new(15.0),
            maximum_temperature: Temperature::new(22.0),
            turbo_temperature: Temperature::new(30.0),
            maximum_price: ElectricityPrice::new(0.30),
        };
        let result =
            State::from_current_state(&settings, current_temperature, electricity_price);

        assert_eq!(result.power, PowerState::On);
        let min = Temperature::new(20.8);
        let max = Temperature::new(20.9);
        assert!(min < result.temperature, "{:?} < {:?}", min, result.temperature);
        assert!(result.temperature < max, "{:?} < {:?}", result.temperature, max);
    }

    #[test]
    fn test_get_desired_state_high_price() {
        let electricity_price = ElectricityPrice::new(1.20);
        let current_temperature = Temperature::new(18.0);
        let settings = CoreConfig {
            minimum_temperature: Temperature::new(15.0),
            maximum_temperature: Temperature::new(22.0),
            turbo_temperature: Temperature::new(30.0),
            maximum_price: ElectricityPrice::new(0.30),
        };
        let result =
            State::from_current_state(&settings, current_temperature, electricity_price);
        let expected = State {
            power: PowerState::Off,
            temperature: settings.minimum_temperature,
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_desired_state_low_temperature() {
        let electricity_price = ElectricityPrice::new(1.20);
        let current_temperature = Temperature::new(12.0);
        let settings = CoreConfig {
            minimum_temperature: Temperature::new(15.0),
            maximum_temperature: Temperature::new(22.0),
            turbo_temperature: Temperature::new(30.0),
            maximum_price: ElectricityPrice::new(0.30),
        };
        let result =
            State::from_current_state(&settings, current_temperature, electricity_price);
        let expected = State {
            power: PowerState::On,
            temperature: settings.turbo_temperature,
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_desired_state_very_high_temperature() {
        let electricity_price = ElectricityPrice::new(-1.20);
        let current_temperature = Temperature::new(45.0);
        let settings = CoreConfig {
            minimum_temperature: Temperature::new(15.0),
            maximum_temperature: Temperature::new(22.0),
            turbo_temperature: Temperature::new(30.0),
            maximum_price: ElectricityPrice::new(0.30),
        };
        let result =
            State::from_current_state(&settings, current_temperature, electricity_price);
        let expected = State {
            power: PowerState::Off,
            temperature: settings.minimum_temperature,
        };
        assert_eq!(result, expected);
    }
}
