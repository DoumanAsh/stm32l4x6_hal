//! Serial Peripheral Interface (SPI) module.

use ::nb;
use ::hal::spi::{FullDuplex, Mode, Phase, Polarity};
use ::stm32l4x6::{SPI1, SPI2, SPI3};

use ::time::Hertz;
use ::rcc::{APB1, APB2, Clocks};

use ::ptr;

use ::gpio::{
    AF5,
    AF6, //Used for SPI3
    //SPI1
    //NSS
    //PA4, PA15,
    //SCK
    PA5, PB3,
    //MISO
    PA6, PB4,
    //MOSI
    PA7, PB5,
    //SPI2
    //NSS
    //PB9, PB12,
    //SCK
    PB10, PB13,
    //MISO
    PB14, PC2,
    //MOSI
    PB15, PC3,
    //SPI3
    //NSS
    //PA15
    //SCK
    PC10,
    //MISO
    PC11,
    //MOSI
    PC12,
};

///Describes SCK Pin
pub trait SCK {
    ///SPI index
    const SPI_IDX: u8;
}

///Describes MISO Pin
pub trait MISO {
    ///SPI index
    const SPI_IDX: u8;
}

///Describes MOSI Pin
pub trait MOSI {
    ///SPI index
    const SPI_IDX: u8;
}

macro_rules! impl_pins_trait {
    ($IDX:expr => {
        TRAIT: $TRAIT:ident,
        AF: $AFx:ident,
        PINS: [$($PIN:ident,)+]
    }) => {
        $(
            impl $TRAIT for $PIN<$AFx> {
                const SPI_IDX: u8 = $IDX;
            }
        )+
    }
}

impl_pins_trait!(1 => {
    TRAIT: SCK,
    AF: AF5,
    PINS: [PA5, PB3,]
});
impl_pins_trait!(1 => {
    TRAIT: MISO,
    AF: AF5,
    PINS: [PA6, PB4,]
});
impl_pins_trait!(1 => {
    TRAIT: MOSI,
    AF: AF5,
    PINS: [PA7, PB5,]
});

impl_pins_trait!(2 => {
    TRAIT: SCK,
    AF: AF5,
    PINS: [PB10, PB13,]
});
impl_pins_trait!(2 => {
    TRAIT: MISO,
    AF: AF5,
    PINS: [PB14, PC2,]
});
impl_pins_trait!(2 => {
    TRAIT: MOSI,
    AF: AF5,
    PINS: [PB15, PC3,]
});

impl_pins_trait!(3 => {
    TRAIT: SCK,
    AF: AF6,
    PINS: [PC10,]
});
impl_pins_trait!(3 => {
    TRAIT: MISO,
    AF: AF6,
    PINS: [PC11,]
});
impl_pins_trait!(3 => {
    TRAIT: MOSI,
    AF: AF6,
    PINS: [PC12,]
});

//Reference: Ch. 42.4.7 Configuration of SPI
///Describes raw SPI from device crate
pub trait InnerSpi where Self: Sized {
    ///Index of SPI, used at runtime to verify that correct PIN is used.
    const IDX: u8;
    ///Type of APB used by SPI.
    type APB;

    ///Retrieves Clocks frequency corresponding to SPI.
    fn get_clock_freq(clocks: &Clocks) -> Hertz;

    ///Retrieves CR1 register block.
    fn cr1(&self) -> & ::stm32l4x6::spi1::CR1;

    ///Retrieves CR2 register block.
    fn cr2(&self) -> & ::stm32l4x6::spi1::CR2;

    ///Retrieves SR register block.
    fn sr(&self) -> & ::stm32l4x6::spi1::SR;

    ///Retrieves DR register block.
    fn dr(&self) -> & ::stm32l4x6::spi1::DR;

