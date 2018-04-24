//! Reset and Clock Control

// TODO right now the various configure functions reach into rcc directly. This is bad. Add an
// opaque CR member to RCC, and add methods to CR and BDCR. They should probably take clock source
// variant arguments.

use stm32l4x6::{rcc, PWR, RCC};

use common::Constrain;
use flash::ACR;
use time::Hertz;

impl Constrain<Rcc> for RCC {
    /// Create an RCC peripheral handle.
    ///
    /// Per Reference Manual Ch. 6.2 the default System Clock source is MSI clock with frequency 4 MHz
    ///
    /// The `constrain` method enables write access to the BDCR, and the `freeze` method disables
    /// it again. This is to enable changing LSE- and RTC-related settings.
    fn constrain(self) -> Rcc {
        // Enable write access to the BDCR; this is necessary to enable the LSE and change RTC
        // settings.
        unsafe {
            (*PWR::ptr()).cr1.modify(|_, w| w.dbp().set_bit());
        }
        // Write access is (similarly) disabled in CFGR::freeze()
        // TODO add PWR to the hal to avoid the above nastiness
        Rcc {
            ahb: AHB(()),
            apb1: APB1(()),
            apb2: APB2(()),
            bdcr: BDCR(()),
            csr: CSR(()),
            cfgr: CFGR {
                hclk: None,
                pclk1: None,
                pclk2: None,
                sysclk: clocking::SysClkSource::MSI(clocking::MediumSpeedInternalRC::new(
                    4_000_000, false,
                )),
            },
        }
    }
}

pub mod clocking {
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

}

/// Constrained RCC peripheral
pub struct Rcc {
    /// AMBA High-performance Bus (AHB) registers.
    pub ahb: AHB,
    /// APB1 peripheral registers.
    pub apb1: APB1,
    /// APB2 peripheral registers.
    pub apb2: APB2,
    /// Backup domain registers.
    pub bdcr: BDCR,
    /// Control/status register.
    pub csr: CSR,
    /// HW clock configuration.
    pub cfgr: CFGR,
}

pub struct AHB(());
impl AHB {
    /// Access AHB1 reset register
    pub fn rstr1(&mut self) -> &rcc::AHB1RSTR {
        unsafe { &(*RCC::ptr()).ahb1rstr }
    }
    /// Access AHB2 reset register
    pub fn rstr2(&mut self) -> &rcc::AHB2RSTR {
        unsafe { &(*RCC::ptr()).ahb2rstr }
    }
    /// Access AHB3 reset register
    pub fn rstr3(&mut self) -> &rcc::AHB3RSTR {
        unsafe { &(*RCC::ptr()).ahb3rstr }
    }

    /// Access AHB1 clock enable register
    pub fn enr1(&mut self) -> &rcc::AHB1ENR {
        unsafe { &(*RCC::ptr()).ahb1enr }
    }
    /// Access AHB3 clock enable register
    pub fn enr2(&mut self) -> &rcc::AHB2ENR {
        unsafe { &(*RCC::ptr()).ahb2enr }
    }
    /// Access AHB3 clock enable register
    pub fn enr3(&mut self) -> &rcc::AHB3ENR {
        unsafe { &(*RCC::ptr()).ahb3enr }
    }
}

pub struct APB1(());
impl APB1 {
    ///Access APB1RSTR1 reset register
    pub fn rstr1(&mut self) -> &rcc::APB1RSTR1 {
        unsafe { &(*RCC::ptr()).apb1rstr1 }
    }
    ///Access APB1RSTR2 reset register
    pub fn rstr2(&mut self) -> &rcc::APB1RSTR2 {
        unsafe { &(*RCC::ptr()).apb1rstr2 }
    }

    ///Access APB1ENR1 reset register
    pub fn enr1(&mut self) -> &rcc::APB1ENR1 {
        unsafe { &(*RCC::ptr()).apb1enr1 }
    }
    ///Access APB1ENR2 reset register
    pub fn enr2(&mut self) -> &rcc::APB1ENR2 {
        unsafe { &(*RCC::ptr()).apb1enr2 }
    }
}

pub struct APB2(());
impl APB2 {
    ///Access APB2RSTR reset register
    pub fn rstr(&mut self) -> &rcc::APB2RSTR {
        unsafe { &(*RCC::ptr()).apb2rstr }
    }

    ///Access APB2ENR reset register
    pub fn enr(&mut self) -> &rcc::APB2ENR {
        unsafe { &(*RCC::ptr()).apb2enr }
    }
}

