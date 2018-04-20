//! Reset and Clock Control

use stm32l4x6::{rcc, RCC};

use ::cmp;

use common::Constrain;
use time::Hertz;
use flash::ACR;

impl Constrain<Rcc> for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            ahb: AHB(()),
            apb1: APB1(()),
            apb2: APB2(()),
            bdcr: BDCR(()),
            cfgr: CFGR { hclk: None, pclk1: None, pclk2: None, sysclk: None }
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
    /// HW clock configuration.
    pub cfgr: CFGR
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

#[repr(u8)]
///Available source of clock for RTC
pub enum RtcClockType {
    None = 0,
    ///Low speed external clock. 32kHz.
    LSE = 1,
    ///Low speed internal clock. 32kHz.
    LSI = 2,
    ///High speed external divided by 32.
    HSE = 3
}

///Backup domain control register.
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
        self.inner().modify(|_, write| write.bdrst().clear_bit());
    }

    ///Returns type of RTC Clock.
    pub fn rtc_clock(&mut self) -> RtcClockType {
        match self.inner().read().rtcsel().bits() {
            0 => RtcClockType::None,
            1 => RtcClockType::LSE,
            2 => RtcClockType::LSI,
            3 => RtcClockType::HSE,
            _ => unimplemented!()
        }
    }

    ///Select clock source for RTC.
    ///
    ///**NOTE:** Once source has been selected, it cannot be changed anymore
    ///unless backup domain is reset.
    pub fn set_rtc_clock(&mut self, clock: RtcClockType) {
        self.inner().modify(|_, write| unsafe { write.rtcsel().bits(clock as u8) });
    }

    ///Sets RTC on/off
    pub fn rtc_enable(&mut self, is_on: bool) {
        self.inner().modify(|_, write| match is_on {
            true => write.rtcen().set_bit(),
            false => write.rtcen().clear_bit(),
        });
    }
}

//TODO: what about HSI48?
///HSI16 clock value
///
///Reference manual Ch 6.2 Clocks
pub const HSI: u32 = 16_000_000;
///Maximum value for System clock.
///
///Reference Ch. 6.2.8
pub const SYS_CLOCK_MAX: u32 = 80_000_000;

///Clock configuration
pub struct CFGR {
    //AHB bus frequency
    hclk: Option<u32>,
    //APB1
    pclk1: Option<u32>,
    //APB2
    pclk2: Option<u32>,
    //System clock
    sysclk: Option<u32>,
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

    /// Sets a frequency for the System clock.
    pub fn sysclk<T: Into<Hertz>>(mut self, freq: T) -> Self {
        self.sysclk = Some(freq.into().0);
        self
    }

    /// Freezes the clock configuration, making it effective
    pub fn freeze(self, acr: &mut ACR) -> Clocks {
        let pllmul = (2 * self.sysclk.unwrap_or(HSI)) / HSI;
        let pllmul = cmp::min(cmp::max(pllmul, 2), 16);
        let pllmul_bits = match pllmul {
            2 => None,
            pllmul => Some(pllmul as u8 - 2)
        };

        let sys_clock = pllmul * HSI / 2;
        assert!(sys_clock <= SYS_CLOCK_MAX);

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
        //TODO: assert?

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
        //TODO: assert?

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
        //TODO: assert?

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

        let rcc = unsafe { &*RCC::ptr() };
        if let Some(pllmul_bits) = pllmul_bits {
            // use PLL as source
            rcc.pllcfgr.write(|w| unsafe { w.pllm().bits(pllmul_bits) });
            rcc.cr.write(|w| w.pllon().set_bit());
            while rcc.cr.read().pllrdy().bit_is_clear() {}

            // SW: PLL selected as system clock
            rcc.cfgr.modify(|_, w| unsafe {
                w.ppre2()
                 .bits(ppre2_bits)
                 .ppre1()
                 .bits(ppre1_bits)
                 .hpre()
                 .bits(hpre_bits)
                 .sw()
                 .bits(0b10)
            });
        } else {
            // use HSI as source

            // SW: HSI selected as system clock
            rcc.cfgr.write(|w| unsafe {
                w.ppre2()
                 .bits(ppre2_bits)
                 .ppre1()
                 .bits(ppre1_bits)
                 .hpre()
                 .bits(hpre_bits)
                 .sw()
                 .bits(0b00)
            });
        }

        Clocks {
            hclk: Hertz(ahb),
            pclk1: Hertz(apb1),
            pclk2: Hertz(apb2),
            sysclk: Hertz(sys_clock),
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
    ///APB1 prescaler
    pub ppre1: u8,
    ///APB2 prescaler
    pub ppre2: u8,
}
