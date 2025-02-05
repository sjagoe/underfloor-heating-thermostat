use anyhow::{bail, Result};
use core::time::Duration;

use control::{CoreConfig, ElectricityPrice, Temperature};

mod private;

#[derive(Copy, Clone, Debug, Default)]
pub struct WifiConfig {
    pub ssid: &'static str,
    pub password: &'static str,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ServerConfig {
    pub electricity_price_api: &'static str,
    #[allow(dead_code)]
    pub metrics_url: &'static str,
    #[allow(dead_code)]
    pub ntp_server: &'static str,
}

#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub measurement_interval: Duration,
    pub set_points: CoreConfig,
    pub wifi: WifiConfig,
    pub server: ServerConfig,
}

impl Config {
    pub fn read() -> Result<Config> {
        let config = Config::from(&private::TOML_CONFIG);

        if config.server.electricity_price_api.is_empty() {
            bail!("Missing electricity price API configuration");
        }

        Ok(config)
    }
}

impl From<&private::TomlConfig> for WifiConfig {
    fn from(config: &private::TomlConfig) -> Self {
        WifiConfig {
            ssid: config.wifi_ssid,
            password: config.wifi_psk,
        }
    }
}

impl From<&private::TomlConfig> for ServerConfig {
    fn from(config: &private::TomlConfig) -> Self {
        ServerConfig {
            electricity_price_api: config.electricity_price_api,
            metrics_url: config.metrics_url,
            ntp_server: config.ntp_server,
        }
    }
}

impl From<&private::TomlConfig> for Config {
    fn from(config: &private::TomlConfig) -> Self {
        Config {
            measurement_interval: Duration::from_secs(config.measurement_interval),
            set_points: CoreConfig {
                minimum_temperature: Temperature::new(config.set_point_minimum_temperature),
                fallback_minimum_temperature: Temperature::new(
                    config.set_point_fallback_minimum_temperature,
                ),
                maximum_temperature: Temperature::new(config.set_point_maximum_temperature),
                turbo_temperature: Temperature::new(30.0),
                maximum_price: ElectricityPrice::new(config.set_point_maximum_price),
            },
            wifi: WifiConfig::from(config),
            server: ServerConfig::from(config),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // fixme, we should measure every few minutes at most
            measurement_interval: Duration::from_secs(1),
            set_points: CoreConfig {
                minimum_temperature: Temperature::new(15.0),
                fallback_minimum_temperature: Temperature::new(18.0),
                maximum_temperature: Temperature::new(22.0),
                turbo_temperature: Temperature::new(30.0),
                maximum_price: ElectricityPrice::new(0.30),
            },
            wifi: WifiConfig::default(),
            server: ServerConfig::default(),
        }
    }
}
