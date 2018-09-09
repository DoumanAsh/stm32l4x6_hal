//! LCD module
//!
//! TODO: Work in progress

use stm32l4x6;

use gpio;
use power::Power;
use rcc::clocking::RtcClkSource;
use rcc::{APB1, AHB, BDCR};

use mem;

pub mod config;
pub mod ram;

pub enum ValidationResult {
    /// Valid Frame Rate
    ///
    /// Contains approximate frame rate
    Ok(u32),
    /// RTC clock is not set. Refer to `rcc::BDCR` for how to set.
    ClockNotSet,
    /// Resulting frame rate is outside of range is below minimum ~30Hz
    SmallFrameRate,
    /// Resulting frame rate is outside of range is is above ~100Hz
    BigFrameRate,
}

/// LCD representations that provides access to HW LCD
///
/// Implements destructor that turns off LCD.
pub struct LCD {
    inner: stm32l4x6::LCD,
}

#[inline]
fn calculate_frame_rate(clock_frequency: u32, ps: u32, div: u32, duty: u8) -> u32 {
    // Take duty * 1000, then divide by 1000 to drop floating point part
    (clock_frequency / ((2u32.pow(ps)) * (16 + div)) * match duty {
        0 => 1000,
        1 => 500,
        2 => 333,
        3 => 250,
        4 => 125,
        _ => unreachable!(),
    } / 1000)
}

impl LCD {
    /// Initializes HW for LCD with LSE as clock source
    ///
    /// ## Steps:
    ///
    /// 1. Enable peripheral clocks
    /// 2. Set LSE as RTC clock.
    /// 3. Turn on LCD's clock
    pub fn init_lse(apb1: &mut APB1, ahb: &mut AHB, pwr: &mut Power, bdcr: &mut BDCR) {
        // Enables peripheral clocks
        apb1.enr1().modify(|_, w| w.pwren().set_bit());
        // Enables LCD GPIO
        ahb.enr2().modify(|_, w| {
            w.gpioaen().set_bit()
             .gpioben().set_bit()
             .gpiocen().set_bit()
            //TODO: there are more pins in D/E sections which are currently
            //      board specific

            //w.gpioden().set_bit()
            //w.gpioeen().set_bit()
        });
        let mut gpio = gpio::C::new(ahb);
        let _vlcd = gpio.PC3.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);

        //TODO: For some reason USB Leds get enabled after firing up these alt functions.
        //      AF11 is supposed to be LCD only function, yet why usb leds are on?

