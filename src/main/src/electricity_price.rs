use time::PrimitiveDateTime;

use control::ElectricityPrice;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct HourlyElectricityPrice {
    // Validity of this data, in UTC
    pub valid_from: PrimitiveDateTime,
    pub valid_until: PrimitiveDateTime,
    // Hourly electricity price. The hour of the day in UTC is the
    // index into the array.
    // This does mean that the "start" of the day in local time may
    // not be at the start of the array.
    pub hourly_price: [ElectricityPrice; 24],
}
