use anyhow::Result;
use esp_idf_svc::hal::{
    gpio::AnyIOPin,
    i2c::{I2c, I2cConfig, I2cDriver},
    peripheral::Peripheral,
    units::Hertz,
};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SharedI2cDriver<'d> {
    pub driver: Arc<Mutex<I2cDriver<'d>>>,
}

pub fn init_i2c_driver<'d>(
    i2c: impl Peripheral<P = impl I2c> + 'd,
    sda: AnyIOPin,
    scl: AnyIOPin,
    baudrate: Hertz,
) -> Result<SharedI2cDriver<'d>> {
    let config = I2cConfig::new().baudrate(baudrate);
    let driver = I2cDriver::new(i2c, sda, scl, &config)?;
    let shared_driver = SharedI2cDriver {
        driver: Arc::new(Mutex::new(driver)),
    };
    Ok(shared_driver)
}
