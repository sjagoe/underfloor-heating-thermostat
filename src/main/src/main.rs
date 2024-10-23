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

use heating::HeatingEvent;
use measurement::{read_temperature, MeasurementEvent};
use rgbled::{RGB8, WS2812RMT};
use status::Status;

pub struct Config {
    measurement_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // fixme, we should measure every few minutes at most
            measurement_interval: Duration::from_secs(1),
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
    led.set_pixel(RGB8::from(Status::Initializing))?;

    let sysloop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let callback_timer = {
        // Avoid move of sysloop into closure
        let localloop = sysloop.clone();
        timer_service.timer(move || {
            let sysloop = localloop.clone();
            sysloop.post::<Status>(&Status::Collecting, delay::BLOCK).expect("Failed to post status");
            info!("Measuring temperature");
            let _temperature = read_temperature(&sysloop, &mut thermistor_enable);
        })?
    };

    let _measurement_handler = {
        // Avoid move of sysloop into closure
        let localloop = sysloop.clone();
        sysloop.subscribe::<MeasurementEvent, _>(move |event| {
            localloop.post::<Status>(&Status::Ready, delay::BLOCK).expect("failed to post status");
            match event.value() {
                Ok(value) => info!("Received event {:?}: {:?}", event, value),
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

    let _status_handler = sysloop.subscribe::<Status, _>(move |event| {
        let colour = RGB8::from(event);
        led.set_pixel(colour).expect("Failed to set LED colour");
    })?;

    let config = Config::default();

    callback_timer.every(config.measurement_interval)?;

    loop {
        // fixme we should go into a low-power state until reacting to an event
        delay::FreeRtos::delay_ms(250);
    }
}
