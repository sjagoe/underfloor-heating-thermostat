use anyhow::Result;
use esp_idf_svc::hal::{
    delay,
    gpio::{AnyOutputPin, Output, PinDriver},
};

use control::{temperature_from_voltage, Temperature};

mod event;

pub use event::MeasurementEvent;

use crate::adc;

pub fn read_temperature(
    enable: &mut PinDriver<AnyOutputPin, Output>,
    adc: &mut adc::ADS1015,
) -> Result<MeasurementEvent> {
    let adc_config = adc::AdcConfig {
        input: adc::AnalogInput::SingleEndedAni0,
        gain: adc::Gain::Full,
        mode: adc::Mode::SingleShot,
        ..adc::AdcConfig::default()
    };
    let reference_voltage = adc.read(&adc_config)?;

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

    let result = adc.read(&adc_config);
    // Disable current through thermistor before checking error
    enable.set_low()?;

    let thermistor_voltage = result?;

    let temperature = temperature_from_voltage(reference_voltage, thermistor_voltage);
    let event = MeasurementEvent::Measurement(Temperature::new(temperature));

    Ok(event)
}
