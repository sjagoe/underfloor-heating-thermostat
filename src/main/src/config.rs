use core::time::Duration;

use control::{CoreConfig, ElectricityPrice, Temperature};

#[toml_cfg::toml_config]
pub struct TomlConfig {
    #[default(300)]
    measurement_interval: u64,
    #[default(18.0)]
    set_point_minimum_temperature: f32,
    #[default(25.0)]
    set_point_maximum_temperature: f32,
    #[default(0.15)]
    set_point_maximum_price: f32,
}

pub struct Config {
    pub measurement_interval: Duration,
    pub fake_electricity_price: ElectricityPrice,
    pub set_points: CoreConfig,
}

impl Config {
    pub fn read() -> Self {
        Config::from(TOML_CONFIG)
    }
}

impl From<TomlConfig> for Config {
    fn from(config: TomlConfig) -> Self {
        Config {
            measurement_interval: Duration::from_secs(config.measurement_interval),
            set_points: CoreConfig {
                minimum_temperature: Temperature::new(config.set_point_minimum_temperature),
                maximum_temperature: Temperature::new(config.set_point_maximum_temperature),
                turbo_temperature: Temperature::new(30.0),
                maximum_price: ElectricityPrice::new(config.set_point_maximum_price),
            },
            ..Config::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // fixme, we should measure every few minutes at most
            measurement_interval: Duration::from_secs(1),
            fake_electricity_price: ElectricityPrice::new(0.20),
            set_points: CoreConfig {
                minimum_temperature: Temperature::new(15.0),
                maximum_temperature: Temperature::new(22.0),
                turbo_temperature: Temperature::new(30.0),
                maximum_price: ElectricityPrice::new(0.30),
            },
        }
    }
}