    ///Configures CR1 register
    fn configure_cr1(&self, freq: Hertz, clocks: &Clocks, mode: Mode) {
        let br = match Self::get_clock_freq(clocks).0 / freq.0 {
            0 => unreachable!(),
            1...2 => 0b000,
            3...5 => 0b001,
            6...11 => 0b010,
            12...23 => 0b011,
            24...39 => 0b100,
            40...95 => 0b101,
            96...191 => 0b110,
            _ => 0b111,
        };

        self.cr1().write(|w| unsafe {
            w.br().bits(br)
             .cpol().bit(mode.polarity == Polarity::IdleHigh)
             .cpha().bit(mode.phase == Phase::CaptureOnSecondTransition)
             //2-line undirectional for Master mode
             .bidimode().clear_bit()
             .lsbfirst().clear_bit()
             //TODO: CRC option?
             .crcen().clear_bit()
             .ssi().set_bit()
             .ssm().set_bit()
             .mstr().set_bit()
        });
    }

    ///Configures CR2 register
    fn configure_cr2(&self) {
        self.cr2().write(|w| unsafe {
            //Data size 8 bit
            w.ds().bits(0b111)
             .ssoe().set_bit()
             //RXNE event is generated if the FIFO level is greater than or equal to 1/4 (8-bit)
             .frxth().set_bit()
        });
    }

    ///Enables SPI
    fn enable(apb: &mut Self::APB);
}

impl InnerSpi for SPI1 {
    const IDX: u8 = 1;
    type APB = APB2;

    #[inline]
    fn get_clock_freq(clocks: &Clocks) -> Hertz {
        clocks.pclk2()
    }

    fn cr1(&self) -> &::stm32l4x6::spi1::CR1 {
        &self.cr1
    }

    fn cr2(&self) -> &::stm32l4x6::spi1::CR2 {
        &self.cr2
    }

    fn sr(&self) -> &::stm32l4x6::spi1::SR {
        &self.sr
    }

    fn dr(&self) -> &::stm32l4x6::spi1::DR {
        &self.dr
    }

    fn enable(apb: &mut Self::APB) {
        // enable and/or reset SPI
        apb.enr().modify(|_, w| w.spi1en().set_bit());
        apb.rstr().modify(|_, w| w.spi1rst().set_bit());
        apb.rstr().modify(|_, w| w.spi1rst().clear_bit());
    }
}

impl InnerSpi for SPI2 {
    const IDX: u8 = 2;
    type APB = APB1;

    #[inline]
    fn get_clock_freq(clocks: &Clocks) -> Hertz {
        clocks.pclk2()
    }

    fn cr1(&self) -> &::stm32l4x6::spi1::CR1 {
        &self.cr1
    }

    fn cr2(&self) -> &::stm32l4x6::spi1::CR2 {
        &self.cr2
    }

    fn sr(&self) -> &::stm32l4x6::spi1::SR {
        &self.sr
    }

    fn dr(&self) -> &::stm32l4x6::spi1::DR {
        &self.dr
    }

    fn enable(apb: &mut Self::APB) {
        // enable and/or reset SPI
        apb.enr1().modify(|_, w| w.spi2en().set_bit());
        apb.rstr1().modify(|_, w| w.spi2rst().set_bit());
        apb.rstr1().modify(|_, w| w.spi2rst().clear_bit());
    }
}

impl InnerSpi for SPI3 {
    const IDX: u8 = 3;
    type APB = APB1;

    #[inline]
    fn get_clock_freq(clocks: &Clocks) -> Hertz {
        clocks.pclk2()
    }

    fn cr1(&self) -> &::stm32l4x6::spi1::CR1 {
        &self.cr1
    }

    fn cr2(&self) -> &::stm32l4x6::spi1::CR2 {
        &self.cr2
    }

    fn sr(&self) -> &::stm32l4x6::spi1::SR {
        &self.sr
    }

    fn dr(&self) -> &::stm32l4x6::spi1::DR {
        &self.dr
    }

    fn enable(apb: &mut Self::APB) {
        // enable and/or reset SPI
        apb.enr1().modify(|_, w| w.sp3en().set_bit());
        apb.rstr1().modify(|_, w| w.spi3rst().set_bit());
        apb.rstr1().modify(|_, w| w.spi3rst().clear_bit());
    }
}


