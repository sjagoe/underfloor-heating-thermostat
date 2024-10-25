use anyhow::Result;
use esp_idf_svc::hal::{delay::FreeRtos, modem::WifiModemPeripheral, peripheral::Peripheral};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    wifi::{AuthMethod, ClientConfiguration, Configuration, EspWifi},
};
use std::sync::{Arc, Mutex, MutexGuard};
use log::*;

struct SharedWifi<'d> {
    esp_wifi: Arc<Mutex<EspWifi<'d>>>,
}

impl<'d> SharedWifi<'d> {
    pub fn connect_wifi(
        modem: impl Peripheral<P = impl WifiModemPeripheral + 'd> + 'd,
        sysloop: EspSystemEventLoop,
        partition: Option<EspDefaultNvsPartition>,
        auth_method: AuthMethod,
        ssid: &str,
        psk: &str,
    ) -> Result<SharedWifi<'d>> {
        let mut wifi = EspWifi::new(modem, sysloop.clone(), partition)?;

        if psk.is_empty() {
            wifi.set_configuration(&Configuration::Client(ClientConfiguration {
                ssid: ssid.try_into().expect("Could not parse SSID"),
                auth_method: AuthMethod::None,
                ..Default::default()
            }))?;
        } else {
            wifi.set_configuration(&Configuration::Client(ClientConfiguration {
                ssid: ssid
                    .try_into()
                    .expect("Could not parse SSID into Wifi config"),
                password: psk
                    .try_into()
                    .expect("Could not parse PSK into Wifi config"),
                auth_method,
                ..Default::default()
            }))?;
        }

        wifi.start()?;
        wifi.connect()?;

        let shared_wifi = SharedWifi {
            esp_wifi: Arc::new(Mutex::new(wifi)),
        };
        Ok(shared_wifi)
    }

    pub fn wait_for_connected(&self) -> Result<()> {
        let wifi = self.esp_wifi.lock().unwrap();

        loop {
            FreeRtos::delay_ms(250);
            let connected = wifi.is_up()?;
            if connected {
                break;
            }
        }

        info!("Connected to wifi");

        Ok(())
    }
}
