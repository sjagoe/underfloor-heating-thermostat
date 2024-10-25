use core::ffi::CStr;
use esp_idf_svc::eventloop::*;

unsafe impl EspEventSource for super::StatusEvent {
    fn source() -> Option<&'static CStr> {
        // String should be unique across the whole project and ESP IDF
        Some(c"STATUS-EVENT")
    }
}

impl EspEventSerializer for super::StatusEvent {
    type Data<'a> = super::StatusEvent;

    fn serialize<F, R>(event: &Self::Data<'_>, f: F) -> R
    where
        F: FnOnce(&EspEventPostData) -> R,
    {
        // Go the easy way since our payload implements Copy and is `'static`
        f(&unsafe { EspEventPostData::new(Self::source().unwrap(), Self::event_id(), event) })
    }
}

impl EspEventDeserializer for super::StatusEvent {
    type Data<'a> = super::StatusEvent;

    fn deserialize<'a>(data: &EspEvent<'a>) -> Self::Data<'a> {
        // Just as easy as serializing
        *unsafe { data.as_payload::<super::StatusEvent>() }
    }
}
