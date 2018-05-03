//! Configuration module
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;

use cmp;

use rcc::Clocks;

/// Max possible value to set on SYST's RVR register.
///
/// SysTick is 24-bit timer.
/// Reference: http://infocenter.arm.com/help/topic/com.arm.doc.dui0553a/Babieigh.html
pub const SYST_MAX_RVR: u32 = (1 << 24);

/// Extension to configure SYST
pub trait SysClockConfig {
    /// Sets reload value in microseconds.
    ///
    /// Limited by `SYST_MAX_RVR`.
    fn set_reload_us(&mut self, us: u32, clocks: &Clocks);
    /// Sets reload value in milliseconds.
    ///
    /// Limited by `SYST_MAX_RVR`.
    #[inline]
    fn set_reload_ms(&mut self, ms: u32, clocks: &Clocks) {
        self.set_reload_us(ms * 1_000, clocks);
    }
}

impl SysClockConfig for SYST {
    fn set_reload_us(&mut self, us: u32, clocks: &Clocks) {
        let rvr = us * (clocks.sysclk.0 / 1_000_000);
        let rvr = cmp::min(rvr, SYST_MAX_RVR);

        self.set_clock_source(SystClkSource::Core);
        self.set_reload(rvr);
    }
}
