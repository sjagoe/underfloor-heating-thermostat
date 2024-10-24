use anyhow::Result;
use core::time::Duration;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        adc::{attenuation::DB_11, oneshot::config::AdcChannelConfig, oneshot::*},
        delay,
        gpio::{AnyOutputPin, PinDriver},
        i2c,
        peripheral::Peripheral,
        prelude::{FromValueType, Peripherals},
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

use control::{CoreConfig, ElectricityPrice, Temperature};

pub struct Config {
    measurement_interval: Duration,
    fake_electricity_price: ElectricityPrice,
    set_points: CoreConfig,
}

// Default address of TI ADS1015/ADS1115
const ADC_ADDR: u8 = 0b1001000;

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

    let config = i2c::I2cConfig::new().baudrate(100.kHz().into());
    let mut i2c = i2c::I2cDriver::new(i2c, sda, scl, &config)?;

    // Configure ADS1015
    let config_high: u8 = 0b1_101_000_1;
        // 0b1 << 7 // Start a conversion
        // | 0b100 << 4 // single-ended measurement from AIN0
        // | 0b000 << 1 // +/- 6.144V range
        // | 0b1; // single-shot mode
    let config_low: u8 = 0b100_0_0_0_00;
        // 0b100 << 5 // 1600 SPS
        // | 0b0 << 4 // Comparator mode traditional
        // | 0b0 << 3 // Comparator polarity
        // | 0b0 << 2 // Comparator latching
        // | 0b00; // Comparator alert after one conversion

    let mut buf: [u8; 2] = [0; 2];
    let _ = i2c.write_read(ADC_ADDR, &[0b11], &mut buf, delay::BLOCK)?;
    warn!("hi_thresh {:04x}", (buf[0] as u16) << 8 | (buf[1] as u16));

    let _ = i2c.write(ADC_ADDR, &[0b11], delay::BLOCK).expect("write hi_thresh address");
    let _ = i2c.write(ADC_ADDR, &[0b10000000, 0b0], delay::BLOCK).expect("write hi_thresh");

    let mut buf: [u8; 2] = [0; 2];
    let _ = i2c.write_read(ADC_ADDR, &[0b10], &mut buf, delay::BLOCK)?;
    warn!("lo_thresh {:04x}", (buf[0] as u16) << 8 | (buf[1] as u16));

    let _ = i2c.write(ADC_ADDR, &[0b10], delay::BLOCK).expect("write lo_thresh address");
    let _ = i2c.write(ADC_ADDR, &[0b0, 0b0], delay::BLOCK).expect("write lo_thresh");

    let mut buf: [u8; 2] = [0; 2];
    let _ = i2c.write_read(ADC_ADDR, &[0b10], &mut buf, delay::BLOCK)?;
    warn!("lo_thresh {:04x}", (buf[0] as u16) << 8 | (buf[1] as u16));

    let mut buf: [u8; 2] = [0; 2];
    let _ = i2c.write_read(ADC_ADDR, &[0b01], &mut buf, delay::BLOCK)?;
    warn!("config {:04x}", (buf[0] as u16) << 8 | (buf[1] as u16));

    while buf != [config_high, config_low] {
        let _ = i2c.write_read(ADC_ADDR, &[0b01], &mut buf, delay::BLOCK)?;
        warn!("config {:04x}", (buf[0] as u16) << 8 | (buf[1] as u16));
        delay::FreeRtos::delay_ms(50);
    }

    let _ = i2c.write(ADC_ADDR, &[0b01], delay::BLOCK).expect("write config address");
    let _ = i2c.write(ADC_ADDR, &[config_high, config_low], delay::BLOCK).expect("write config");

    let mut buf: [u8; 2] = [0; 2];
    let _ = i2c.write_read(ADC_ADDR, &[0b01], &mut buf, delay::BLOCK)?;
    warn!("config {:04x}", (buf[0] as u16) << 8 | (buf[1] as u16));

    let ureference: u16;
    loop {
        let mut buf: [u8; 2] = [0; 2];
        let _ = i2c.write_read(ADC_ADDR, &[0b01], &mut buf, delay::BLOCK)?;
        info!("config 0b{:08b} 0b{:08b}", buf[0], buf[1]);
        if buf[0] >> 7 == 0b1 {
            buf = [0; 2];
            let _ = i2c.write(ADC_ADDR, &[0b00], delay::BLOCK)?;
            let _ = i2c.read(ADC_ADDR, &mut buf, delay::BLOCK)?;
            info!("value 0b{:08b} 0b{:08b}", buf[0], buf[1]);
            ureference = (buf[0] as u16) << 8 | (buf[1] as u16);
            break;
        }
    }
    let reference = ureference as f32;

    info!("Reference value: {:?} ({:?})", reference, ureference);

    // loop {
    //     // fixme we should go into a low-power state until reacting to an event
    //     delay::FreeRtos::delay_ms(250);

    //     let config_high: u8 =
    //         0b1 << 7 // Start a conversion
    //         | 0b101 << 4// single-ended measurement from AIN1
    //         | 0b000 << 1 // +/- 6.144V range
    //         | 0b1; // single-shot mode
    //     let config_low: u8 =
    //         0b000 << 5 // 128 SPS
    //         | 0b0 << 4 // Comparator mode traditional
    //         | 0b0 << 3 // Comparator polarity
    //         | 0b0 << 2 // Comparator latching
    //         | 0b00; // Comparator alert after one conversion

    //     let _ = i2c.write(ADC_ADDR, &[0b01], delay::BLOCK)?;
    //     let _ = i2c.write(ADC_ADDR, &[config_high, config_low], delay::BLOCK)?;

    //     'adc: loop {
    //         delay::FreeRtos::delay_ms(100);
    //         let mut buf: [u8; 2] = [0; 2];
    //         let _ = i2c.write_read(ADC_ADDR, &[0b01], &mut buf, delay::BLOCK)?;
    //         info!("config 0b{:08b} 0b{:08b}", buf[0], buf[1]);
    //         if buf[0] >> 7 == 0b1 {
    //             buf = [0; 2];
    //             let _ = i2c.write_read(ADC_ADDR, &[0b00], &mut buf, delay::BLOCK)?;
    //             info!("value 0b{:08b} 0b{:08b}", buf[0], buf[1]);
    //             let uvalue: u16 = (buf[0] as u16) << 8 | (buf[1] as u16);
    //             let value = uvalue as f32;
    //             info!("Received ADC reading of {:?} ({:?}), {:?}%", value, uvalue, value * 100.0 / reference);
    //             break 'adc;
    //         }
    //     }
    // }
    Ok(())
}
