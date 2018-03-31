//! Hardware Timers
use cortex_m::peripheral::SYST;
use cortex_m::peripheral::syst::SystClkSource;
use hal::timer::{CountDown, Periodic};
use nb;

use config::SYST_MAX_RVR;
use time::{Hertz, Clocks};
use rcc::{APB1, APB2};

use cast::{u32, u16};

use stm32l4x6::{
    //advanced timers
    TIM1, TIM8, //stm32l4x6::rcc::apb2enr | apb2rstr:
    //General purpose
    TIM2, TIM3, TIM4, TIM5, //stm32l4x6::rcc::apb1enr1 | apb1rstr1
    TIM15, TIM16, TIM17, //stm32l4x6::rcc::apb2enr | apb2rstr
    //Basic timers
    TIM6, TIM7, //stm32l4x6::rcc::apb1enr1 | apb1rstr1
    // low-power timer
    //LPTIM1 //stm32l4x6::rcc::apb1enr1 | apb1rstr1
};

///Possible timer events
pub enum Event {
    ///Interrupt on timeout. Set by DIER's UIE register.
    Timeout,
}

///HW Timer
pub struct Timer<TIM> {
    clocks: Clocks,
    tim: TIM
}

impl Timer<SYST> {
    pub fn syst<T: Into<Hertz>>(mut syst: SYST, timeout: T, clocks: Clocks) -> Self {
        syst.set_clock_source(SystClkSource::Core);
        let mut timer = Timer { tim: syst, clocks };
        timer.start(timeout);
        timer
    }

    /// Starts listening for an `event`
    pub fn subscribe(&mut self, event: Event) {
        match event {
            Event::Timeout => self.tim.enable_interrupt(),
        }
    }

    /// Stops listening for an `event`
    pub fn unsubscribe(&mut self, event: Event) {
        match event {
            Event::Timeout => self.tim.disable_interrupt(),
        }
    }
}

impl CountDown for Timer<SYST> {
    type Time = Hertz;

    fn start<T: Into<Hertz>>(&mut self, timeout: T) {
        let rvr = self.clocks.sys.0 / timeout.into().0 - 1;

        assert!(rvr < SYST_MAX_RVR);

        self.tim.set_reload(rvr);
        self.tim.clear_current();
        self.tim.enable_counter();
    }

    fn wait(&mut self) -> nb::Result<(), !> {
        match self.tim.has_wrapped() {
            true => Ok(()),
            false => Err(nb::Error::WouldBlock)
        }
    }
}

