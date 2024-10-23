use anyhow::Result;
use core::ffi::CStr;
use esp_idf_svc::eventloop::*;

use control::Temperature;

#[derive(Copy, Clone, Debug)]
pub enum MeasurementEvent {
    Measurement(Temperature),
}

impl MeasurementEvent {
    pub fn value(&self) -> Result<Temperature> {
        match self {
            MeasurementEvent::Measurement(value) => Ok(*value),
        }
    }
}

unsafe impl EspEventSource for MeasurementEvent {
    fn source() -> Option<&'static CStr> {
        // String should be unique across the whole project and ESP IDF
        Some(c"NTC-MEASUREMENT")
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
