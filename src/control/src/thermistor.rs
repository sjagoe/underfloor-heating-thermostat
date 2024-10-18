use micromath::F32Ext;

static KELVIN_OFFSET: f32 = 273.15;

pub struct ThermistorProperties {
    beta: f32,
    r1: f32,
    t1: f32,
}

static THERMISTOR_PROPERTIES: ThermistorProperties = ThermistorProperties {
    beta: 3750.0,
    r1: 12000.0,
    t1: 25.0,
};

pub fn temperature_from_resistance(r2: f32) -> f32 {
    //                     1
    // t2 =  ------------------------------
    //           ln(rNtc / r1)        1
    //           -------------   +  ----
    //               beta            t1

    let r1 = THERMISTOR_PROPERTIES.r1;

    if r1 == r2 {
        return THERMISTOR_PROPERTIES.t1 as f32;
    }

    let beta = THERMISTOR_PROPERTIES.beta;
    let t1: f32 = (THERMISTOR_PROPERTIES.t1 as f32) + KELVIN_OFFSET;

    let resistance_ratio: f32 = (r2 as f32) / (r1 as f32);

    let t2: f32 = 1.0 / ((resistance_ratio.ln() / beta) + (1.0 / t1));

    t2 - KELVIN_OFFSET
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_from_ntc_resistance() {
        let t2 = temperature_from_resistance(12_000.0);
        assert_eq!(t2, 25.0);

        let t2 = temperature_from_resistance(12_001.0);
        assert!(t2 < 25.0);
        assert!(t2 > 24.95);

        let t2 = temperature_from_resistance(11_999.0);
        assert!(t2 > 25.0);
        assert!(t2 < 25.05);

        let t2 = temperature_from_resistance(13_050.0);
        assert!(t2 > 23.0);
        assert!(t2 < 23.05);

        let t2 = temperature_from_resistance(958.0);
        assert!(t2 > 99.95);
        assert!(t2 < 100.0);
    }
}