/// SPI errors.
pub enum Error {
    /// Overrun occurred
    Overrun,
    /// Mode fault
    ModeFault,
    /// CRC cheksum error.
    Crc,
}

/// SPI
pub struct Spi<SPI, SCK, MISO, MOSI> {
    spi: SPI,
    pins: (SCK, MISO, MOSI),
}

impl<SPI: InnerSpi, S: SCK, MI: MISO, MO: MOSI> Spi<SPI, S, MI, MO> {
    /// Creates new instance of SPI.
    ///
    /// It takes ownership of raw SPI object and corresponding PINs.
    ///
    /// Function performs following actions:
    ///
    /// - Reset and enable SPI;
    /// - Configure CR1;
    /// - Configure CR2;
    ///
    /// # Pancis:
    ///
    /// In debug mode the function checks if index of each PIN corresponds to SPI's index.
    pub fn new(spi: SPI, pins: (S, MI, MO), freq: Hertz, mode: Mode, clocks: &Clocks, apb: &mut SPI::APB) -> Self {
        debug_assert_eq!(SPI::IDX, S::SPI_IDX);
        debug_assert_eq!(SPI::IDX, MI::SPI_IDX);
        debug_assert_eq!(SPI::IDX, MO::SPI_IDX);

        SPI::enable(apb);

        spi.configure_cr1(freq, clocks, mode);
        spi.configure_cr2();

        Self {
            spi,
            pins
        }
    }

    ///Re-creates SPI instance from its components.
    ///
    ///Note: it is up to user to ensure that SPI has been created using [new](#method.new) previously
    pub unsafe fn from_raw(spi: SPI, pins: (S, MI, MO)) -> Self {
        Self {
            spi,
            pins
        }
    }

    ///Consumes self and returns SPI and PINS
    pub fn into_raw(self) -> (SPI, (S, MI, MO)) {
        (self.spi, self.pins)
    }
}

impl<SPI: InnerSpi, S: SCK, MI: MISO, MO: MOSI> FullDuplex<u8> for Spi<SPI, S, MI, MO> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Error> {
        let sr = self.spi.sr().read();

        Err(if sr.ovr().bit_is_set() {
            nb::Error::Other(Error::Overrun)
        } else if sr.modf().bit_is_set() {
            nb::Error::Other(Error::ModeFault)
        } else if sr.crcerr().bit_is_set() {
            nb::Error::Other(Error::Crc)
        } else if sr.rxne().bit_is_set() {
            // NOTE(read_volatile) read only 1 byte (the svd2rust API only allows
            // reading a half-word)
            return Ok(unsafe {
                ptr::read_volatile(&self.spi.dr() as *const _ as *const u8)
            });
        } else {
            nb::Error::WouldBlock
        })
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Error> {
        let sr = self.spi.sr().read();

        Err(if sr.ovr().bit_is_set() {
            nb::Error::Other(Error::Overrun)
        } else if sr.modf().bit_is_set() {
            nb::Error::Other(Error::ModeFault)
        } else if sr.crcerr().bit_is_set() {
            nb::Error::Other(Error::Crc)
        } else if sr.txe().bit_is_set() {
            // NOTE(write_volatile) see note above
            unsafe { ptr::write_volatile(&self.spi.dr() as *const _ as *mut u8, byte) }
            return Ok(());
        } else {
            nb::Error::WouldBlock
        })
    }
}

impl<SPI: InnerSpi, S: SCK, MI: MISO, MO: MOSI> ::hal::blocking::spi::transfer::Default<u8> for Spi<SPI, S, MI, MO> {}

impl<SPI: InnerSpi, S: SCK, MI: MISO, MO: MOSI> ::hal::blocking::spi::write::Default<u8> for Spi<SPI, S, MI, MO> {}

#[cfg(feature = "STM32L476VG")]
mod stm32l476vg;
