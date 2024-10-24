use anyhow::Result;
use esp_idf_svc::{
    hal::{
        delay,
        gpio::AnyIOPin,
        i2c::{I2c, I2cConfig, I2cDriver},
        peripheral::Peripheral,
        units::Hertz,
    },
};

mod config;

pub use config::*;

pub struct ADS1015<'d> {
    i2c: I2cDriver<'d>,
}

impl<'d> ADS1015<'d> {
    pub fn from_peripheral(
        i2c: impl Peripheral<P = impl I2c> + 'd,
        sda: AnyIOPin,
        scl: AnyIOPin,
        baudrate: Hertz,
    ) -> Result<ADS1015<'d>> {
        let config = I2cConfig::new().baudrate(baudrate);
        let driver = I2cDriver::new(i2c, sda, scl, &config)?;
        let adc = ADS1015 {
            i2c: driver,
        };
        Ok(adc)
    }

    pub fn read(&mut self, config: &AdcConfig) -> Result<f32> {
        let config_bytes = config.to_u8_array(true);
        let write_buffer: [u8; 3] = [0b01, config_bytes[0], config_bytes[1]];
        log::warn!("ADC config {:08b} {:08b} {:08b}", write_buffer[0], write_buffer[1], write_buffer[2]);
        let _ = self.i2c.write(config.address, &write_buffer, delay::BLOCK)?;

        loop {
            let mut buf: [u8; 2] = [0; 2];
            let _ = self.i2c.write_read(config.address, &[0b01], &mut buf, delay::BLOCK)?;
            log::error!("read config {:08b} {:08b}", buf[0], buf[1]);
            if buf[0] >> 7 == 0b1 {
                buf = [0; 2];
                let _ = self.i2c.write_read(config.address, &[0b00], &mut buf, delay::BLOCK)?;
                let uvalue: u16 = (buf[0] as u16) << 4 | (buf[1] as u16) >> 4;
                log::error!("read value {:?}", uvalue);
                return Ok(config.gain.apply(uvalue));
            }
        }
    }
}