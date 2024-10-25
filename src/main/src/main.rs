use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay,
        gpio::{AnyOutputPin, PinDriver},
        prelude::{FromValueType, Peripherals},
    },
    sys::{esp, esp_wifi_connect},
    timer::EspTaskTimerService,
    wifi::{AuthMethod, WifiEvent},
};
use log::*;

mod adc;
mod config;
mod electricity_price;
mod heating;
mod http;
mod i2c;
mod measurement;
mod rgbled;
mod status;
mod trigger;
mod wifi;

use config::Config;
use heating::HeatingEvent;
use measurement::MeasurementEvent;
use rgbled::{RGB8, WS2812RMT};
use status::StatusEvent;
use trigger::TriggerEvent;

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

    let config = Config::read()?;

    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio6;
    let scl = peripherals.pins.gpio7;
    let shared_i2c_driver = i2c::init_i2c_driver(i2c, sda.into(), scl.into(), 100.kHz().into())?;

    let sysloop = EspSystemEventLoop::take()?;

    let shared_wifi = wifi::SharedWifi::connect_wifi(
        peripherals.modem,
        sysloop.clone(),
        None,
        AuthMethod::WPA2Personal,
        config.wifi.ssid,
        config.wifi.password,
    )?;

    let timer_service = EspTaskTimerService::new()?;
    let measurement_timer = {
        // Avoid move of sysloop into closure
        let localloop = sysloop.clone();
        timer_service.timer(move || {
            localloop
                .post::<TriggerEvent>(&TriggerEvent, delay::BLOCK)
                .expect("Failed to post trigger");
        })?
    };

    let _trigger_handler = {
        // Avoid move of sysloop into closure
        let localloop = sysloop.clone();
        let i2c_driver = shared_i2c_driver.clone();
        sysloop.subscribe::<TriggerEvent, _>(move |_| {
            localloop
                .post::<StatusEvent>(&StatusEvent::Measuring, delay::BLOCK)
                .expect("Failed to post status");
            let mut driver = i2c_driver.lock();
            let temperature =
                MeasurementEvent::take_temperature_reading(&mut thermistor_enable, &mut driver)
                    .expect("Failed to take temperature reading");
            localloop
                .post::<MeasurementEvent>(&temperature, delay::BLOCK)
                .expect("Failed to post measurement");
        })?
    };

    let _wifi_handler = {
        let _localloop = sysloop.clone();
        sysloop.subscribe::<WifiEvent, _>(move |event| match event {
            WifiEvent::StaDisconnected => {
                error!("Received STA Disconnected event {:?}", event);
                delay::FreeRtos::delay_ms(1000);
                // NOTE: calling the FFI binding directly to prevent casusing a move
                // on the the EspWifi instance.
                if let Err(err) = esp!(unsafe { esp_wifi_connect() }) {
                    info!("Error calling wifi.connect in wifi reconnect {:?}", err);
                }
            }
            _ => info!("Received other Wifi event: {:?}", event),
        })?
    };

    let _measurement_handler = {
        // Avoid move of sysloop into closure
        let localloop = sysloop.clone();
        let set_points = config.set_points;
        let price = config.fake_electricity_price;
        sysloop.subscribe::<MeasurementEvent, _>(move |event| {
            info!("Received event {:?}", event);
            event
                .handle(&localloop, &set_points, price)
                .expect("Failed to handle measurement event");
        })?
    };

    let _heating_handler = {
        // Avoid move of sysloop into closure
        let localloop = sysloop.clone();
        sysloop.subscribe::<HeatingEvent, _>(move |event| {
            info!("Received event {:?}", event);
            let power_state = event
                .switch_heating(&mut heating_enable)
                .expect("Failed to switch heating");
            let status = StatusEvent::from(power_state);
            localloop
                .post::<StatusEvent>(&status, delay::BLOCK)
                .expect("Failed to post status event");
        })?
    };

    let _status_handler = sysloop.subscribe::<StatusEvent, _>(move |event| {
        let colour = RGB8::from(event);
        led.set_pixel(colour).expect("Failed to set LED colour");
    })?;

    shared_wifi.wait_for_connected()?;

    let _hourly_electricity_price =
        electricity_price::HourlyElectricityPrice::fetch(config.server.electricity_price_api);

    measurement_timer.every(config.measurement_interval)?;

    // Run the first evaluation immediately
    sysloop.post::<TriggerEvent>(&TriggerEvent, delay::BLOCK)?;

    loop {
        // fixme we should go into a low-power state until reacting to an event
        delay::FreeRtos::delay_ms(250);
    }
}
