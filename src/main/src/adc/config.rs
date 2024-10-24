#[derive(Clone, Copy, Debug)]
pub enum AnalogInput {
    DifferentialAni0Ani1,
    DifferentialAni0Ani3,
    DifferentialAni1Ani3,
    DifferentialAni2Ani3,
    SingleEndedAni0,
    SingleEndedAni1,
    SingleEndedAni2,
    SingleEndedAni3,
}

#[derive(Clone, Copy, Debug)]
pub enum Gain {
    Full,
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Continuous,
    SingleShot,
}

#[derive(Clone, Copy, Debug)]
pub enum DataRate {
    SPS128,
    SPS250,
    SPS490,
    SPS920,
    SPS1600,
    SPS2400,
    SPS3300,
    SPS3300Duplicate,
}

#[derive(Clone, Copy, Debug)]
pub enum ComparatorMode {
    Traditional,
    Window,
}

#[derive(Clone, Copy, Debug)]
pub enum ComparatorPolarity {
    ActiveLow,
    ActiveHigh,
}

#[derive(Clone, Copy, Debug)]
pub enum ComparatorLatching {
    NonLatching,
    Latching,
}

#[derive(Clone, Copy, Debug)]
pub enum ComparatorQueue {
    SingleConversion,
    DoubleConversion,
    QuadConversion,
    Disable,
}

#[derive(Clone, Copy, Debug)]
pub struct ComparatorConfig {
    pub mode: ComparatorMode,
    pub polarity: ComparatorPolarity,
    pub latching: ComparatorLatching,
    pub queue: ComparatorQueue,
}

#[derive(Clone, Copy, Debug)]
pub struct AdcConfig {
    pub address: u8,
    pub input: AnalogInput,
    pub gain: Gain,
    pub mode: Mode,
    pub rate: DataRate,
    pub comparator: ComparatorConfig,
}

impl Default for ComparatorConfig {
    fn default() -> Self {
        ComparatorConfig {
            mode: ComparatorMode::Traditional,
            polarity: ComparatorPolarity::ActiveLow,
            latching: ComparatorLatching::NonLatching,
            queue: ComparatorQueue::Disable,
        }
    }
}

impl From<ComparatorConfig> for u8 {
    fn from(input: ComparatorConfig) -> u8 {
        u8::from(input.mode)
            | u8::from(input.polarity)
            | u8::from(input.latching)
            | u8::from(input.queue)
    }
}

impl AdcConfig {
    pub fn to_u8_array(&self, begin: bool) -> [u8; 2] {
        let start_bit: u8 = match begin {
            true => 0b1,
            false => 0b0,
        };
        let config_high = start_bit << 7
            | u8::from(self.input)
            | u8::from(self.gain)
            | u8::from(self.mode);
        let config_low = u8::from(self.rate)
            | u8::from(self.comparator);

        [config_high, config_low]
    }
}

impl Default for AdcConfig {
    fn default() -> Self{
        AdcConfig {
            address: 0b1001000,
            input: AnalogInput::DifferentialAni0Ani1,
            gain: Gain::Full,
            mode: Mode::Continuous,
            rate: DataRate::SPS1600,
            comparator: ComparatorConfig::default(),
        }
    }
}

impl From<AnalogInput> for u8 {
    fn from(input: AnalogInput) -> u8 {
        let value: u8 = match input {
            AnalogInput::DifferentialAni0Ani1 => 0b000,
            AnalogInput::DifferentialAni0Ani3 => 0b001,
            AnalogInput::DifferentialAni1Ani3 => 0b010,
            AnalogInput::DifferentialAni2Ani3 => 0b011,
            AnalogInput::SingleEndedAni0 => 0b100,
            AnalogInput::SingleEndedAni1 => 0b101,
            AnalogInput::SingleEndedAni2 => 0b110,
            AnalogInput::SingleEndedAni3 => 0b111,
        };
        value << 4
    }
}

impl Gain {
    pub fn apply(&self, value: u16) -> f32 {
        let multiplier = match self {
            Gain::Full => 3.0,
        };
        (value as f32) * multiplier
    }
}

impl From<Gain> for u8 {
    fn from(input: Gain) -> u8 {
        let value = match input {
            Gain::Full => 0b000,
        };
        value << 1
    }
}

impl From<Mode> for u8 {
    fn from(input: Mode) -> u8 {
        match input {
            Mode::Continuous => 0b0,
            Mode::SingleShot => 0b1,
        }
    }
}

impl From<DataRate> for u8 {
    fn from(input: DataRate) -> u8 {
        let value: u8 = match input {
            DataRate::SPS128 => 0b000,
            DataRate::SPS250 => 0b001,
            DataRate::SPS490 => 0b010,
            DataRate::SPS920 => 0b011,
            DataRate::SPS1600 => 0b100,
            DataRate::SPS2400 => 0b101,
            DataRate::SPS3300 => 0b110,
            DataRate::SPS3300Duplicate => 0b111,
        };
        value << 5
    }
}

impl From<ComparatorMode> for u8 {
    fn from(input: ComparatorMode) -> u8 {
        let value: u8 = match input {
            ComparatorMode::Traditional => 0b0,
            ComparatorMode::Window => 0b1,
        };
        value << 4
    }
}

impl From<ComparatorPolarity> for u8 {
    fn from(input: ComparatorPolarity) -> u8 {
        let value: u8 = match input {
            ComparatorPolarity::ActiveLow => 0b0,
            ComparatorPolarity::ActiveHigh => 0b1,
        };
        value << 3
    }
}

impl From<ComparatorLatching> for u8 {
    fn from(input: ComparatorLatching) -> u8 {
        let value: u8 = match input {
            ComparatorLatching::NonLatching => 0b0,
            ComparatorLatching::Latching => 0b1,
        };
        value << 2
    }
}

impl From<ComparatorQueue> for u8 {
    fn from(input: ComparatorQueue) -> u8 {
        match input {
            ComparatorQueue::SingleConversion => 0b00,
            ComparatorQueue::DoubleConversion => 0b01,
            ComparatorQueue::QuadConversion => 0b10,
            ComparatorQueue::Disable => 0b11,
        }
    }
}
