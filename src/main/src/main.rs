use anyhow::Result;
use core::time::Duration;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay,
        gpio::{AnyOutputPin, PinDriver},
        prelude::Peripherals,
    },
    timer::EspTaskTimerService,
};
use log::*;

mod heating;
mod measurement;
mod rgbled;
mod status;

use heating::{get_next_desired_state, HeatingEvent};
use measurement::{read_temperature, MeasurementEvent};
use rgbled::{RGB8, WS2812RMT};
use status::StatusEvent;

use control::{
    CoreConfig,
    ElectricityPrice,
    Temperature,
};

pub struct Config {
    measurement_interval: Duration,
    set_points: CoreConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // fixme, we should measure every few minutes at most
            measurement_interval: Duration::from_secs(1),
            set_points: CoreConfig {
                minimum_temperature: Temperature::new(15.0),
                maximum_temperature: Temperature::new(22.0),
                turbo_temperature: Temperature::new(30.0),
                maximum_price: ElectricityPrice::new(0.30),
            },
        }
    }
}


fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let thermistor_enable_pin: AnyOutputPin = peripherals.pins.gpio10.into();
    let mut thermistor_enable = PinDriver::output(thermistor_enable_pin)?;
    thermistor_enable.set_low()?;

    let heating_enable_pin: AnyOutputPin = peripherals.pins.gpio11.into();
    let mut heating_enable = PinDriver::output(heating_enable_pin)?;
    heating_enable.set_low()?;

    let mut led = WS2812RMT::new(peripherals.pins.gpio8, peripherals.rmt.channel0)?;
    led.set_pixel(RGB8::from(StatusEvent::Initializing))?;

    let config = Config::default();

    let sysloop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let callback_timer = {
        // Avoid move of sysloop into closure
        let localloop = sysloop.clone();
        timer_service.timer(move || {
            let sysloop = localloop.clone();
            sysloop.post::<StatusEvent>(&StatusEvent::Collecting, delay::BLOCK).expect("Failed to post status");
            info!("Measuring temperature");
            let temperature = read_temperature(&mut thermistor_enable).expect("Failed to read temperature");
            localloop.post::<MeasurementEvent>(&temperature, delay::BLOCK).unwrap();
        })?
    };

    let _measurement_handler = {
        // Avoid move of sysloop into closure
        let localloop = sysloop.clone();
        let set_points = config.set_points;
        sysloop.subscribe::<MeasurementEvent, _>(move |event| {
            localloop.post::<StatusEvent>(&StatusEvent::Ready, delay::BLOCK).expect("failed to post status");
            match event.value() {
                Ok(value) => {
                    info!("Received event {:?}: {:?}", event, value);
                    let price = ElectricityPrice::new(0.20);
                    let heating_event = get_next_desired_state(&set_points, value, price);
                    localloop.post::<HeatingEvent>(&heating_event, delay::BLOCK).expect("failed to post heating event");
                }
                Err(err) => error!("Received bad event {:?}: {:?}", event, err),
            }
        })?
    };

    let _heating_handler = {
        // Avoid move of sysloop into closure
        let _localloop = sysloop.clone();
        sysloop.subscribe::<HeatingEvent, _>(move |event| {
            info!("Received event {:?}", event);
        })?
    };

    let _status_handler = sysloop.subscribe::<StatusEvent, _>(move |event| {
        let colour = RGB8::from(event);
        led.set_pixel(colour).expect("Failed to set LED colour");
    })?;

    callback_timer.every(config.measurement_interval)?;

    loop {
        // fixme we should go into a low-power state until reacting to an event
        delay::FreeRtos::delay_ms(250);
    }
}
