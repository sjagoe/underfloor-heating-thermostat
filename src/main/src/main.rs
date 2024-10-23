use anyhow::Result;
use core::time::Duration;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay::FreeRtos,
        gpio::{AnyOutputPin, PinDriver},
        prelude::Peripherals,
    },
    timer::EspTaskTimerService,
};
use log::*;

mod measurement;
mod rgbled;

use measurement::{read_temperature, MeasurementEvent};
use rgbled::{RGB8, WS2812RMT};

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
    led.set_pixel(RGB8::new(10, 10, 0))?;

    let sysloop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let callback_timer = {
        // Avoid move of sysloop into closure
        let localloop = sysloop.clone();
        timer_service.timer(move || {
            info!("Measuring temperature");
            let sysloop = localloop.clone();
            let _temperature = read_temperature(&sysloop, &mut thermistor_enable);
        })?
    };
    callback_timer.every(Duration::from_secs(1))?;

    let _sub = sysloop.subscribe::<MeasurementEvent, _>(|event| {
        match event.value() {
            Ok(value) => info!("Received event {:?}: {:?}", event, value),
            Err(err) => error!("Received bad event {:?}: {:?}", event, err),
        }
    })?;

    loop {
        led.set_pixel(RGB8::new(0, 0, 10))?;
        FreeRtos::delay_ms(250);
        led.set_pixel(RGB8::new(0, 10, 0))?;
        FreeRtos::delay_ms(250);
    }
}
