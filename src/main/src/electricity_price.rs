use anyhow::Result;
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use time::{ext::NumericalDuration, PrimitiveDateTime, Time};

use crate::http;
use crate::StatusEvent;
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

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct MultiDayElectricityPrice {
    today: Option<HourlyElectricityPrice>,
    tomorrow: Option<HourlyElectricityPrice>,
}

impl MultiDayElectricityPrice {
    fn fetch(url: &str, now: PrimitiveDateTime) -> Result<MultiDayElectricityPrice> {
        let json = http::get(url)?;
        let data: MultiDayElectricityPrice = serde_json::from_str(&json)?;

        if let Some(tomorrow) = &data.tomorrow {
            if now >= tomorrow.valid_until {
                error!("Received stale electricity price data!");
                return Ok(MultiDayElectricityPrice::default());
            }
            if now >= tomorrow.valid_from && now < tomorrow.valid_until {
                warn!("MultiDayElectricityPrice.tomorrow appears to be today's data");
                let new_data = MultiDayElectricityPrice {
                    today: Some(tomorrow.clone()),
                    tomorrow: None,
                };
                return Ok(new_data);
            }
        }

        Ok(data)
    }
}

#[derive(Clone)]
pub struct SharedElectricityPrice {
    prices: Arc<Mutex<MultiDayElectricityPrice>>,
}

impl SharedElectricityPrice {
    pub fn fetch(url: &str, now: PrimitiveDateTime) -> Result<SharedElectricityPrice> {
        let data = MultiDayElectricityPrice::fetch(url, now)?;
        let shared_data = SharedElectricityPrice {
            prices: Arc::new(Mutex::new(data)),
        };
        Ok(shared_data)
    }

    pub fn current_price(&self) -> Option<ElectricityPrice> {
        let now = crate::utils::time::get_datetime().expect("Failed to get tiem");
        let key_time = Time::from_hms(now.time().hour(), 0, 0).expect("Failed to construct time");
        let key = PrimitiveDateTime::new(now.date(), key_time);

        let prices = self.prices.lock().unwrap();
        let today = prices.today.clone();
        let tomorrow = &prices.tomorrow.clone();

        if let Some(today) = today {
            if let Some(price) = today.hourly_price.get(&key) {
                return Some(*price);
            }
        }
        if let Some(tomorrow) = tomorrow {
            if let Some(price) = tomorrow.hourly_price.get(&key) {
                return Some(*price);
            }
        }
        None
    }

    pub fn status(&self) -> Option<StatusEvent> {
        let prices = self.prices.lock().unwrap();

        if prices.today.is_none() {
            return Some(StatusEvent::MissingData);
        }
        None
    }

    pub fn maybe_update(&self, url: &str) -> Result<()> {
        let now = crate::utils::time::get_datetime()?;

        let mut prices = self.prices.lock().unwrap();
        let today = prices.today.clone();
        let tomorrow = &prices.tomorrow.clone();

        let mut do_update = || -> Result<()> {
            info!("Updating electricity price data");
            let data = MultiDayElectricityPrice::fetch(url, now)?;
            prices.today = data.today;
            prices.tomorrow = data.tomorrow;
            Ok(())
        };

        match today {
            Some(today) => {
                if now >= today.valid_from && now < today.valid_until && tomorrow.is_some() {
                    info!("Electricity price data is current");
                    // no need to update; we have tomorrow's data and we're still in today
                    return Ok(());
                }

                match tomorrow {
                    Some(tomorrow) => {
                        if now >= tomorrow.valid_from && now < tomorrow.valid_until {
                            info!("Promoting tomorrow's data to being in use");
                            prices.today = Some(tomorrow.clone());
                            prices.tomorrow = None;
                        }
                    }
                    None => {
                        if now > today.valid_until - 3.hours() {
                            do_update()?;
                        } else {
                            info!("Current prices are valid and it's too early to fetch tomorrow");
                        }
                    }
                }
            }
            None => do_update()?,
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
