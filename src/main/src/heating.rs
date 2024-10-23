mod event;

pub use event::HeatingEvent;

use control::{
    desired_state,
    CoreConfig,
    ElectricityPrice,
    Temperature,
};

pub fn get_next_desired_state(config: &CoreConfig, temperature: Temperature, price: ElectricityPrice) -> HeatingEvent {
    let next_state = desired_state(config, temperature, price);

    HeatingEvent::from(next_state)
}
