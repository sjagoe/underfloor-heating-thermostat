use core::ffi::CStr;
use esp_idf_svc::eventloop::*;

#[derive(Copy, Clone, Debug)]
pub enum I2CEventType {
    ReadTemperature,
}

#[derive(Copy, Clone, Debug)]
pub struct I2CEvent {
    pub event_type: I2CEventType,
}

unsafe impl EspEventSource for I2CEvent {
    fn source() -> Option<&'static CStr> {
        // String should be unique across the whole project and ESP IDF
        Some(c"I2C-REQUEST-EVENT")
    }
}

impl EspEventSerializer for I2CEvent {
    type Data<'a> = I2CEvent;

    fn serialize<F, R>(event: &Self::Data<'_>, f: F) -> R
    where
        F: FnOnce(&EspEventPostData) -> R,
    {
        // Go the easy way since our payload implements Copy and is `'static`
        f(&unsafe { EspEventPostData::new(Self::source().unwrap(), Self::event_id(), event) })
    }
}

impl EspEventDeserializer for I2CEvent {
    type Data<'a> = I2CEvent;

    fn deserialize<'a>(data: &EspEvent<'a>) -> Self::Data<'a> {
        // Just as easy as serializing
        *unsafe { data.as_payload::<I2CEvent>() }
    }
}
