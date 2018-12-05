//! Serial module with definition of UxART interfaces

use core::ptr;
use core::ops;

use embedded_hal::serial;
pub use stm32l4x6::{USART1, USART2, USART3};

use crate::rcc::{APB1, APB2, Clocks};
use crate::time::{Hertz};
//We should define here only common pins
use crate::gpio::{
    AF7,
    //USART1: TX, RX, CK
    PA9, PA10, PA8,
    PB6, PB7, PB5,
    //USART2: TX, RX, CK
    PA2, PA3, PA4,
    //USART3: TX, RX, CK
    PB10, PB11, PB12,
    PC10, PC11, PC12,
};

/// Interrupt event
#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    /// New data has been received
    Rxne,
    /// New data can be sent
    Txe,
    /// The line has gone idle
    Idle
}

/// Serial error
#[derive(PartialEq, Eq, Debug)]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
}

impl Into<nb::Error<Self>> for Error {
    #[inline]
    fn into(self) -> nb::Error<Self> {
        nb::Error::Other(self)
    }
}

///UxART pin definition
pub trait Pin {
    ///UART index
    const UART_IDX: u8;

    fn does_belong(idx: u8) -> bool {
        Self::UART_IDX == idx
    }
}

///Dummy Pin to be used when you don't need CK
pub struct DummyPin;
impl Pin for DummyPin {
    const UART_IDX: u8 = 0;

    fn does_belong(_: u8) -> bool {
        true
    }
}

///TX pin
///
///Responsible for transmitting data
pub trait TX: Pin {}
///RX pin
///
///Responsible for receiving data
pub trait RX: Pin {}
///Clock pin
///
///Outputs the transmitter data clock for synchronous transmission
pub trait CK: Pin {}

//impl it only for CK
//other pins must be always used
impl CK for DummyPin {}

macro_rules! impl_pins_trait {
    ($IDX:expr => {
        TRAIT: $TRAIT:ident,
        AF: $AFx:ident,
        PINS: [$($PIN:ident,)+]
    }) => {
        $(
            impl Pin for $PIN<$AFx> {
                const UART_IDX: u8 = $IDX;
            }

            impl $TRAIT for $PIN<$AFx> {
            }
        )+
    }
}

impl_pins_trait!(1 => {
    TRAIT: TX,
    AF: AF7,
    PINS: [PA9, PB6,]
});
impl_pins_trait!(1 => {
    TRAIT: RX,
    AF: AF7,
    PINS: [PA10, PB7,]
});
impl_pins_trait!(1 => {
    TRAIT: CK,
    AF: AF7,
    PINS: [PA8, PB5,]
});

impl_pins_trait!(2 => {
    TRAIT: TX,
    AF: AF7,
    PINS: [PA2,]
});
impl_pins_trait!(2 => {
    TRAIT: RX,
    AF: AF7,
    PINS: [PA3,]
});
impl_pins_trait!(2 => {
    TRAIT: CK,
    AF: AF7,
    PINS: [PA4,]
});

impl_pins_trait!(3 => {
    TRAIT: TX,
    AF: AF7,
    PINS: [PB10, PC10,]
});
impl_pins_trait!(3 => {
    TRAIT: RX,
    AF: AF7,
    PINS: [PB11, PC11,]
});
impl_pins_trait!(3 => {
    TRAIT: CK,
    AF: AF7,
    PINS: [PB12, PC12,]
});

///Describes raw UxART from device crate
pub trait RawSerial where Self: Sized {
    ///Index of Serial, used at runtime to verify that correct PIN is used.
    const IDX: u8;
    ///Type of APB used by Serial interface.
    type APB;

    ///Access register block
    fn registers(&self) -> &crate::stm32l4x6::usart1::RegisterBlock;

    ///Retrieves reference to ISR registers.
    fn isr(&self) -> &crate::stm32l4x6::usart1::ISR {
        &self.registers().isr
    }

    ///Retrieves reference to RDR registers.
    fn rdr(&self) -> &crate::stm32l4x6::usart1::RDR {
        &self.registers().rdr
    }