macro_rules! impl_timer {
    ($($TIMx:ident: [constructor: $timx:ident; $APB:ident: {apb: $apb:ident; $enr:ident: $enr_bit:ident; $rstr:ident: $rstr_bit:ident; ppre: $ppre:ident}])+) => {
        $(
            impl Timer<$TIMx> {
                ///Creates new instance of timer.
                pub fn $timx<T: Into<Hertz>>(tim: $TIMx, timeout: T, clocks: Clocks, apb: &mut $APB) -> Timer<$TIMx> {
                    // enable and reset peripheral to a clean slate state
                    apb.$enr().modify(|_, w| w.$enr_bit().set_bit());
                    apb.$rstr().modify(|_, w| w.$rstr_bit().set_bit());
                    apb.$rstr().modify(|_, w| w.$rstr_bit().clear_bit());

                    let mut timer = Timer {
                        clocks,
                        tim,
                    };
                    timer.start(timeout);

                    timer
                }

                /// Starts listening for an `event`
                pub fn subscribe(&mut self, event: Event) {
                    match event {
                        Event::Timeout => self.tim.dier.write(|w| w.uie().set_bit())
                    }
                }

                /// Stops listening for an `event`
                pub fn unsubscribe(&mut self, event: Event) {
                    match event {
                        Event::Timeout => self.tim.dier.write(|w| w.uie().clear_bit())
                    }
                }

                /// Paused timer and releases the TIM peripheral
                pub fn free(self) -> $TIMx {
                    self.tim.cr1.modify(|_, w| w.cen().clear_bit());
                    self.tim
                }

            }

            impl Periodic for Timer<$TIMx> {}
            impl CountDown for Timer<$TIMx> {
                type Time = Hertz;

                fn start<T: Into<Self::Time>>(&mut self, timeout: T) {
                    //pause
                    self.tim.cr1.modify(|_, w| w.cen().clear_bit());
                    //reset counter's value
                    self.tim.cnt.reset();

                    let frequency = timeout.into().0;

                    //TODO: kinda copy-pasted calcs.
                    //      Generally bits are the same but better to re-check later on.
                    //      TIM2 and TIM5 are 32bit timers so their ARR also can be set with high
                    //      bit which is not influenced by psc though?
                    let ppre = match self.clocks.$ppre {
                        1 => 1,
                        _ => 2
                    };
                    let ticks = self.clocks.$apb.0 * ppre / frequency;

                    let psc = u16((ticks - 1) / (1 << 16)).unwrap();
                    self.tim.psc.write(|w| unsafe { w.psc().bits(psc) });

                    let arr = u16(ticks / u32(psc + 1)).unwrap();
                    self.tim.arr.write(|w| unsafe { w.bits(u32(arr)) });

                    // Trigger an update event to load the prescaler value to the clock
                    self.tim.egr.write(|w| w.ug().set_bit());
                    // The above line raises an update event which will indicate
                    // that the timer is already finnished. Since this is not the case,
                    // it should be cleared
                    self.tim.sr.modify(|_, w| w.uif().clear_bit());

                    // start counter
                    self.tim.cr1.modify(|_, w| w.cen().set_bit());
                }

                 fn wait(&mut self) -> nb::Result<(), !> {
                     match self.tim.sr.read().uif().bit_is_clear() {
                         true => Err(nb::Error::WouldBlock),
                         false => {
                            self.tim.sr.modify(|_, w| w.uif().clear_bit());
                            Ok(())
                         }
                     }
                 }
            }

        )+
    }
}

impl_timer!(
    TIM1: [
        constructor: tim1;
        APB2: {
            apb: apb2;
            enr: tim1en;
            rstr: tim1rst;
            ppre: ppre2
        }
    ]
    TIM8: [
        constructor: tim8;
        APB2: {
            apb: apb2;
            enr: tim8en;
            rstr: tim8rst;
            ppre: ppre2
        }
    ]
    TIM2: [
        constructor: tim2;
        APB1: {
            apb: apb1;
            enr1: tim2en;
            rstr1: tim2rst;
            ppre: ppre1
        }
    ]
    TIM3: [
        constructor: tim3;
        APB1: {
            apb: apb1;
            enr1: tim3en;
            rstr1: tim3rst;
            ppre: ppre1
        }
    ]
    TIM4: [
        constructor: tim4;
        APB1: {
            apb: apb1;
            enr1: tim4en;
            rstr1: tim4rst;
            ppre: ppre1
        }
    ]
    TIM5: [
        constructor: tim5;
        APB1: {
            apb: apb1;
            enr1: tim5en;
            rstr1: tim5rst;
            ppre: ppre1
        }
    ]
    TIM15: [
        constructor: tim15;
        APB2: {
            apb: apb2;
            enr: tim15en;
            rstr: tim15rst;
            ppre: ppre2
        }
    ]
    TIM16: [
        constructor: tim16;
        APB2: {
            apb: apb2;
            enr: tim16en;
            rstr: tim16rst;
            ppre: ppre2
        }
    ]
    TIM17: [
        constructor: tim17;
        APB2: {
            apb: apb2;
            enr: tim17en;
            rstr: tim17rst;
            ppre: ppre2
        }
    ]
    TIM6: [
        constructor: tim6;
        APB1: {
            apb: apb1;
            enr1: tim6en;
            rstr1: tim6rst;
            ppre: ppre1
        }
    ]
    TIM7: [
        constructor: tim7;
        APB1: {
            apb: apb1;
            enr1: tim7en;
            rstr1: tim7rst;
            ppre: ppre1
        }
    ]
);
