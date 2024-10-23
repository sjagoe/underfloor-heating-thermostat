use anyhow::Result;
use esp_idf_svc::hal::gpio::{
    AnyOutputPin,
    Output,
    PinDriver,
};

use control::thermistor::temperature_from_voltage;

pub fn with_thermistor(enable: &mut PinDriver<AnyOutputPin, Output>, read: fn() -> Result<f32>) -> Result<f32> {
    read()
}

pub fn read_temperature(enable: &mut PinDriver<AnyOutputPin, Output>) -> Result<f32> {
    let thermistor_voltage = with_thermistor(enable, || {
        Ok(0.0)
    })?;

    // fixme
    let adc_reference_voltage = 1.0;

    let temperature = temperature_from_voltage(adc_reference_voltage, thermistor_voltage);

    Ok(temperature)
}