///Backup domain control register.
///
///Note that it may be write protected and in order to modify it
///`Power Control Register` can be accessed to lift protection.
///See description of CR1's DBP bit in Ch. 5.4.1
///
///See Reference manual Ch. 6.4.29
pub struct BDCR(());
impl BDCR {
    #[inline]
    pub fn inner(&mut self) -> &rcc::BDCR {
        unsafe { &(*RCC::ptr()).bdcr }
    }

    ///Resets entire Backup domain.
    ///
    ///Use it when you want to change clock source.
    pub fn reset(&mut self) {
        self.inner().modify(|_, write| write.bdrst().set_bit());
        self.inner().modify(|_, write| write.bdrst().clear_bit());
    }

    ///Returns type of RTC Clock.
    pub fn rtc_clock(&mut self) -> clocking::RtcClkSource {
        match self.inner().read().rtcsel().bits() {
            0 => clocking::RtcClkSource::None,
            1 => clocking::RtcClkSource::LSE,
            2 => clocking::RtcClkSource::LSI,
            3 => clocking::RtcClkSource::HSEDiv32,
            _ => unimplemented!(),
        }
    }

    ///Select clock source for RTC.
    ///
    ///**NOTE:** Once source has been selected, it cannot be changed anymore
    ///unless backup domain is reset.
    pub fn set_rtc_clock(&mut self, clock: clocking::RtcClkSource) {
        self.inner()
            .modify(|_, write| unsafe { write.rtcsel().bits(clock.bits()) });
    }

    ///Sets RTC on/off
    pub fn rtc_enable(&mut self, is_on: bool) {
        self.inner().modify(|_, write| write.rtcen().bit(is_on));
    }

    ///Sets LSE on/off
    pub fn lse_enable(&mut self, is_on: bool) {
        let inner = self.inner();

        if inner.read().lseon().bit() == is_on {
            return;
        }

        inner.modify(|_, write| write.lseon().bit(is_on));
        match is_on {
            true => while inner.read().lserdy().bit_is_clear() {},
            false => while inner.read().lserdy().bit_is_set() {},
        }
    }
}

///Control/Status Register
///
///See Reference manual Ch. 6.4.29
pub struct CSR(());
impl CSR {
    #[inline]
    pub fn inner(&mut self) -> &rcc::CSR {
        unsafe { &(*RCC::ptr()).csr }
    }

    ///Turns on/off LSI oscillator.
    pub fn lsi_enable(&mut self, is_on: bool) {
        let inner = self.inner();

        if inner.read().lsion().bit() == is_on {
            return;
        }

        inner.modify(|_, write| write.lsion().bit(is_on));
        match is_on {
            true => while inner.read().lsirdy().bit_is_clear() {},
            false => while inner.read().lsirdy().bit_is_set() {},
        }
    }
}

///Maximum value for System clock.
///
///Reference Ch. 6.2.8
pub const SYS_CLOCK_MAX: u32 = 80_000_000;

///Clock configuration
pub struct CFGR {
    /// AHB bus frequency
    hclk: Option<u32>,
    /// APB1
    pclk1: Option<u32>,
    /// APB2
    pclk2: Option<u32>,
    /// SYSCLK - not Option because it cannot be None
    sysclk: clocking::SysClkSource,
}

