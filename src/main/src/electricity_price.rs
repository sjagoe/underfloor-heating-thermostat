use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::PrimitiveDateTime;

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

impl HourlyElectricityPrice {
    pub fn fetch(url: &str) -> Result<HourlyElectricityPrice> {
        let json = http::get(url)?;
        let data: HourlyElectricityPrice = serde_json::from_str(&json)?;
        Ok(data)
    }
}
