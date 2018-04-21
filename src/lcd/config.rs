//!LCD configuration values
//!
//!All enums are marked with `#[repr(u8)]` so they can be taken as faw bytes using `<enum> as u8`

///Possible LCD events
pub enum Event {
    ///Enables interrupt to be run at the beggining of each frame.
    StartFrame,
    ///Enables interrupt to be run each time LCD is updated
    UpdateDone,
}

#[derive(Copy, Clone)]
#[repr(u8)]
#[allow(non_camel_case_types)]
///LCD Prescaler value.
pub enum Prescaler {
    ///LCDCLK
    PS_1 = 0,
    ///LCDCLK/2
    PS_2 = 1,
    ///LCDCLK/4
    PS_4 = 2,
    ///LCDCLK/8
    PS_8 = 3,
    ///LCDCLK/16
    PS_16 = 4,
    ///LCDCLK/32
    PS_32 = 5,
    ///LCDCLK/64
    PS_64 = 6,
    ///LCDCLK/128
    PS_128 = 7,
    ///LCDCLK/256
    PS_256 = 8,
    ///LCDCLK/512
    PS_512 = 9,
    ///LCDCLK/1024
    PS_1024 = 10,
    ///LCDCLK/2048
    PS_2048 = 11,
    ///LCDCLK/4096
    PS_4096 = 12,
    ///LCDCLK/8192
    PS_8192 = 13,
    ///LCDCLK/16384
    PS_16384 = 14,
    ///LCDCLK/32768
    PS_32768 = 15,
}

#[derive(Copy, Clone)]
#[repr(u8)]
#[allow(non_camel_case_types)]
///LCD's clock divider.
pub enum Divider {
    ///CLKPS/16
    DIV_16 = 0,
    ///CLKPS/17
    DIV_17 = 1,
    ///CLKPS/18
    DIV_18 = 2,
    ///CLKPS/19
    DIV_19 = 3,
    ///CLKPS/20
    DIV_20 = 4,
    ///CLKPS/21
    DIV_21 = 5,
    ///CLKPS/22
    DIV_22 = 6,
    ///CLKPS/23
    DIV_23 = 7,
    ///CLKPS/24
    DIV_24 = 8,
    ///CLKPS/25
    DIV_25 = 9,
    ///CLKPS/26
    DIV_26 = 10,
    ///CLKPS/27
    DIV_27 = 11,
    ///CLKPS/28
    DIV_28 = 12,
    ///CLKPS/29
    DIV_29 = 13,
    ///CLKPS/30
    DIV_30 = 14,
    ///CLKPS/30
    DIV_31 = 15,
}

#[derive(Copy, Clone)]
#[repr(u8)]
///LCD's duty cycle.
pub enum Duty {
    Static = 0,
    ///1/2 Duty
    OneTo2 = 1,
    ///1/3 Duty
    OneTo3 = 2,
    ///1/4 Duty
    OneTo4 = 3,
    ///1/8 Duty
    OneTo8 = 4,
}

#[repr(u8)]
///LCD's bias selector.
pub enum Bias {
    Bias14 = 0,
    Bias12 = 1,
    Bias13 = 2,
}

#[repr(u8)]
///Blink mode selection
pub enum BlinkMode {
    ///Disabled
    Disabled = 0,
    ///Blink enabled for SEG[0], COM[0] (1 pixel)
    Seg0Com0 = 1,
    ///Blink enabled for SEG[0], all COMs. (up to 8 pixels depending on the programmed duty)
    Seg0ComAll = 2,
    ///Blink enabled on all SEGs and all COMs (all pixels)
    SegAllComAll = 3,
}

#[repr(u8)]
///Blink frequency
pub enum BlinkFreq {
    Div8 = 0,
    Div16 = 1,
    Div32 = 2,
    Div64 = 3,
    Div128 = 4,
    Div256 = 5,
    Div512 = 6,
    Div1024 = 7,
}

#[repr(u8)]
///Dead time duration.
///
///During the dead time the COM and SEG voltage levels are held at 0 V to reduce the contrast
///without modifying the frame rate.
pub enum DeadTime {
    None = 0,
    Phase1 = 1,
    Phase2 = 2,
    Phase3 = 3,
    Phase4 = 4,
    Phase5 = 5,
    Phase6 = 6,
    Phase7 = 7
}

#[repr(u8)]
///Voltage source for LCD
pub enum VoltageSource {
    ///Voltage step-up converter
    Internal = 0,
    ///VLCD pin
    External = 1
}

#[repr(u8)]
///Pulse duration
///
///A short pulse will lead to lower power consumption, but displays with high internal resistance may
///need a longer pulse to achieve satisfactory contrast
///
///Formula: `PulseDuration/(LCDCLK/PS)`
pub enum PulseDuration {
    None = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7
}

#[repr(u8)]
///Specifies LCD's maximum voltage in range from **2.60V** to **3.51V**
///
///For details refer to board's datasheet.
pub enum Contrast {
    None = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7
}

#[repr(u8)]
///High drive mode.
///
///Displays with high internal resistance may need a longer drive time to achieve satisfactory contrast.
///This bit is useful in this case if some additional power consumption can be tolerated.
pub enum HighDrive {
    Off = 0,
    ///When `On`, the [PulseDuration](struct.PulseDuration.html) have to be programmed to `One`
    On = 1
}

#[repr(u8)]
///SEG pin remapping.
///
///Refer to `Reference manual Ch. 25.3.7` for details
pub enum MuxSegment {
    Off = 0,
    ///SEG[31:28] are multiplexed with SEG[43:40]
    On = 1
}

#[derive(Default)]
///LCD configuration struct.
///
///When option is not set, nothing is written into LCD's registers.
pub struct Config {
    pub prescaler: Option<Prescaler>,
    pub divider: Option<Divider>,
    pub duty: Option<Duty>,
    pub bias: Option<Bias>,
    pub blink_mode: Option<BlinkMode>,
    pub blink_freq: Option<BlinkFreq>,
    pub dead_time: Option<DeadTime>,
    pub voltage_source: Option<VoltageSource>,
    pub pulse_duration: Option<PulseDuration>,
    pub contrast: Option<Contrast>,
    pub high_drive: Option<HighDrive>,
    pub mux_segment: Option<MuxSegment>
}
