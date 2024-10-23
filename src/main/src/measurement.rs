use anyhow::Result;
use esp_idf_svc::hal::{
    delay,
    gpio::{AnyOutputPin, Output, PinDriver},
};

use control::{temperature_from_voltage, Temperature};

mod event;

pub use event::MeasurementEvent;

pub fn with_thermistor(
    enable: &mut PinDriver<AnyOutputPin, Output>,
    read: impl Fn() -> Result<f32>,
) -> Result<f32> {
    let _ = enable
        .set_high()
         .inspect_err(|_| {
             enable.set_low().expect("Unable to enable thermistor");
         });

    // Let current through the thermistor settle
    delay::FreeRtos::delay_ms(100);

    let result = read();

    enable.set_low()?;

    result
}

pub fn read_temperature(
    enable: &mut PinDriver<AnyOutputPin, Output>,
    fake_voltage: f32,
) -> Result<MeasurementEvent> {
    let thermistor_voltage = with_thermistor(enable, || Ok(fake_voltage))?;

    // fixme
    let adc_reference_voltage = 3300.0;

    let temperature = temperature_from_voltage(adc_reference_voltage, thermistor_voltage);
    let event = MeasurementEvent::Measurement(Temperature::new(temperature));

    Ok(event)
}
