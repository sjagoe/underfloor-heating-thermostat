use anyhow::Result;
use core::time::Duration;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        adc::{attenuation::DB_11, oneshot::config::AdcChannelConfig, oneshot::*},
        delay,
        gpio::{AnyOutputPin, PinDriver},
        peripheral::Peripheral,
        prelude::{FromValueType, Peripherals},
    },
    timer::EspTaskTimerService,
};
use log::*;

mod adc;
mod heating;
mod measurement;
mod rgbled;
mod status;

use heating::{get_next_desired_state, HeatingEvent};
use measurement::{read_temperature, MeasurementEvent};
use rgbled::{RGB8, WS2812RMT};
use status::StatusEvent;

use control::{CoreConfig, ElectricityPrice, Temperature};
use control::temperature_from_voltage;

pub struct Config {
    measurement_interval: Duration,
    fake_electricity_price: ElectricityPrice,
    set_points: CoreConfig,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // fixme, we should measure every few minutes at most
            measurement_interval: Duration::from_secs(1),
            fake_electricity_price: ElectricityPrice::new(0.20),
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

    let mut peripherals = Peripherals::take().unwrap();

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
            sysloop
                .post::<StatusEvent>(&StatusEvent::Collecting, delay::BLOCK)
                .expect("Failed to post status");
            info!("Measuring temperature");

            // Only way I could find to use the ADC in this closure is
            // to use unsafe clone
            let adc_peripheral = unsafe { peripherals.adc1.clone_unchecked() };
            let gpio2 = unsafe { peripherals.pins.gpio2.clone_unchecked() };

            let adc = AdcDriver::new(adc_peripheral).expect("fuck");
            let adc_config = AdcChannelConfig {
                attenuation: DB_11, // Allegedly a range up to 3.6V
                calibration: true,
                ..Default::default()
            };
            let mut adc_pin = AdcChannelDriver::new(&adc, gpio2, &adc_config).expect("fuck");

            let value = adc.read(&mut adc_pin).expect("Failed to read from adc");
            warn!("read adc value {:?} -> {:?}", value, f32::from(value));

            let temperature = read_temperature(&mut thermistor_enable, value.into())
                .expect("Failed to read temperature");
            localloop
                .post::<MeasurementEvent>(&temperature, delay::BLOCK)
                .unwrap();
        })?
    };

    let _measurement_handler = {
        // Avoid move of sysloop into closure
        let localloop = sysloop.clone();
        let set_points = config.set_points;
        let price = config.fake_electricity_price;
        sysloop.subscribe::<MeasurementEvent, _>(move |event| {
            localloop
                .post::<StatusEvent>(&StatusEvent::Ready, delay::BLOCK)
                .expect("failed to post status");
            match event.value() {
                Ok(value) => {
                    info!("Received event {:?}: {:?}", event, value);
                    let heating_event = get_next_desired_state(&set_points, value, price);
                    localloop
                        .post::<HeatingEvent>(&heating_event, delay::BLOCK)
                        .expect("failed to post heating event");
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

    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio6;
    let scl = peripherals.pins.gpio7;
    let mut adc = adc::ADS1015::from_peripheral(i2c, sda.into(), scl.into(), 100.kHz().into())?;

    let adc_config = adc::AdcConfig {
        input: adc::AnalogInput::SingleEndedAni0,
        gain: adc::Gain::Full,
        mode: adc::Mode::SingleShot,
        ..adc::AdcConfig::default()
    };

    let reference = adc.read(&adc_config)?;

    info!("Reference value: {:?}", reference);

    loop {
        // fixme we should go into a low-power state until reacting to an event
        delay::FreeRtos::delay_ms(250);

        let adc_config = adc::AdcConfig {
            input: adc::AnalogInput::SingleEndedAni1,
            gain: adc::Gain::Full,
            mode: adc::Mode::SingleShot,
            ..adc::AdcConfig::default()
        };

        let value = adc.read(&adc_config)?;
        warn!("newRead adc value {:?} ({:?}%)", value, value / reference);
        let temperature = temperature_from_voltage(reference, value);
        warn!("newTemperature: {:?}", temperature);
    }
}
