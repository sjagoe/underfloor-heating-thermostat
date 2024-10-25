mod event;

use control::{CoreConfig, ElectricityPrice, PowerState, SetPoint, Temperature};

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

pub fn get_next_desired_state(
    config: &CoreConfig,
    temperature: Temperature,
    price: ElectricityPrice,
) -> HeatingEvent {
    let next_state = SetPoint::from_current_state(config, temperature, price);

    HeatingEvent::from(next_state)
}
