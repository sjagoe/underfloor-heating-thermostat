use core::ffi::CStr;
use esp_idf_svc::eventloop::*;

use control::{PowerState, SetPoint, Temperature};

#[derive(Debug, Clone, Copy)]
pub enum HeatingPower {
    TurnOn,
    TurnOff,
}

#[derive(Debug, Clone, Copy)]
pub struct HeatingEvent {
    #[allow(dead_code)]
    power: HeatingPower,
    #[allow(dead_code)]
    temperature: Temperature,
}

impl From<PowerState> for HeatingPower {
    fn from(power: PowerState) -> HeatingPower {
        match power {
            PowerState::On => HeatingPower::TurnOn,
            PowerState::Off => HeatingPower::TurnOff,
        }
    }
}

impl From<SetPoint> for HeatingEvent {
    fn from(state: SetPoint) -> HeatingEvent {
        HeatingEvent {
            power: HeatingPower::from(state.power),
            temperature: state.temperature,
        }
    }
}

unsafe impl EspEventSource for HeatingEvent {
    fn source() -> Option<&'static CStr> {
        // String should be unique across the whole project and ESP IDF
        Some(CStr::from_bytes_with_nul(b"HEATING-ENABLE-EVENT\0").unwrap())
    }
}

impl EspEventSerializer for HeatingEvent {
    type Data<'a> = HeatingEvent;

    fn serialize<F, R>(event: &Self::Data<'_>, f: F) -> R
    where
        F: FnOnce(&EspEventPostData) -> R,
    {
        // Go the easy way since our payload implements Copy and is `'static`
        f(&unsafe { EspEventPostData::new(Self::source().unwrap(), Self::event_id(), event) })
    }
}

impl EspEventDeserializer for HeatingEvent {
    type Data<'a> = HeatingEvent;

    fn deserialize<'a>(data: &EspEvent<'a>) -> Self::Data<'a> {
        // Just as easy as serializing
        *unsafe { data.as_payload::<HeatingEvent>() }
    }
}
