//! Reset and Clock Control

use stm32l4x6::{rcc, RCC};
use cast::u32;

use ::cmp;

use common::Constrain;
use time::{Hertz, Clocks};
use flash::ACR;

impl Constrain<Rcc> for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            ahb: AHB(()),
            apb1: APB1(()),
            apb2: APB2(()),
            cfgr: CFGR { hpre: None, ppre1: None, ppre2: None, sys: None }
        }
    }
}

/// Constrained RCC peripheral
pub struct Rcc {
    /// AMBA High-performance Bus (AHB) registers
    pub ahb: AHB,
    /// APB1 peripheral registers
    pub apb1: APB1,
    /// APB2 peripheral registers
    pub apb2: APB2,
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
    hpre: Option<u32>,
    //APB1
    ppre1: Option<u32>,
    //APB2
    ppre2: Option<u32>,
    //System clock
    sys: Option<u32>,
}

impl CFGR {
    /// Sets a frequency for the AHB bus.
    pub fn hpre<T: Into<Hertz>>(mut self, freq: T) -> Self {
        self.hpre = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the APB1 bus.
    pub fn ppre1<T: Into<Hertz>>(mut self, freq: T) -> Self {
        self.ppre1 = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the APB2 bus.
    pub fn ppre2<T: Into<Hertz>>(mut self, freq: T) -> Self {
        self.ppre2 = Some(freq.into().0);
        self
    }

    /// Sets a frequency for the System clock.
    pub fn sys<T: Into<Hertz>>(mut self, freq: T) -> Self {
        self.sys = Some(freq.into().0);
        self
    }

    /// Freezes the clock configuration, making it effective
    pub fn freeze(self, acr: &mut ACR) -> Clocks {
        let pllmul = (2 * self.sys.unwrap_or(HSI)) / HSI;
        let pllmul = cmp::min(cmp::max(pllmul, 2), 16);
        let pllmul_bits = match pllmul {
            2 => None,
            pllmul => Some(pllmul as u8 - 2)
        };

        let sys_clock = pllmul * HSI / 2;
        assert!(sys_clock <= SYS_CLOCK_MAX);

        let hpre_bits = match self.hpre.map(|hpre| sys_clock / hpre) {
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

        let ppre1_bits = match self.ppre1.map(|ppre1| ahb / ppre1) {
            Some(0) => unreachable!(),
            Some(1) => 0b011,
            Some(2) => 0b100,
            Some(3...5) => 0b101,
            Some(6...11) => 0b110,
            _ => 0b1111,
        };

        let ppre1: u8 = 1 << (ppre1_bits - 0b011);
        let apb1 = ahb / u32(ppre1);
        //TODO: assert?

        let ppre2_bits = match self.ppre2.map(|ppre2| ahb / ppre2) {
            Some(0) => unreachable!(),
            Some(1) => 0b011,
            Some(2) => 0b100,
            Some(3...5) => 0b101,
            Some(6...11) => 0b110,
            _ => 0b1111,
        };

        let ppre2: u8 = 1 << (ppre2_bits - 0b011);
        let apb2 = ahb / u32(ppre2);
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
            ahb: Hertz(ahb),
            apb1: Hertz(apb1),
            ppre1,
            apb2: Hertz(apb2),
            ppre2,
            sys: Hertz(sys_clock),
        }
    }
}
