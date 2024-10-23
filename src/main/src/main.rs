use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay::FreeRtos,
        gpio::{AnyOutputPin, PinDriver},
        prelude::Peripherals,
    },
};

mod measurement;
mod rgbled;

use rgbled::{RGB8, WS2812RMT};

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    // Start the LED off yellow
    let mut led = WS2812RMT::new(peripherals.pins.gpio8, peripherals.rmt.channel0)?;
    led.set_pixel(RGB8::new(10, 10, 0))?;

    let thermistor_enable_pin: AnyOutputPin = peripherals.pins.gpio10.into();
    let mut thermistor_enable = PinDriver::output(thermistor_enable_pin)?;
    thermistor_enable.set_low()?;

    let heating_enable_pin: AnyOutputPin = peripherals.pins.gpio11.into();
    let mut heating_enable = PinDriver::output(heating_enable_pin)?;
    heating_enable.set_low()?;

    let sysloop = EspSystemEventLoop::take()?;

    loop {
        led.set_pixel(RGB8::new(0, 0, 10))?;
        FreeRtos::delay_ms(250);
        let _temperature = read_temperature(&mut thermistor_enable);
        led.set_pixel(RGB8::new(0, 10, 0))?;
        FreeRtos::delay_ms(250);
    }
}
