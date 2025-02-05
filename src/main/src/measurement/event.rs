use core::ffi::CStr;
use esp_idf_svc::eventloop::*;

unsafe impl EspEventSource for super::MeasurementEvent {
    fn source() -> Option<&'static CStr> {
        // String should be unique across the whole project and ESP IDF
        Some(c"NTC-MEASUREMENT")
    }
}

impl EspEventSerializer for super::MeasurementEvent {
    type Data<'a> = super::MeasurementEvent;

    fn serialize<F, R>(event: &Self::Data<'_>, f: F) -> R
    where
        F: FnOnce(&EspEventPostData) -> R,
    {
        // Go the easy way since our payload implements Copy and is `'static`
        f(&unsafe { EspEventPostData::new(Self::source().unwrap(), Self::event_id(), event) })
    }
}

impl EspEventDeserializer for super::MeasurementEvent {
    type Data<'a> = super::MeasurementEvent;

    fn deserialize<'a>(data: &EspEvent<'a>) -> Self::Data<'a> {
        // Just as easy as serializing
        *unsafe { data.as_payload::<super::MeasurementEvent>() }
    }
}
