use rgb::RGB8;

mod event;

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
