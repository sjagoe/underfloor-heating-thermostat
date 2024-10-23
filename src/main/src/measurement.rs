use anyhow::Result;
use core::ffi::CStr;
use esp_idf_svc::{
    eventloop::*,
    hal::{
        delay,
        gpio::{
            AnyOutputPin,
            Output,
            PinDriver,
        },
    },
};

use control::temperature_from_voltage;

#[derive(Copy, Clone, Debug)]
pub enum MeasurementEvent {
    Measurement(f32),
}

impl MeasurementEvent {
    pub fn value(&self) -> Result<f32> {
        match self {
            MeasurementEvent::Measurement(value) => Ok(*value),
        }
    }
}

unsafe impl EspEventSource for MeasurementEvent {
    fn source() -> Option<&'static CStr> {
        // String should be unique across the whole project and ESP IDF
        Some(CStr::from_bytes_with_nul(b"NTC-MEASUREMENT\0").unwrap())
    }
}

impl EspEventSerializer for MeasurementEvent {
    type Data<'a> = MeasurementEvent;

    fn serialize<F, R>(event: &Self::Data<'_>, f: F) -> R
    where
        F: FnOnce(&EspEventPostData) -> R,
    {
        // Go the easy way since our payload implements Copy and is `'static`
        f(&unsafe { EspEventPostData::new(Self::source().unwrap(), Self::event_id(), event) })
    }
}

impl EspEventDeserializer for MeasurementEvent {
    type Data<'a> = MeasurementEvent;

    fn deserialize<'a>(data: &EspEvent<'a>) -> Self::Data<'a> {
        // Just as easy as serializing
        *unsafe { data.as_payload::<MeasurementEvent>() }
    }
}

pub fn with_thermistor(enable: &mut PinDriver<AnyOutputPin, Output>, read: fn() -> Result<f32>) -> Result<f32> {
    enable.set_high().or_else(|err| {
        enable.set_low().expect("Unable to enable thermistor");
        Err(err)
    }).expect("Unable to enable thermistor");

    // Let current through the thermistor settle
    delay::FreeRtos::delay_ms(100);

    let result = read();

    enable.set_low()?;

    match result {
        Ok(voltage) => Ok(voltage),
        Err(e) => Err(e),
    }
}

pub fn read_temperature(sysloop: &EspSystemEventLoop, enable: &mut PinDriver<AnyOutputPin, Output>) -> Result<()> {
    let thermistor_voltage = with_thermistor(enable, || {
        Ok(1500.0)
    })?;

    // fixme
    let adc_reference_voltage = 4000.0;

    let temperature = temperature_from_voltage(adc_reference_voltage, thermistor_voltage);
    sysloop.post::<MeasurementEvent>(&MeasurementEvent::Measurement(temperature), delay::BLOCK).unwrap();

    Ok(())
}
