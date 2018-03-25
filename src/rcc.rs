//! Reset and Clock Control

use stm32l4x6::{rcc, RCC};

use common::Constrain;

impl Constrain<Rcc> for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            ahb: AHB(())
        }
    }
}

/// Constrained RCC peripheral
pub struct Rcc {
    /// AMBA High-performance Bus (AHB) registers
    pub ahb: AHB
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