impl CFGR {
    /// Sets a frequency for the AHB bus.
    pub fn hclk<T: Into<Hertz>>(mut self, freq: T) -> Self {
        self.hclk = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the APB1 bus.
    pub fn pclk1<T: Into<Hertz>>(mut self, freq: T) -> Self {
        self.pclk1 = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the APB2 bus.
    pub fn pclk2<T: Into<Hertz>>(mut self, freq: T) -> Self {
        self.pclk2 = Some(freq.into().0);
        self
    }

    /// Sets a frequency and a source for the System clock
    pub fn sysclk(mut self, src: clocking::SysClkSource) -> Self {
        if let clocking::SysClkSource::PLL(s) = src {
            if let clocking::PLLClkSource::None = s.src {
                panic!("PLL must have input clock to drive SYSCLK");
            }
        } else {
            self.sysclk = src;
        }
        self
    }

    /// Freezes the clock configuration, making it effective
    pub fn freeze(self, acr: &mut ACR) -> Clocks {
        let rcc = unsafe { &*RCC::ptr() };

        let (sys_clock, sw_bits) = match self.sysclk {
            clocking::SysClkSource::MSI(s) => s.configure(rcc),
            clocking::SysClkSource::HSI16(s) => s.configure(rcc),
            clocking::SysClkSource::HSE(s) => s.configure(rcc),
            clocking::SysClkSource::PLL(s) => s.configure(rcc),
        };

        let hpre_bits = match self.hclk.map(|hclk| sys_clock / hclk) {
            Some(0) => unreachable!(),
            Some(1) => 0b0111,
            Some(2) => 0b1000,
            Some(3...5) => 0b1001,
            Some(6...11) => 0b1010,
            Some(12...39) => 0b1011,
            Some(40...95) => 0b1100,
            Some(96...191) => 0b1101,
            Some(192...383) => 0b1110,
            _ => 0b1111,
        };

        let ahb = sys_clock / (1 << (hpre_bits - 0b0111));

        let ppre1_bits = match self.pclk1.map(|pclk1| ahb / pclk1) {
            Some(0) => unreachable!(),
            Some(1) => 0b011,
            Some(2) => 0b100,
            Some(3...5) => 0b101,
            Some(6...11) => 0b110,
            _ => 0b111,
        };

        let ppre1 = 1 << (ppre1_bits - 0b011);
        let apb1 = ahb / ppre1 as u32;

        let ppre2_bits = match self.pclk2.map(|pclk2| ahb / pclk2) {
            Some(0) => unreachable!(),
            Some(1) => 0b011,
            Some(2) => 0b100,
            Some(3...5) => 0b101,
            Some(6...11) => 0b110,
            _ => 0b111,
        };

        let ppre2 = 1 << (ppre2_bits - 0b011);
        let apb2 = ahb / ppre2 as u32;

        //Reference AN4621 note Figure. 4
        //from 0 wait state to 4
        let latency = if sys_clock <= 16_000_000 {
            0b000
        } else if sys_clock <= 32_000_000 {
            0b001
        } else if sys_clock <= 48_000_00 {
            0b010
        } else if sys_clock <= 64_000_00 {
            0b011
        } else {
            0b100
        };

        acr.acr().write(|w| unsafe { w.latency().bits(latency) });

        rcc.cfgr.modify(|_, w| unsafe {
            w.ppre2()
                .bits(ppre2_bits)
                .ppre1()
                .bits(ppre1_bits)
                .hpre()
                .bits(hpre_bits)
                .sw()
                .bits(sw_bits)
        });

        // Disable BDCR write access
        unsafe {
            (*PWR::ptr()).cr1.modify(|_, w| w.dbp().clear_bit());
        }

        Clocks {
            hclk: Hertz(ahb),
            pclk1: Hertz(apb1),
            pclk2: Hertz(apb2),
            sysclk: Hertz(sys_clock),
            pll_src: match self.sysclk {
                clocking::SysClkSource::PLL(s) => Some(s.src),
                _ => None,
            },
            pll_psc: match self.sysclk {
                clocking::SysClkSource::PLL(s) => Some(s.m),
                _ => None,
            },
            ppre1: ppre1,
            ppre2: ppre2,
        }
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy)]
pub struct Clocks {
    ///Frequency of AHB bus (HCLK).
    pub hclk: Hertz,
    ///Frequency of APB1 bus (PCLK1).
    pub pclk1: Hertz,
    ///Frequency of APB2 bus (PCLK2).
    pub pclk2: Hertz,
    ///Frequency of System clocks (SYSCLK).
    pub sysclk: Hertz,
    /// Clock source to drive PLL modules
    pub pll_src: Option<clocking::PLLClkSource>,
    /// PLL clock source prescaler, "M" in the clock tree
    pub pll_psc: Option<u8>,
    ///APB1 prescaler
    pub ppre1: u8,
    ///APB2 prescaler
    pub ppre2: u8,
}

impl Clocks {
    /// Returns the frequency of the AHB
    pub fn hclk(&self) -> Hertz {
        self.hclk
    }

    /// Returns the frequency of the APB1
    pub fn pclk1(&self) -> Hertz {
        self.pclk1
    }

    /// Returns the frequency of the APB2
    pub fn pclk2(&self) -> Hertz {
        self.pclk2
    }

    /// Returns the value of the PCLK1 prescaler
    pub fn ppre1(&self) -> u8 {
        self.ppre1
    }

    // TODO remove `allow`
    /// Returns the value of the PCLK2 prescaler
    #[allow(dead_code)]
    pub fn ppre2(&self) -> u8 {
        self.ppre2
    }

    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
