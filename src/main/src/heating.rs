use anyhow::Result;
use esp_idf_svc::hal::gpio::{AnyOutputPin, Output, PinDriver};
use log::*;

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

impl HeatingEvent {
    pub fn switch_heating(
        self,
        enable: &mut PinDriver<AnyOutputPin, Output>,
    ) -> Result<HeatingPower> {
        match (self.power, enable.is_set_high()) {
            (HeatingPower::TurnOn, false) => {
                info!("Turning on heating output");
                enable.set_high()?;
            }
            (HeatingPower::TurnOff, true) => {
                info!("Turning off heating output");
                enable.set_low()?;
            }
            (desired_state, _) => {
                info!("Heating is already in desired state {:?}", desired_state);
            }
        }
        Ok(self.power)
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
