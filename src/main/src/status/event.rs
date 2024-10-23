use core::ffi::CStr;
use esp_idf_svc::eventloop::*;
use rgb::RGB8;

#[derive(Debug, Clone, Copy)]
pub enum StatusEvent {
    Initializing,
    Ready,
    Collecting,
}

impl From<StatusEvent> for RGB8 {
    fn from(status: StatusEvent) -> RGB8 {
        match status {
            StatusEvent::Initializing => RGB8::new(10, 10, 0),
            StatusEvent::Ready => RGB8::new(0, 10, 0),
            StatusEvent::Collecting => RGB8::new(0, 0, 10),
        }
    }
}

unsafe impl EspEventSource for StatusEvent {
    fn source() -> Option<&'static CStr> {
        // String should be unique across the whole project and ESP IDF
        Some(c"STATUS-EVENT")
    }
}

impl EspEventSerializer for StatusEvent {
    type Data<'a> = StatusEvent;

    fn serialize<F, R>(event: &Self::Data<'_>, f: F) -> R
    where
        F: FnOnce(&EspEventPostData) -> R,
    {
        // Go the easy way since our payload implements Copy and is `'static`
        f(&unsafe { EspEventPostData::new(Self::source().unwrap(), Self::event_id(), event) })
    }
}

impl EspEventDeserializer for StatusEvent {
    type Data<'a> = StatusEvent;

    fn deserialize<'a>(data: &EspEvent<'a>) -> Self::Data<'a> {
        // Just as easy as serializing
        *unsafe { data.as_payload::<StatusEvent>() }
    }
}
