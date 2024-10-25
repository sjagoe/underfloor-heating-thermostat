#[toml_cfg::toml_config]
pub struct TomlConfig {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default(300)]
    measurement_interval: u64,
    #[default(18.0)]
    set_point_minimum_temperature: f32,
    #[default(25.0)]
    set_point_maximum_temperature: f32,
    #[default(0.15)]
    set_point_maximum_price: f32,
}
