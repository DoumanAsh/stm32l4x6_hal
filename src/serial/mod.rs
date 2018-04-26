
//! Serial

use core::ptr;
use core::marker::PhantomData;

use hal::serial;
use nb;
use stm32l4x6::{USART1, USART2, USART3, UART4, UART5};

use rcc::{APB1, APB2, CCIPR};
use rcc::clocking::{USARTClkSource, InputClock};
use time::Bps;

#[cfg(feature = "STM32L496AG")]
pub mod stm32l496ag;
#[cfg(feature = "STM32L496AG")]
pub use self::stm32l496ag::*;

/// Interrupt event
pub enum Event {
    /// New data has been received
    Rxne,
    /// New data can be sent
    Txe,
}

/// Serial error
#[derive(Debug)]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
    #[doc(hidden)] _Extensible,
}

// FIXME these should be "closed" traits
/// TX pin - DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait TxPin<USART> {}

/// RX pin - DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait RxPin<USART> {}

/// Serial abstraction
pub struct Serial<USART, PINS> {
    usart: USART,
    pins: PINS,
}

/// Serial receiver
pub struct Rx<USART> {
    _usart: PhantomData<USART>,
}

/// Serial transmitter
pub struct Tx<USART> {
    _usart: PhantomData<USART>,
}

