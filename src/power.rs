//! Power control

use stm32l4x6::{pwr, PWR};

use common::Constrain;

impl Constrain<Power> for PWR {
    fn constrain(self) -> Power {
        Power(())
    }
}

/// Constrained Power control module
pub struct Power(());
impl Power {
    pub fn cr1(&mut self) -> &pwr::CR1 {
        unsafe { &(*PWR::ptr()).cr1 }
    }

    pub fn cr2(&mut self) -> &pwr::CR2 {
        unsafe { &(*PWR::ptr()).cr2 }
    }

    pub fn cr3(&mut self) -> &pwr::CR3 {
        unsafe { &(*PWR::ptr()).cr3 }
    }

    pub fn cr4(&mut self) -> &pwr::CR4 {
        unsafe { &(*PWR::ptr()).cr4 }
    }

    pub fn sr1(&mut self) -> &pwr::SR1 {
        unsafe { &(*PWR::ptr()).sr1 }
    }

    pub fn sr2(&mut self) -> &pwr::SR2 {
        unsafe { &(*PWR::ptr()).sr2 }
    }

    pub fn scr(&mut self) -> &pwr::SCR {
        unsafe { &(*PWR::ptr()).scr }
    }

    /// Removes write protection from Backup Domain Control register.
    pub fn remove_bdp(&mut self) {
        let cr1 = self.cr1();
        if cr1.read().dbp().bit_is_clear() {
            // We need to enable write access
            // to configure clock
            cr1.modify(|_, w| w.dbp().set_bit());
            // Wait for it to take effect
            while cr1.read().dbp().bit_is_clear() {}
        }
    }
}
