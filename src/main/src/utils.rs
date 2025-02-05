pub mod time {
    use std::{convert::TryFrom, time::SystemTime};
    use time::*;

    pub fn get_datetime() -> Result<PrimitiveDateTime> {
        let unixtime = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        let tm = unsafe { *esp_idf_svc::sys::localtime(&(unixtime.as_secs() as i64)) };
        let month = Month::try_from(1u8 + tm.tm_mon as u8)?;
        let date = Date::from_calendar_date(1900 + tm.tm_year, month, tm.tm_mday as _)?;
        let time = Time::from_hms(tm.tm_hour as _, tm.tm_min as _, tm.tm_sec as _)?;

        Ok(PrimitiveDateTime::new(date, time))
    }
}
