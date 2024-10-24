use anyhow::Result;
use esp_idf_svc::hal::{delay, i2c::I2cDriver};

mod config;

pub use config::*;

pub fn read(driver: &mut I2cDriver, config: &AdcConfig) -> Result<f32> {
    let config_bytes = config.to_u8_array(true);
    let write_buffer: [u8; 3] = [0b01, config_bytes[0], config_bytes[1]];
    driver.write(config.address, &write_buffer, delay::BLOCK)?;

    loop {
        let mut buf: [u8; 2] = [0; 2];
        driver.write_read(config.address, &[0b01], &mut buf, delay::BLOCK)?;
        if buf[0] >> 7 == 0b1 {
            buf = [0; 2];
            driver.write_read(config.address, &[0b00], &mut buf, delay::BLOCK)?;
            let uvalue: u16 = (buf[0] as u16) << 4 | (buf[1] as u16) >> 4;
            return Ok(config.gain.apply(uvalue));
        }
    }
}
