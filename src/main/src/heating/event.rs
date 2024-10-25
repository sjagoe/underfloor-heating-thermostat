use core::ffi::CStr;
use esp_idf_svc::eventloop::*;

unsafe impl EspEventSource for super::HeatingEvent {
    fn source() -> Option<&'static CStr> {
        // String should be unique across the whole project and ESP IDF
        Some(c"HEATING-ENABLE-EVENT")
    }
}

impl EspEventSerializer for super::HeatingEvent {
    type Data<'a> = super::HeatingEvent;

    fn serialize<F, R>(event: &Self::Data<'_>, f: F) -> R
    where
        F: FnOnce(&EspEventPostData) -> R,
    {
        // Go the easy way since our payload implements Copy and is `'static`
        f(&unsafe { EspEventPostData::new(Self::source().unwrap(), Self::event_id(), event) })
    }
}

impl EspEventDeserializer for super::HeatingEvent {
    type Data<'a> = super::HeatingEvent;

    fn deserialize<'a>(data: &EspEvent<'a>) -> Self::Data<'a> {
        // Just as easy as serializing
        *unsafe { data.as_payload::<super::HeatingEvent>() }
    }
}
