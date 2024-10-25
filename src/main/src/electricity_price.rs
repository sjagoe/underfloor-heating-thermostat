use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use time::{
    ext::NumericalDuration,
    PrimitiveDateTime,
};
use std::sync::{Arc, Mutex};
use log::*;

use crate::http;
use control::ElectricityPrice;

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HourlyElectricityPrice {
    // Validity of this data, in UTC
    pub valid_from: PrimitiveDateTime,
    pub valid_until: PrimitiveDateTime,
    // Map of time at start of period, in UTC
    // On the date YYYY-MM-DD, when the prior day is YY-MM-dd,
    // YYYY-MM-ddT22:00:00Z (the day before) is the first item in the map
    // YYYY-MM-DDT21:00:00Z is the final item in the map
    pub hourly_price: HashMap<PrimitiveDateTime, ElectricityPrice>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MultiDayElectricityPrice {
    today: HourlyElectricityPrice,
    tomorrow: Option<HourlyElectricityPrice>,
}

impl MultiDayElectricityPrice {
    fn fetch(url: &str, now: PrimitiveDateTime) -> Result<MultiDayElectricityPrice> {
        let json = http::get(url)?;
        let data: MultiDayElectricityPrice = serde_json::from_str(&json)?;

        if let Some(tomorrow) = &data.tomorrow {
            if now >= tomorrow.valid_from && now < tomorrow.valid_until {
                warn!("MultiDayElectricityPrice.tomorrow appears to be today's data");
                let new_data = MultiDayElectricityPrice {
                    today: tomorrow.clone(),
                    tomorrow: None,
                };
                return Ok(new_data);
            }
        }

        Ok(data)
    }
}

pub struct SharedElectricityPrice {
    prices: Arc<Mutex<MultiDayElectricityPrice>>
}

impl SharedElectricityPrice {
    pub fn fetch(url: &str, now: PrimitiveDateTime) -> Result<SharedElectricityPrice> {
        let data = MultiDayElectricityPrice::fetch(url, now)?;
        let shared_data = SharedElectricityPrice {
            prices: Arc::new(Mutex::new(data)),
        };
        Ok(shared_data)
    }

    pub fn maybe_update(&self, url: &str) -> Result<()> {
        let now = crate::utils::time::get_datetime()?;

        let mut prices = self.prices.lock().unwrap();
        let today = &prices.today;

        warn!("now: {:?}", now);
        warn!("valid-from {:?}", today.valid_from);
        warn!("valid-until {:?}", today.valid_until);
        if now >= today.valid_from && now < today.valid_until && prices.tomorrow.is_some() {
            info!("Electricity price data is current");
            // no need to update; we have tomorrow's data and we're still in today
            return Ok(());
        }

        match &prices.tomorrow {
            Some(tomorrow) => {
                if now >= tomorrow.valid_from && now < tomorrow.valid_until {
                    info!("Promoting tomorrow's data to being in use");
                    prices.today = tomorrow.clone();
                    prices.tomorrow = None;
                }
            }
            None => {
                if now > prices.today.valid_until - 3.hours() {
                    info!("Updating electricity price data");
                    let data = MultiDayElectricityPrice::fetch(url, now)?;
                    prices.today = data.today;
                    prices.tomorrow = data.tomorrow;
                } else {
                    info!("Current prices are valid and it's too early to fetch tomorrow");
                }
            }
        }

        Ok(())
    }
}


impl fmt::Debug for SharedElectricityPrice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prices = self.prices.lock().unwrap();
        f.debug_struct("SharedElectricityPrice")
            .field("prices", &prices)
            .finish()
    }
}
