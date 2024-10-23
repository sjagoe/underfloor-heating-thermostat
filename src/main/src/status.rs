use core::ffi::CStr;
use esp_idf_svc::eventloop::*;
use rgb::RGB8;

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Initializing,
    Ready,
    Collecting,
}

impl From<Status> for RGB8 {
    fn from(status: Status) -> RGB8 {
        match status {
            Status::Initializing => RGB8::new(10, 10, 0),
            Status::Ready => RGB8::new(0, 10, 0),
            Status::Collecting => RGB8::new(0, 0, 10),
        }
    }
}

unsafe impl EspEventSource for Status {
    fn source() -> Option<&'static CStr> {
        // String should be unique across the whole project and ESP IDF
        Some(CStr::from_bytes_with_nul(b"DEMO-SERVICE\0").unwrap())
    }
}

impl EspEventSerializer for Status {
    type Data<'a> = Status;

    fn serialize<F, R>(event: &Self::Data<'_>, f: F) -> R
    where
        F: FnOnce(&EspEventPostData) -> R,
    {
        // Go the easy way since our payload implements Copy and is `'static`
        f(&unsafe { EspEventPostData::new(Self::source().unwrap(), Self::event_id(), event) })
    }
}

impl EspEventDeserializer for Status {
    type Data<'a> = Status;

    fn deserialize<'a>(data: &EspEvent<'a>) -> Self::Data<'a> {
        // Just as easy as serializing
        *unsafe { data.as_payload::<Status>() }
    }
}
