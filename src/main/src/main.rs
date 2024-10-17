#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{delay::Delay, prelude::*};

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();

    esp_println::logger::init_logger_from_env();

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
let _init = esp_wifi::init(
    esp_wifi::EspWifiInitFor::Wifi,
    timg0.timer0,
    esp_hal::rng::Rng::new(peripherals.RNG),
    peripherals.RADIO_CLK,
)
.unwrap();

    loop {
        log::info!("Hello world!");
        delay.delay(500.millis());
    }
}