macro_rules! hal {
    ($(
        $USARTX:ident: ($usartX:ident, $APB:ident, $enr: ident, $usartXen:ident, $usartXsel:ident),
    )+) => {
        $(
            impl<TX, RX> Serial<$USARTX, (TX, RX)> {
                /// Configures the $USARTX peripheral to provide 8N1 asynchronous serial communication
                /// with an oversampling rate of 16.
                pub fn $usartX(
                    usart: $USARTX,
                    pins: (TX, RX),
                    baud_rate: Bps,
                    clock: USARTClkSource,
                    apb: &mut $APB,
                    ccipr: &mut CCIPR,
                ) -> Self
                where
                    TX: TxPin<$USARTX>,
                    RX: RxPin<$USARTX>,
                {
                    let (clk_f, sel_bits) = match clock {
                        USARTClkSource::PCLK(c)   => (c.freq(), 0b00),
                        USARTClkSource::SYSCLK(f) => (f.into(), 0b01),
                        USARTClkSource::HSI16(c)  => (c.freq(), 0b10),
                        USARTClkSource::LSE       => (32_768,   0b11),
                    };
                    ccipr.inner().modify(|_,w| unsafe {
                        w.$usartXsel().bits(sel_bits)
                    }); // __HAL_RCC_USART2_CONFIG <- HAL_RCCEx_PeriphCLKConfig

                    apb.$enr().modify(|_, w| w.$usartXen().set_bit());
                    while apb.$enr().read().$usartXen().bit_is_clear() {} // __HAL_RCC_USART2_CLK_ENABLE <- HAL_UART_MspInit <- HAL_UART_Init

                    usart.cr1.modify(|_,w| w.ue().clear_bit()); // __HAL_UART_DISABLE in HAL_UART_Init
                    // configuration bits can only be written when the usart peripheral is disabled

                    // From UART_SetConfig:
                    usart.cr1.modify(|_,w| w
                                     .m1().clear_bit()
                                     .m0().clear_bit()    // 8-bit word length
                                     .pce().clear_bit()   // parity control disabled
                                     .te().set_bit()      // enable tx
                                     .re().set_bit()      // enable rx
                                     .over8().clear_bit() // 16x oversampling
                                    );
                    usart.cr2.modify(|_,w| unsafe { w.stop().bits(0b00) });          // 1 stop bit
                    usart.cr3.modify(|_,w| w.rtse().clear_bit().ctse().clear_bit()); // no hardware flow control

                    let brr = clk_f / baud_rate.0; // 40.5.4 USART baud rate generation
                    if brr < 16 {
                        panic!("impossible BRR");
                    }
                    usart.brr.write(|w| unsafe { w.bits(brr) });

                    // In asynchronous mode, the following bits must be kept cleared:
                    // - LINEN and CLKEN bits in the USART_CR2 register,
                    // - SCEN, HDSEL and IREN  bits in the USART_CR3 register.
                    usart.cr2.modify(|_,w| w.linen().clear_bit().clken().clear_bit());
                    usart.cr3.modify(|_,w| w.scen().clear_bit().hdsel().clear_bit().iren().clear_bit());

                    usart.cr1.modify(|_,w| w.ue().set_bit()); // __HAL_UART_ENABLE in HAL_UART_Init

                    while usart.isr.read().teack().bit_is_clear() {} // UART_CheckIdleState in HAL_UART_Init
                    while usart.isr.read().reack().bit_is_clear() {}

                    Serial { usart, pins }
                }

                /// Starts listening for an interrupt event
                pub fn listen(&mut self, event: Event) {
                    match event {
                        Event::Rxne => {
                            self.usart.cr1.modify(|_, w| w.rxneie().set_bit())
                        },
                        Event::Txe => {
                            self.usart.cr1.modify(|_, w| w.txeie().set_bit())
                        },
                    }
                }

                /// Starts listening for an interrupt event
                pub fn unlisten(&mut self, event: Event) {
                    match event {
                        Event::Rxne => {
                            self.usart.cr1.modify(|_, w| w.rxneie().clear_bit())
                        },
                        Event::Txe => {
                            self.usart.cr1.modify(|_, w| w.txeie().clear_bit())
                        },
                    }
                }

                /// Splits the `Serial` abstraction into a transmitter and a receiver half
                pub fn split(self) -> (Tx<$USARTX>, Rx<$USARTX>) {
                    (
                        Tx {
                            _usart: PhantomData,
                        },
                        Rx {
                            _usart: PhantomData,
                        },
                    )
                }

                /// Releases the USART peripheral and associated pins
                pub fn free(self) -> ($USARTX, (TX, RX)) {
                    (self.usart, self.pins)
                }
            }

            impl serial::Read<u8> for Rx<$USARTX> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {
                    // NOTE(unsafe) atomic read with no side effects
                    let isr = unsafe { (*$USARTX::ptr()).isr.read() };

                    Err(if isr.pe().bit_is_set() {
                        nb::Error::Other(Error::Parity)
                    } else if isr.fe().bit_is_set() {
                        nb::Error::Other(Error::Framing)
                    } else if isr.nf().bit_is_set() {
                        nb::Error::Other(Error::Noise)
                    } else if isr.ore().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if isr.rxne().bit_is_set() {
                        // NOTE(read_volatile) see `write_volatile` below
                        return Ok(unsafe {
                            ptr::read_volatile(&(*$USARTX::ptr()).rdr as *const _ as *const _)
                        });
                    } else {
                        nb::Error::WouldBlock
                    })
                }
            }

            impl serial::Write<u8> for Tx<$USARTX> {
                // NOTE(!) See section "29.7 USART interrupts"; the only possible errors during transmission
                // are: clear to send (which is disabled in this case) errors and framing errors (which only
                // occur in SmartCard mode); neither of these apply to our hardware configuration
                type Error = !;

                fn flush(&mut self) -> nb::Result<(), !> {
                    // NOTE(unsafe) atomic read with no side effects
                    let isr = unsafe { (*$USARTX::ptr()).isr.read() };

                    if isr.tc().bit_is_set() {
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }

                fn write(&mut self, byte: u8) -> nb::Result<(), !> {
                    // NOTE(unsafe) atomic read with no side effects
                    let isr = unsafe { (*$USARTX::ptr()).isr.read() };

                    if isr.txe().bit_is_set() {
                        // NOTE(unsafe) atomic write to stateless register
                        // NOTE(write_volatile) 8-bit write that's not possible through the svd2rust API
                        unsafe {
                            ptr::write_volatile(&(*$USARTX::ptr()).tdr as *const _ as *mut _, byte)
                        }
                        Ok(())
                    } else {
                        Err(nb::Error::WouldBlock)
                    }
                }
            }
        )+
    }
}

hal! {
    USART1: (usart1, APB2, enr,  usart1en, usart1sel),
    USART2: (usart2, APB1, enr1, usart2en, usart2sel),
    USART3: (usart3, APB1, enr1, usart3en, usart3sel),
    UART4:  (uart4,  APB1, enr1, uart4en,  uart4sel),
    UART5:  (uart5,  APB1, enr1, uart5en,  uart5sel),
}
