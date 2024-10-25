use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay,
        gpio::{AnyIOPin, AnyOutputPin, Output, PinDriver},
        i2c::{I2c, I2cConfig, I2cDriver},
        peripheral::Peripheral,
        units::Hertz,
    },
};

mod event;

use crate::read_temperature;
use crate::MeasurementEvent;
use crate::StatusEvent;

pub fn init_i2c_driver<'d>(
    i2c: impl Peripheral<P = impl I2c> + 'd,
    sda: AnyIOPin,
    scl: AnyIOPin,
    baudrate: Hertz,
) -> Result<I2cDriver<'d>> {
    let config = I2cConfig::new().baudrate(baudrate);
    let driver = I2cDriver::new(i2c, sda, scl, &config)?;
    Ok(driver)
}

#[derive(Copy, Clone, Debug)]
pub enum I2CEventType {
    ReadTemperature,
}

#[derive(Copy, Clone, Debug)]
pub struct I2CEvent {
    pub event_type: I2CEventType,
}

impl I2CEvent {
    pub fn handle(
        self,
        sysloop: &EspSystemEventLoop,
        thermistor_enable: &mut PinDriver<AnyOutputPin, Output>,
        i2c_driver: &mut I2cDriver,
    ) -> Result<()> {
        match self.event_type {
            I2CEventType::ReadTemperature => {
                sysloop.post::<StatusEvent>(&StatusEvent::Collecting, delay::BLOCK)?;
                let temperature = read_temperature(thermistor_enable, i2c_driver)?;
                sysloop.post::<MeasurementEvent>(&temperature, delay::BLOCK)?;
            }
        }
        Ok(())
    }
}