    ///Retrieves reference to TDR registers.
    fn tdr(&self) -> &crate::stm32l4x6::usart1::TDR {
        &self.registers().tdr
    }

    ///Retrieves reference to CR1 registers
    fn cr1(&self) -> &crate::stm32l4x6::usart1::CR1 {
        &self.registers().cr1
    }

    ///Retrieves reference to BRR registers
    fn brr(&self) -> &crate::stm32l4x6::usart1::BRR {
        &self.registers().brr
    }

    ///Retrieves clock frequency for interface.
    fn get_clock_freq(clocks: &Clocks) -> Hertz;

    ///Turns on interface by setting corresponding bits.
    fn enable(apb: &mut Self::APB);

    ///Turns off interface by setting corresponding bits.
    fn disable(apb: &mut Self::APB);

    /// Starts listening for an interrupt event
    fn subscribe(&self, event: Event) {
        match event {
            Event::Rxne => self.cr1().modify(|_, w| w.rxneie().set_bit()),
            Event::Txe => self.cr1().modify(|_, w| w.txeie().set_bit()),
            Event::Idle => self.cr1().modify(|_, w| w.idleie().set_bit()),
        }
    }

    /// Starts listening for an interrupt event
    fn unsubscribe(&self, event: Event) {
        match event {
            Event::Rxne => self.cr1().modify(|_, w| w.rxneie().clear_bit()),
            Event::Txe => self.cr1().modify(|_, w| w.txeie().clear_bit()),
            Event::Idle => self.cr1().modify(|_, w| w.idleie().clear_bit()),
        }
    }
}

impl RawSerial for USART1 {
    const IDX: u8 = 1;
    type APB = APB2;

    #[inline]
    fn get_clock_freq(clocks: &Clocks) -> Hertz {
        clocks.pclk2()
    }

    fn registers(&self) -> &crate::stm32l4x6::usart1::RegisterBlock {
        unsafe { &(*Self::ptr()) }
    }

    fn enable(apb: &mut Self::APB) {
        // enable and/or reset SPI
        apb.enr().modify(|_, w| w.usart1en().set_bit());
        apb.rstr().modify(|_, w| w.usart1rst().set_bit());
        apb.rstr().modify(|_, w| w.usart1rst().clear_bit());
    }

    fn disable(apb: &mut Self::APB) {
        apb.enr().modify(|_, w| w.usart1en().clear_bit());
    }
}

impl RawSerial for USART2 {
    const IDX: u8 = 2;
    type APB = APB1;

    #[inline]
    fn get_clock_freq(clocks: &Clocks) -> Hertz {
        clocks.pclk1()
    }

    fn registers(&self) -> &crate::stm32l4x6::usart1::RegisterBlock {
        unsafe { &(*Self::ptr()) }
    }

    fn enable(apb: &mut Self::APB) {
        apb.enr1().modify(|_, w| w.usart2en().set_bit());
        apb.rstr1().modify(|_, w| w.usart2rst().set_bit());
        apb.rstr1().modify(|_, w| w.usart2rst().clear_bit());
    }

    fn disable(apb: &mut Self::APB) {
        apb.enr1().modify(|_, w| w.usart2en().clear_bit());
    }
}

impl RawSerial for USART3 {
    const IDX: u8 = 3;
    type APB = APB1;

    #[inline]
    fn get_clock_freq(clocks: &Clocks) -> Hertz {
        clocks.pclk1()
    }

    fn registers(&self) -> &crate::stm32l4x6::usart1::RegisterBlock {
        unsafe { &(*Self::ptr()) }
    }

    fn enable(apb: &mut Self::APB) {
        apb.enr1().modify(|_, w| w.usart3en().set_bit());
        apb.rstr1().modify(|_, w| w.usart3rst().set_bit());
        apb.rstr1().modify(|_, w| w.usart3rst().clear_bit());
    }

    fn disable(apb: &mut Self::APB) {
        apb.enr1().modify(|_, w| w.usart3en().clear_bit());
    }
}

