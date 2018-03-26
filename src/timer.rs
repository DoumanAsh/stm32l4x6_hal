//! Hardware Timers
use hal::timer::{CountDown, Periodic};
use nb;

use time::{Hertz, Clocks};
use rcc::{APB1, APB2};

use stm32l4x6::{
    //advanced timers
    TIM1, TIM8, //stm32l4x6::rcc::apb2enr | apb2rstr:
    //General purpose
    TIM2, TIM3, TIM4, TIM5, //stm32l4x6::rcc::apb1enr1 | apb1rstr1
    TIM15, TIM16, TIM17, //stm32l4x6::rcc::apb2enr | apb2rstr
    //Basic timers
    TIM6, TIM7, //stm32l4x6::rcc::apb1enr1 | apb1rstr1
    // low-power timer
    LPTIM1 //stm32l4x6::rcc::apb1enr1 | apb1rstr1
};

///Possible timer events
pub enum Event {
    ///Interrupt on timeout. Set by DIER's UIE register.
    Timeout,
}

///HW Timer
pub struct Timer<TIM> {
    clocks: Clocks,
    tim: TIM,
    timeout: Hertz
}

macro_rules! impl_timer {
    ($($TIMx:ident: [constructor: $timx:ident; $APB:ident: {$enr:ident: $enr_bit:ident; $rstr:ident: $rstr_bit:ident}])+) => {
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
                        timeout: Hertz(0)
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

                    self.timeout = timeout.into();
                    let frequency = self.timeout.0;

                    //TODO: calc timer settings
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
            enr: tim1en;
            rstr: tim1rst
        }
    ]
    TIM8: [
        constructor: tim8;
        APB2: {
            enr: tim8en;
            rstr: tim8rst
        }
    ]
);



