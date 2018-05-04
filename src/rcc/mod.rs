//! Reset and Clock Control

// TODO right now the various configure functions reach into rcc directly. This is bad. Add an
// opaque CR member to RCC, and add methods to CR and BDCR. They should probably take clock source
// variant arguments.

#![deny(missing_docs, unused_results)]

use stm32l4x6::{rcc, PWR, RCC};

use common::Constrain;
use flash::ACR;
use time::Hertz;

pub mod clocking;

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
            ccipr: CCIPR(()),
            csr: CSR(()),
            cfgr: CFGR {
                hclk: None,
                pclk1: None,
                pclk2: None,
                sysclk: clocking::SysClkSource::MSI(clocking::MediumSpeedInternalRC::new(4_000_000, false)),
            },
            icscr: ICSCR(()),
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
    /// Peripherals independent clock configuration register
    pub ccipr: CCIPR,
    /// HW clock configuration.
    pub cfgr: CFGR,
    /// Control/status register.
    pub csr: CSR,
    /// Internal clock sources calibration register
    pub icscr: ICSCR,
}

/// AHB 1-3 register access
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

/// APB1 register access
pub struct APB1(());
impl APB1 {
    /// Access APB1RSTR1 reset register
    pub fn rstr1(&mut self) -> &rcc::APB1RSTR1 {
        unsafe { &(*RCC::ptr()).apb1rstr1 }
    }
    /// Access APB1RSTR2 reset register
    pub fn rstr2(&mut self) -> &rcc::APB1RSTR2 {
        unsafe { &(*RCC::ptr()).apb1rstr2 }
    }

    /// Access APB1ENR1 reset register
    pub fn enr1(&mut self) -> &rcc::APB1ENR1 {
        unsafe { &(*RCC::ptr()).apb1enr1 }
    }
    /// Access APB1ENR2 reset register
    pub fn enr2(&mut self) -> &rcc::APB1ENR2 {
        unsafe { &(*RCC::ptr()).apb1enr2 }
    }
}

/// APB2 register access
pub struct APB2(());
impl APB2 {
    /// Access APB2RSTR reset register
    pub fn rstr(&mut self) -> &rcc::APB2RSTR {
        unsafe { &(*RCC::ptr()).apb2rstr }
    }

    /// Access APB2ENR reset register
    pub fn enr(&mut self) -> &rcc::APB2ENR {
        unsafe { &(*RCC::ptr()).apb2enr }
    }
}

/// Backup domain control register.
///
/// Note that it may be write protected and in order to modify it
/// `Power Control Register` can be accessed to lift protection.
/// See description of CR1's DBP bit in Ch. 5.4.1
///
/// See Reference manual Ch. 6.4.29
pub struct BDCR(());
impl BDCR {
    /// Return a raw pointer to the BDCR register
    #[inline]
    pub fn inner(&mut self) -> &rcc::BDCR {
        unsafe { &(*RCC::ptr()).bdcr }
    }

    /// Resets entire Backup domain.
    ///
    /// Use it when you want to change clock source.
    pub fn reset(&mut self) {
        self.inner().modify(|_, write| write.bdrst().set_bit());
        self.inner().modify(|_, write| write.bdrst().clear_bit());
    }

    /// Returns type of RTC Clock.
    pub fn rtc_clock(&mut self) -> clocking::RtcClkSource {
        match self.inner().read().rtcsel().bits() {
            0 => clocking::RtcClkSource::None,
            1 => clocking::RtcClkSource::LSE,
            2 => clocking::RtcClkSource::LSI,
            3 => clocking::RtcClkSource::HSEDiv32,
            _ => unimplemented!(),
        }
    }

    /// Select clock source for RTC.
    ///
    /// **NOTE:** Once source has been selected, it cannot be changed anymore
    /// unless backup domain is reset.
    pub fn set_rtc_clock(&mut self, clock: clocking::RtcClkSource) {
        self.inner().modify(|_, write| unsafe { write.rtcsel().bits(clock.bits()) });
    }

    /// Sets RTC on/off
    pub fn rtc_enable(&mut self, is_on: bool) {
        self.inner().modify(|_, write| write.rtcen().bit(is_on));
    }

    /// Sets LSE on/off
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

///Peripherals independent clock configuration register
///
///See Reference Manual 6.4.28
pub struct CCIPR(());
impl CCIPR {
    #[inline]
    pub fn inner(&mut self) -> &rcc::CCIPR {
        unsafe { &(*RCC::ptr()).ccipr }
    }
}

///Internal clock sources calibration register
///
///See Reference Manual 6.4.2
pub struct ICSCR(());
impl ICSCR {
    #[inline]
    pub fn inner(&mut self) -> &rcc::ICSCR {
        unsafe { &(*RCC::ptr()).icscr }
    }
}

///Control/Status Register
///
/// See Reference manual Ch. 6.4.29
pub struct CSR(());
impl CSR {
    /// Return a raw pointer to the CSR register
    #[inline]
    pub fn inner(&mut self) -> &rcc::CSR {
        unsafe { &(*RCC::ptr()).csr }
    }

    /// Turns on/off LSI oscillator.
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

/// Maximum value for System clock.
///
/// Reference Ch. 6.2.8
pub const SYS_CLOCK_MAX: u32 = 80_000_000;

/// Clock configuration
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
        }
        self.sysclk = src;
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

        // Reference AN4621 note Figure. 4
        // from 0 wait state to 4
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

        rcc.cfgr
            .modify(|_, w| unsafe { w.ppre2().bits(ppre2_bits).ppre1().bits(ppre1_bits).hpre().bits(hpre_bits).sw().bits(sw_bits) });

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
    /// Frequency of AHB bus (HCLK).
    pub hclk: Hertz,
    /// Frequency of APB1 bus (PCLK1).
    pub pclk1: Hertz,
    /// Frequency of APB2 bus (PCLK2).
    pub pclk2: Hertz,
    /// Frequency of System clocks (SYSCLK).
    pub sysclk: Hertz,
    /// Clock source to drive PLL modules
    pub pll_src: Option<clocking::PLLClkSource>,
    /// PLL clock source prescaler, "M" in the clock tree
    pub pll_psc: Option<u8>,
    /// APB1 prescaler
    pub ppre1: u8,
    /// APB2 prescaler
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