///Serial interface
pub struct Serial<S, TX, RX, CK> {
    pub serial: S,
    pins: (TX, RX, CK)
}

impl<UART: RawSerial, T: TX, R: RX, C: CK> ops::Deref for Serial<UART, T, R, C> {
    type Target = UART;

    fn deref(&self) -> &Self::Target {
        &self.serial
    }
}

impl<UART: RawSerial, T: TX, R: RX> Serial<UART, T, R, DummyPin> {
    #[inline]
    ///Initializes Serial with dummy CK
    pub fn with_dummy(serial: UART, pins: (T, R), baud_rate: u32, clocks: &Clocks, apb: &mut UART::APB) -> Self {
        Self::new(serial, (pins.0, pins.1, DummyPin), baud_rate, clocks, apb)
    }
}

impl<UART: RawSerial, T: TX, R: RX, C: CK> Serial<UART, T, R, C> {
    /// Creates new instance of serial interface
    ///
    /// # Arguments:
    ///
    /// - `serial` - Serial interface.
    /// - `pins` - Pins used by `serial`.
    /// - `baud_rate` - Rate to set for TX and RX pins, See Reference Ch. 40.5.4 for details
    /// - `apb` - APBx corresponding to Serial.
    ///
    /// It takes ownership of raw Serial object and corresponding PINs.
    ///
    /// # Pancis:
    ///
    /// In debug mode the function checks if index of each PIN corresponds to Serial's index.
    pub fn new(serial: UART, pins: (T, R, C), baud_rate: u32, clocks: &Clocks, apb: &mut UART::APB) -> Self {
        //TODO: Baurd can be auto-detected, should be configurable?
        //      See Ch. 40.5.6
        debug_assert!(T::does_belong(UART::IDX));
        debug_assert!(R::does_belong(UART::IDX));
        debug_assert!(C::does_belong(UART::IDX));

        UART::enable(apb);

        //TODO: DMA requires to enable dmat bit
        //      Should configurable

        let brr = UART::get_clock_freq(clocks).0 / baud_rate;
        assert!(brr >= 16, "impossible baud rate");
        serial.brr().write(|w| unsafe { w.bits(brr) });

        //Enables interface(UE), and receiver(RE) with transmitter(TE)
        serial.cr1().write(|w| w.ue().set_bit().re().set_bit().te().set_bit());

        Self {
            serial,
            pins
        }
    }

    ///Re-creates Serial instance from its components.
    ///
    ///Note: it is up to user to ensure that Serial has been created using [new](#method.new) previously
    pub unsafe fn from_raw(serial: UART, pins: (T, R, C)) -> Self {
        Self {
            serial,
            pins
        }
    }

    ///Consumes self and returns Serial and PINS
    pub fn into_raw(self) -> (UART, (T, R, C)) {
        (self.serial, self.pins)
    }
}

impl<UART: RawSerial, T: TX, R: RX, C: CK> serial::Read<u8> for Serial<UART, T, R, C> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Error> {
        let isr = self.serial.isr().read();

        Err(if isr.pe().bit_is_set() {
            Error::Parity.into()
        } else if isr.fe().bit_is_set() {
            Error::Framing.into()
        } else if isr.nf().bit_is_set() {
            Error::Noise.into()
        } else if isr.ore().bit_is_set() {
            Error::Overrun.into()
        } else if isr.rxne().bit_is_set() {
            return Ok(unsafe {
                ptr::read_volatile(self.serial.rdr() as *const _ as *const u8)
            });
        } else {
            nb::Error::WouldBlock
        })
    }
}

impl<UART: RawSerial, T: TX, R: RX, C: CK> serial::Write<u8> for Serial<UART, T, R, C> {
    //TODO: Error handling for advanced use cases?
    type Error = ();

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        let isr = self.serial.isr().read();

        if isr.tc().bit_is_set() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn write(&mut self, byte: u8) -> nb::Result<(), ()> {
        let isr = self.serial.isr().read();

        if isr.txe().bit_is_set() {
            unsafe {
                ptr::write_volatile(self.serial.tdr() as *const _ as *mut u8, byte);
            }
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}
