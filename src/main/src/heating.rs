mod event;

pub use event::HeatingEvent;

use control::{CoreConfig, ElectricityPrice, SetPoint, Temperature};

pub fn get_next_desired_state(
    config: &CoreConfig,
    temperature: Temperature,
    price: ElectricityPrice,
) -> HeatingEvent {
    let next_state = SetPoint::from_current_state(config, temperature, price);

    HeatingEvent::from(next_state)
}
