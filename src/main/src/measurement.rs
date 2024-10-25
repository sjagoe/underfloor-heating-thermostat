use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay,
        gpio::{AnyOutputPin, Output, PinDriver},
        i2c::I2cDriver,
    },
};
use log::*;

use control::{temperature_from_voltage, CoreConfig, ElectricityPrice, Temperature};

mod event;

use crate::adc;
use crate::heating::{get_next_desired_state, HeatingEvent};
use crate::StatusEvent;

#[derive(Copy, Clone, Debug)]
pub enum MeasurementEvent {
    Measurement(Temperature),
}

impl MeasurementEvent {
    pub fn value(self) -> Result<Temperature> {
        match self {
            MeasurementEvent::Measurement(value) => Ok(value),
        }
    }

    pub fn handle(
        self,
        sysloop: &EspSystemEventLoop,
        set_points: &CoreConfig,
        price: ElectricityPrice,
    ) -> Result<()> {
        sysloop.post::<StatusEvent>(&StatusEvent::Ready, delay::BLOCK)?;
        match self.value() {
            Ok(value) => {
                let heating_event = get_next_desired_state(&set_points, value, price);
                sysloop.post::<HeatingEvent>(&heating_event, delay::BLOCK)?;
            }
            Err(err) => error!("Received bad event {:?}: {:?}", self, err),
        }
        Ok(())
    }
}

pub fn read_temperature(
    enable: &mut PinDriver<AnyOutputPin, Output>,
    i2c_driver: &mut I2cDriver,
) -> Result<MeasurementEvent> {
    let adc_config = adc::AdcConfig {
        input: adc::AnalogInput::SingleEndedAni0,
        gain: adc::Gain::Full,
        mode: adc::Mode::SingleShot,
        ..adc::AdcConfig::default()
    };
    let reference_voltage = adc::read(i2c_driver, &adc_config)?;

    let _ = enable.set_high().inspect_err(|_| {
        enable.set_low().expect("Unable to enable thermistor");
    });

    // Let current through the thermistor settle
    delay::FreeRtos::delay_ms(100);

    let adc_config = adc::AdcConfig {
        input: adc::AnalogInput::SingleEndedAni1,
        gain: adc::Gain::Full,
        mode: adc::Mode::SingleShot,
        ..adc::AdcConfig::default()
    };

    let result = adc::read(i2c_driver, &adc_config);
    // Disable current through thermistor before checking error
    enable.set_low()?;

    let thermistor_voltage = result?;

    let temperature = temperature_from_voltage(reference_voltage, thermistor_voltage);
    let event = MeasurementEvent::Measurement(Temperature::new(temperature));

    Ok(event)
}
