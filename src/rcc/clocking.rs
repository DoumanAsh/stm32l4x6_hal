//! The `clocking` module contains representations of the various objects in the STM32L4x6
//! clock tree (see Reference Manual Figs 15 and 16) useful for wiring them up.
//!
//! There are two main concepts: sources (enum variants) and clocks (structs). For example, the
//! SYSCLK clock can be driven by any one of four sources: HSI, MSI, HSI, and PLLCLK. This
//! knowledge is encoded in the SysClkSources enum.
//!
//! Each enum variant contains information about the clock it represents. Some clocks are
//! configurable, and thus have fields, and some are not. For example, the LSI
//! (LowSpeedInternalRC) clock is always 32 kHz, but the MSI (MediumSpeedInternalRC) clock can
//! be configured and thus has a frequency component.
//!
//! To use them, compose them and feed them to, e.g., sysclk.
//!
//! ```rust
//! let mut rcc = RCC.constrain();
//! let msi_clk = clocking::MediumSpeedInternalRC::new(8_000_000, false);
//! let sys_clk_src = clocking::SysClkSource::MSI(msi_clk);
//! let cfgr = rcc.cfgr.sysclk(sys_clk_src);
//! ```
//!
//! The PLL is a bit more complex because it _is_ a source (`PLLClkOutput`) and also _requires_
//! a source (`PLLClkSource`), but you compose the types similarly.

use super::rcc;

pub trait InputClock {
    fn freq(&self) -> u32;
}

/// High-speed internal 16 MHz RC
#[derive(Clone, Copy)]
pub struct HighSpeedInternal16RC {
    pub always_on: bool,
    pub auto_start: bool,
}

impl HighSpeedInternal16RC {
    /// Applies the selection options to the configuration registers and turns the clock on
    pub fn configure(&self, rcc: &rcc::RegisterBlock) -> (u32, u8) {
        rcc.cr.modify(|_, w| {
            w.hsion()
                .set_bit()
                .hsikeron()
                .bit(self.always_on)
                .hsiasfs()
                .bit(self.auto_start)
        });
        while rcc.cr.read().hsirdy().bit_is_clear() {}
        (16_000_000, 0b01)
    }
}

/// Medium-speed internal 100 kHz - 48 MHz RC
#[derive(Clone, Copy)]
pub struct MediumSpeedInternalRC {
    freq: u32,
    auto_cal: bool,
}

impl MediumSpeedInternalRC {
    pub fn new(freq: u32, auto_cal: bool) -> Self {
        MediumSpeedInternalRC { freq, auto_cal }
    }

    pub fn bits(&self) -> u8 {
        match self.freq {
            100_000 => 0b0000,
            200_000 => 0b0001,
            400_000 => 0b0010,
            800_000 => 0b0011,
            1_000_000 => 0b0100,
            2_000_000 => 0b0101,
            4_000_000 => 0b0110,
            8_000_000 => 0b0111,
            16_000_000 => 0b1000,
            24_000_000 => 0b1001,
            32_000_000 => 0b1010,
            48_000_000 => 0b1011,
            _ => panic!("bad MSI speed value!"),
        }
    }

    /// Configures the MSI to the specified frequency, and enables hardware
    /// auto-calibration if requested by enabling (and waiting for) the LSE.
    pub fn configure(&self, rcc: &rcc::RegisterBlock) -> (u32, u8) {
        rcc.cr
            .modify(|_, w| unsafe { w.msirange().bits(self.bits()).msirgsel().set_bit() });
        while rcc.cr.read().msirdy().bit_is_clear() {}

        if self.auto_cal {
            // FIXME This... may not work? I'm not sure if I've got a board problem or using
            // the LSE requires some precondition I'm missing. In either case, LSERDY is never
            // set by the hardware, so auto_cal doesn't succeed.
            rcc.apb1enr1.modify(|_, w| w.pwren().set_bit());

            rcc.bdcr.modify(|_, w| w.lseon().clear_bit());
            while rcc.bdcr.read().lserdy().bit_is_set() {}
            rcc.bdcr
                .modify(|_, w| unsafe { w.lsedrv().bits(0b11).lseon().set_bit() });
            while rcc.bdcr.read().lserdy().bit_is_clear() {}
            rcc.cr.modify(|_, w| w.msipllen().set_bit());
        }
        (self.freq(), 0b00)
    }
}

impl InputClock for MediumSpeedInternalRC {
    fn freq(&self) -> u32 {
        self.freq
    }
}

/// High-speed external 4-48 MHz oscillator
#[derive(Clone, Copy)]
pub struct HighSpeedExternalOSC(pub u32);

impl InputClock for HighSpeedExternalOSC {
    fn freq(&self) -> u32 {
        self.0
    }
}

impl HighSpeedExternalOSC {
    /// Turns on the HSE oscillator.
    ///
    /// (Should this also configure the pin?)
    pub fn configure(&self, rcc: &rcc::RegisterBlock) -> (u32, u8) {
        rcc.cr.modify(|_, w| w.hseon().set_bit());
        while rcc.cr.read().hserdy().bit_is_clear() {}
        (self.freq(), 0b10)
    }
}

