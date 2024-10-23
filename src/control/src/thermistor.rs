#[allow(unused_imports)]
use micromath::F32Ext;  // Required for f32::ln

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

#[allow(dead_code)]
pub fn temperature_from_resistance(r2: f32) -> f32 {
    //                     1
    // t2 =  ------------------------------
    //           ln(rNtc / r1)        1
    //           -------------   +  ----
    //               beta            t1

    let r1 = THERMISTOR_PROPERTIES.r1;

    if r1 == r2 {
        return THERMISTOR_PROPERTIES.t1;
    }

    let beta = THERMISTOR_PROPERTIES.beta;
    let t1: f32 = THERMISTOR_PROPERTIES.t1 + KELVIN_OFFSET;

    let resistance_ratio = r2 / r1;

    let t2 = 1.0 / ((resistance_ratio.ln() / beta) + (1.0 / t1));

    t2 - KELVIN_OFFSET
}

#[allow(dead_code)]
pub fn voltage_to_resistance(v_supply: f32, sample: f32, reference_reistance: f32) -> f32 {
    //
    // Vcc *--
    //       |
    //      R1
    //       |---- Vout
    //      R2
    //       |
    // GND *--
    //
    // Vcc = Vr1 + Vr2
    //
    // I = Vcc / (R1 + R2)
    //
    // Ir2 = Vr2 / R2 = Vcc / (R1 + R2)
    //
    // Vr2 = (Vcc * R2) / (R1 + R2)
    //
    // Vr2 * R1 + Vr2 * R2 = Vcc * R2
    //
    // Vcc * R2 - Vr2 * R2 = Vr2 * R1
    //
    // R2 (Vcc - Vr2) = Vr2 * R1
    //
    // R2 = (Vr2 * R1) / (Vcc - Vr2)

    (sample * reference_reistance) / (v_supply - sample)
}

pub fn temperature_from_voltage(v_supply: f32, sample: f32) -> f32 {
    // NTC temperature (Kelvin) given resistance and beta value
    //
    //                     1
    // t2 =  ------------------------------
    //           ln(rNtc / R1)        1
    //           --------------  +  ----
    //               beta            T1

    // Voltage divider for resistance from voltage
    // rNtc = R2 = (Vr2 * R1) / (Vcc - Vr2)

    // Substuting

    //                     1
    // t2 =  ---------------------------------------------------
    //           ln((Vr2 * R1) / (Vcc - Vr2) / R1)       1
    //           ----------------------------------  +  ----
    //               beta                                T1

    //                     1
    // t2 =  ------------------------------------
    //           ln(Vr2 / (Vcc - Vr2))       1
    //           ----------------------  +  ----
    //               beta                    t1

    let a = sample / (v_supply - sample);
    let b = a.ln() / THERMISTOR_PROPERTIES.beta;
    let c = b + (1.0 / (THERMISTOR_PROPERTIES.t1 + KELVIN_OFFSET));
    (1.0 / c) - KELVIN_OFFSET
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

    #[test]
    fn test_voltage_to_resistance() {
        let vcc = 5000.0;
        let r1 = 12_000.0;
        // 50% voltage divider
        let r = voltage_to_resistance(vcc, 2500.0, r1);
        assert_eq!(r, 12_000.0);

        let r = voltage_to_resistance(vcc, 1000.0, r1);
        assert_eq!(r, 3000.0);

        let r = voltage_to_resistance(vcc, 4000.0, r1);
        assert_eq!(r, 48000.0);
    }

    #[test]
    fn test_voltage_to_temperature() {
        let vcc = 5000.0;
        let r1 = 12_000.0;
        let r2 = voltage_to_resistance(vcc, 1000.0, r1);
        let temperature = temperature_from_resistance(r2);
        assert!(temperature > 61.9);
        assert!(temperature < 61.95);

        let r2 = voltage_to_resistance(vcc, 4000.0, r1);
        let temperature = temperature_from_resistance(r2);
        assert!(temperature < 4.575);
        assert!(temperature > -4.625);
    }

    #[test]
    fn test_voltage_to_temperature_direct() {
        let vcc = 5000.0;
        let temperature = temperature_from_voltage(vcc, 1000.0);
        assert!(temperature > 61.9);
        assert!(temperature < 61.95);

        let temperature = temperature_from_voltage(vcc, 4000.0);
        assert!(temperature < 4.575);
        assert!(temperature > -4.625);
    }
}
