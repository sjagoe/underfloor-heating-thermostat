use anyhow::Result;
use esp_idf_svc::{
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

#[derive(Copy, Clone, Debug)]
pub enum MeasurementEvent {
    Measurement(Temperature),
}

impl MeasurementEvent {
    pub fn take_temperature_reading(
        enable: &mut PinDriver<AnyOutputPin, Output>,
        i2c_driver: &mut I2cDriver,
    ) -> Result<MeasurementEvent> {
        let reference_adc_config = adc::AdcConfig {
            input: adc::AnalogInput::SingleEndedAni0,
            gain: adc::Gain::Full,
            mode: adc::Mode::SingleShot,
            ..adc::AdcConfig::default()
        };
        let reference_voltage = adc::read(i2c_driver, &reference_adc_config)?;

        let _ = enable.set_high().inspect_err(|_| {
            enable.set_low().expect("Unable to enable thermistor");
        });

        // Let current through the thermistor settle
        delay::FreeRtos::delay_ms(100);

        let adc_config = adc::AdcConfig {
            input: adc::AnalogInput::SingleEndedAni1,
            ..reference_adc_config
        };

        let result = adc::read(i2c_driver, &adc_config);
        // Disable current through thermistor before checking error
        enable.set_low()?;

        let thermistor_voltage = result?;

        let temperature = temperature_from_voltage(reference_voltage, thermistor_voltage);
        let event = MeasurementEvent::Measurement(Temperature::new(temperature));

        Ok(event)
    }

    pub fn value(self) -> Result<Temperature> {
        match self {
            MeasurementEvent::Measurement(value) => Ok(value),
        }
    }

    pub fn handle(
        self,
        set_points: &CoreConfig,
        price: Option<ElectricityPrice>,
    ) -> Option<HeatingEvent> {
        match self.value() {
            Ok(value) => {
                return Some(get_next_desired_state(set_points, value, price));
            }
            Err(err) => error!("Received bad event {:?}: {:?}", self, err),
        }
        None
    }
}