pub enum RtcClkSource {
    None,
    /// Internal 32 kHz RC
    LSI,
    /// External 32.768 kHz oscillator
    LSE,
    /// High-speed external oscillator, prescaled by (a fixed value of) 32
    HSEDiv32,
}

impl RtcClkSource {
    /// Returns the output frequency of the RtcClkSource based on its input.
    pub fn freq(&self, hse: Option<HighSpeedExternalOSC>) -> Option<u32> {
        match *self {
            RtcClkSource::None => None,
            RtcClkSource::LSI => Some(32_000),
            RtcClkSource::LSE => Some(32_768),
            RtcClkSource::HSEDiv32 => {
                if let Some(clk) = hse {
                    Some(clk.freq() / 32)
                } else {
                    None
                }
            }
        }
    }

    pub fn bits(&self) -> u8 {
        match *self {
            RtcClkSource::None => 0,
            RtcClkSource::LSI => 1,
            RtcClkSource::LSE => 2,
            RtcClkSource::HSEDiv32 => 3,
        }
    }
}

#[derive(Clone, Copy)]
pub enum SysClkSource {
    HSI16(HighSpeedInternal16RC),
    MSI(MediumSpeedInternalRC),
    HSE(HighSpeedExternalOSC),
    PLL(PLLClkOutput),
}

impl InputClock for SysClkSource {
    fn freq(&self) -> u32 {
        match *self {
            SysClkSource::HSI16(_) => 16_000_000,
            SysClkSource::MSI(s) => s.freq(),
            SysClkSource::HSE(s) => s.freq(),
            SysClkSource::PLL(s) => s.freq(),
        }
    }
}

/// PLLCLK output of PLL module
#[derive(Clone, Copy)]
pub struct PLLClkOutput {
    pub src: PLLClkSource,
    pub m: u8,
    n: u8,
    r: u8,
    f: u32,
}

impl PLLClkOutput {
    /// Create a new PLL clock source to use as an input. The arguments refer to the scale
    /// factors described in Figs. 15 and 16 of the reference manual, and end up in the PLLM,
    /// PLLN, and PLLR fields of the PLLCFGR register.
    pub fn new(src: PLLClkSource, m: u8, n: u8, r: u8) -> Self {
        assert!(m > 0 && m < 9);
        assert!(n > 7 && n < 87);
        assert!(r == 2 || r == 4 || r == 6 || r == 8);
        let f = src.freq() / m as u32 * n as u32 / r as u32;
        assert!(f < super::SYS_CLOCK_MAX);

        PLLClkOutput { src, m, n, r, f }
    }

    /// Configure the PLL to enable the PLLCLK output. This explicitly does not (yet?)
    /// support any PLL other than `PLL`, and no other outputs than `PLLCLK`, so this is
    /// not suitable for driving e.g. USB.
    pub fn configure(&self, rcc: &rcc::RegisterBlock) -> (u32, u8) {
        let pllsrc_bits = self.src.configure(rcc);
        rcc.cr.modify(|_, w| w.pllon().clear_bit());
        while rcc.cr.read().pllrdy().bit_is_set() {}
        rcc.pllcfgr.modify(|_, w| unsafe {
            w.pllsrc()
                .bits(pllsrc_bits)
                .pllm()
                .bits(self.m - 1)
                .plln()
                .bits(self.n)
                .pllr()
                .bits(self.r)
        });
        rcc.cr.modify(|_, w| w.pllon().set_bit());
        while rcc.cr.read().pllrdy().bit_is_clear() {}
        rcc.pllcfgr.modify(|_, w| w.pllren().set_bit());
        (self.freq(), 0b11)
    }
}

impl InputClock for PLLClkOutput {
    fn freq(&self) -> u32 {
        self.f
    }
}

/*
/// PLLADC2CLK output of PLLSAI2
#[derive(Clone, Copy)]
pub struct PLLADC2Clk {
src: PLLClkSource,
...,
}
*/

#[derive(Clone, Copy)]
pub enum PLLClkSource {
    None,
    MSI(MediumSpeedInternalRC),
    HSI16(HighSpeedInternal16RC),
    HSE(HighSpeedExternalOSC),
}

impl PLLClkSource {
    /// This configures the input to the PLL. It's usually only called by
    /// PLLClkOutput::configure.
    pub fn configure(&self, rcc: &rcc::RegisterBlock) -> u8 {
        match self {
            PLLClkSource::None => 0b00,
            PLLClkSource::MSI(s) => {
                s.configure(rcc);
                0b01
            }
            PLLClkSource::HSI16(s) => {
                s.configure(rcc);
                0b10
            }
            PLLClkSource::HSE(s) => {
                s.configure(rcc);
                0b11
            }
        }
    }
}

impl InputClock for PLLClkSource {
    fn freq(&self) -> u32 {
        match *self {
            PLLClkSource::None => 0,
            PLLClkSource::MSI(s) => s.freq(),
            PLLClkSource::HSI16(_) => 16_000_000,
            PLLClkSource::HSE(s) => s.freq(),
        }
    }
}