        //Enable segments
        //SEG18
        gpio.PC0.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG19
        gpio.PC1.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG20
        gpio.PC2.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG22
        gpio.PC4.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG23
        gpio.PC5.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG24
        gpio.PC6.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG25
        gpio.PC7.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG26
        gpio.PC8.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //SEG27
        gpio.PC9.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //COM4/SEG28/40
        gpio.PC10.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //COM5/SEG29/41
        gpio.PC11.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //COM6/SEG30/42
        gpio.PC12.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);

        let mut gpio = gpio::A::new(ahb);
        //SEG0
        gpio.PA1.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG1
        gpio.PA2.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG2
        gpio.PA3.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG3
        gpio.PA6.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG4
        gpio.PA7.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //COM0
        gpio.PA8.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //COM1
        gpio.PA9.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //COM2
        gpio.PA10.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //SEG17
        gpio.PA15.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);

        let mut gpio = gpio::B::new(ahb);
        //SEG5
        gpio.PB0.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG6
        gpio.PB1.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG7
        gpio.PB3.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG8
        gpio.PB4.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG9
        gpio.PB5.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG21
        gpio.PB7.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrl);
        //SEG16
        gpio.PB8.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //COM3
        gpio.PB9.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //SEG10
        gpio.PB10.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //SEG11
        gpio.PB11.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //SEG12
        gpio.PB12.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //SEG13
        gpio.PB13.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //SEG14
        gpio.PB14.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);
        //SEG15
        gpio.PB15.into_alt_fun::<gpio::AF11>(&mut gpio.moder, &mut gpio.afrh);

        // Configures RTC clock
        pwr.remove_bdp();
        // TODO: Reset BDCR to change clock?
        bdcr.lse_enable(true);
        bdcr.set_rtc_clock(RtcClkSource::LSE);

        // Turn LCD's clock
        apb1.enr1().modify(|_, w| w.lcden().set_bit());
    }

    /// Initializes LCD
    ///
    /// ## Requirements
    ///
    /// * Pre-initialize HW using `init` method.
    /// * Configure LCD GPIO pins as alternative functions. You should check board's documentation
    /// for pins.
    /// * Verify configuration through `validate` function
    ///
    /// ## Steps:
    ///
    /// 1. Turns off.
    /// 2. Reset RAM registers and set update request.
    /// 3. Performs configuration.
    /// 4. Turns on.
    pub fn new(lcd: stm32l4x6::LCD, config: config::Config) -> Self {
        let mut lcd = Self { inner: lcd };

        lcd.off();

        lcd.reset_ram();
        lcd.update_request();

        lcd.configure(config);

        lcd.on();

        // Wait for LCD to get enabled
        while lcd.inner.sr.read().ens().bit_is_clear() {}
        // Wait for LCD to get ready
        while lcd.inner.sr.read().rdy().bit_is_clear() {}

        lcd
    }

    /// Performs validation of settings.
    ///
    /// HSE clock is not supported yet...
    pub fn validate(lcd: &mut stm32l4x6::LCD, bdcr: &mut BDCR, configuration: &config::Config) -> ValidationResult {
        let clock_frequency: u32 = match bdcr.rtc_clock().freq(None) {
            Some(f) => f,
            None => return ValidationResult::ClockNotSet,
        };

        let ps = configuration.prescaler.as_ref().map(|val| *val as u8).unwrap_or(lcd.fcr.read().ps().bits()) as u32;
        let div = configuration.divider.as_ref().map(|val| *val as u8).unwrap_or(lcd.fcr.read().div().bits()) as u32;
        let duty = configuration.duty.as_ref().map(|val| *val as u8).unwrap_or(lcd.cr.read().duty().bits());

        let frame_rate = calculate_frame_rate(clock_frequency, ps, div, duty);

        if frame_rate < 29 {
            ValidationResult::SmallFrameRate
        } else if frame_rate > 110 {
            ValidationResult::BigFrameRate
        } else {
            ValidationResult::Ok(frame_rate)
        }
    }

    #[inline]
    /// Returns whether LCD is enabled or not
    pub fn is_enabled(&mut self) -> bool {
        self.inner.sr.read().ens().bit_is_set()
    }

    #[inline]
    /// Returns whether LCD is ready or not
    pub fn is_ready(&mut self) -> bool {
        self.inner.sr.read().rdy().bit_is_set()
    }

    /// Performs LCD's configuration
    pub fn configure(&mut self, config: config::Config) {
        let config::Config {
            prescaler,
            divider,
            blink_mode,
            blink_freq,
            contrast,
            dead_time,
            pulse_duration,
            high_drive,
            bias,
            duty,
            mux_segment,
            voltage_source,
        } = config;

        self.inner.fcr.modify(|_, w| {
            if let Some(prescaler) = prescaler {
                unsafe {
                    w.ps().bits(prescaler as u8);
                }
            }
            if let Some(div) = divider {
                unsafe {
                    w.div().bits(div as u8);
                }
            }
            if let Some(blink) = blink_mode {
                unsafe {
                    w.blink().bits(blink as u8);
                }
            }
            if let Some(blinkf) = blink_freq {
                unsafe {
                    w.blinkf().bits(blinkf as u8);
                }
            }
            if let Some(contrast) = contrast {
                unsafe {
                    w.cc().bits(contrast as u8);
                }
            }
            if let Some(dead) = dead_time {
                unsafe {
                    w.dead().bits(dead as u8);
                }
            }
            if let Some(pulse) = pulse_duration {
                unsafe {
                    w.pon().bits(pulse as u8);
                }
            }
            match high_drive {
                Some(config::HighDrive::On) => w.hd().set_bit(),
                Some(config::HighDrive::Off) => w.hd().clear_bit(),
                _ => w,
            }
        });

        // Wait for FCR to sync
        while self.inner.sr.read().fcrsf().bit_is_clear() {}

        self.inner.cr.modify(|_, w| {
            if let Some(bias) = bias {
                unsafe {
                    w.bias().bits(bias as u8);
                }
            }
            if let Some(duty) = duty {
                unsafe {
                    w.duty().bits(duty as u8);
                }
            }
            match voltage_source {
                Some(config::VoltageSource::Internal) => w.vsel().set_bit(),
                Some(config::VoltageSource::External) => w.vsel().clear_bit(),
                _ => w,
            };
            match mux_segment {
                Some(config::MuxSegment::On) => w.mux_seg().set_bit(),
                Some(config::MuxSegment::Off) => w.mux_seg().clear_bit(),
                _ => w,
            }
        });
    }

    #[inline]
    /// Resets LCD's RAM.
    ///
    /// To have effect user must request update.
    pub fn reset_ram(&mut self) {
        self.inner.ram_com0.reset();
        self.inner.ram_com1.reset();
        self.inner.ram_com2.reset();
        self.inner.ram_com3.reset();
        self.inner.ram_com4.reset();
        self.inner.ram_com5.reset();
        self.inner.ram_com6.reset();
        self.inner.ram_com7.reset();
    }

    #[inline]
    /// Requests to transfer written data to buffer by setting SR's UDR bit
    ///
    /// Note: Once set, it can be cleared only by hardware
    /// In addition to that until value is cleared, RAM is write-protected.
    ///
    /// No update can occur until display shall be enabled.
    pub fn update_request(&mut self) {
        self.inner.sr.modify(|_, w| w.udr().set_bit())
    }

    #[inline]
    /// Turns LCD on by setting CR's EN bit
    pub fn on(&mut self) {
        self.inner.cr.modify(|_, w| w.lcden().set_bit())
    }

    #[inline]
    /// Turns LCD off by clearing CR's EN bit
    pub fn off(&mut self) {
        self.inner.cr.modify(|_, w| w.lcden().clear_bit())
    }

    /// Starts listening for an `event`
    pub fn subscribe(&mut self, event: config::Event) {
        self.inner.fcr.modify(|_, w| match event {
            config::Event::StartFrame => w.sofie().set_bit(),
            config::Event::UpdateDone => w.uddie().set_bit(),
        })
    }

    /// Stops listening for an `event`
    pub fn unsubscribe(&mut self, event: config::Event) {
        self.inner.fcr.modify(|_, w| match event {
            config::Event::StartFrame => w.sofie().clear_bit(),
            config::Event::UpdateDone => w.uddie().clear_bit(),
        })
    }

    /// Writes into RAM by index.
    pub fn write_ram<I: self::ram::Index>(&mut self, data: u32) {
        I::write(self, data)
    }

    pub fn into_raw(mut self) -> stm32l4x6::LCD {
        // We cannot move out of value that implements Drop
        // so let's trick it and since underlying LCD doesn't implement Drop it is safe.
        let mut result = unsafe { mem::uninitialized::<stm32l4x6::LCD>() };
        mem::swap(&mut result, &mut self.inner);
        mem::forget(self);

        result
    }
}

