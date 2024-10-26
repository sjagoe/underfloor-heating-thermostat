use rgb::RGB8;

mod event;

use crate::heating::HeatingPower;

#[derive(Debug, Clone, Copy)]
pub enum StatusEvent {
    Initializing,
    MissingData,
    Ready,
    Measuring,
    HeatingOn,
}

impl From<StatusEvent> for RGB8 {
    fn from(status: StatusEvent) -> RGB8 {
        match status {
            StatusEvent::Initializing => RGB8::new(10, 10, 0),
            StatusEvent::MissingData => RGB8::new(10, 0, 10),
            StatusEvent::Ready => RGB8::new(0, 10, 0),
            StatusEvent::Measuring => RGB8::new(0, 0, 10),
            StatusEvent::HeatingOn => RGB8::new(10, 0, 0),
        }
    }
}

impl From<HeatingPower> for StatusEvent {
    fn from(power: HeatingPower) -> StatusEvent {
        match power {
            HeatingPower::TurnOn => StatusEvent::HeatingOn,
            HeatingPower::TurnOff => StatusEvent::Ready,
        }
    }
}