impl Drop for LCD {
    fn drop(&mut self) {
        self.off();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn calculate_frame_rate() {
        use super::config;

        // Reference manual Ch. 25.3.2 Table 160
        let frame_rate = super::calculate_frame_rate(32_768, config::Prescaler::PS_8 as u32, config::Divider::DIV_17 as u32, config::Duty::OneTo8 as u8);
        assert_eq!(frame_rate, 30);
        let frame_rate = super::calculate_frame_rate(32_768, 4, 1, config::Duty::OneTo4 as u8);
        assert_eq!(frame_rate, 30);
        let frame_rate = super::calculate_frame_rate(32_768, 4, 6, config::Duty::OneTo3 as u8);
        assert_eq!(frame_rate, 30);
        let frame_rate = super::calculate_frame_rate(32_768, 4, 6, config::Duty::OneTo3 as u8);
        assert_eq!(frame_rate, 30);
        let frame_rate = super::calculate_frame_rate(32_768, 5, 1, config::Duty::OneTo2 as u8);
        assert_eq!(frame_rate, 30);
        let frame_rate = super::calculate_frame_rate(32_768, config::Prescaler::PS_64 as u32, config::Divider::DIV_17 as u32, config::Duty::Static as u8);
        assert_eq!(frame_rate, 30);

        let frame_rate = super::calculate_frame_rate(32_768, 1, 4, config::Duty::OneTo8 as u8);
        assert_eq!(frame_rate, 102);
        let frame_rate = super::calculate_frame_rate(32_768, 2, 4, config::Duty::OneTo4 as u8);
        assert_eq!(frame_rate, 102);
        let frame_rate = super::calculate_frame_rate(32_768, 2, 11, config::Duty::OneTo3 as u8);
        assert_eq!(frame_rate, 100);
        let frame_rate = super::calculate_frame_rate(32_768, 4, 4, config::Duty::Static as u8);
        assert_eq!(frame_rate, 102);

        let frame_rate = super::calculate_frame_rate(1_000_000, 6, 3, config::Duty::OneTo8 as u8);
        assert_eq!(frame_rate, 102);
        let frame_rate = super::calculate_frame_rate(1_000_000, 8, 3, config::Duty::OneTo2 as u8);
        assert_eq!(frame_rate, 102);
    }
}
