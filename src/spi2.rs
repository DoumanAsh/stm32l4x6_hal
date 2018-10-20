//! Serial Peripheral Interface (SPI) module.

use ::nb;
use ::hal::spi::{FullDuplex, Mode, Phase, Polarity};
use ::stm32l4x6::{SPI1, SPI2, SPI3};

use ::time::Hertz;
use ::rcc::{APB1, APB2, Clocks};

use ::ptr;

///Describes GPIO Pins to be used by SPI.
pub mod gpio {
    ///SPI1 Pins
    ///
    ///Uses AF5
    pub mod spi1 {
        pub mod nss {
            pub use ::gpio::{PA4, PA15};
            #[cfg(feature = "STM32L476VG")]
            pub use ::gpio::stm32l476vg::gpio::{PE12};
        }
        pub mod sck {
            pub use ::gpio::{PA5, PB3};
            #[cfg(feature = "STM32L476VG")]
            pub use ::gpio::stm32l476vg::gpio::{PE13};
        }
        pub mod miso {
            pub use ::gpio::{PA6, PB4};
            #[cfg(feature = "STM32L476VG")]
            pub use ::gpio::stm32l476vg::gpio::{PE14};
        }
        pub mod mosi {
            pub use ::gpio::{PA7, PB5};
            #[cfg(feature = "STM32L476VG")]
            pub use ::gpio::stm32l476vg::gpio::{PE15};
        }
    }

    ///SPI2 Pins
    ///
    ///Uses AF5
    pub mod spi2 {
        pub mod nss {
            pub use ::gpio::{PB9, PB12};
        }
        pub mod sck {
            pub use ::gpio::{PB10, PB13};
        }
        pub mod miso {
            pub use ::gpio::{PB14, PC2};
        }
        pub mod mosi {
            pub use ::gpio::{PB15, PC3};
        }
    }

    ///SPI3 Pins
    ///
    ///Uses AF6.
    ///Can overlap with others.
    pub mod spi3 {
        pub mod nss {
            pub use ::gpio::{PA4, PA15};
        }

        pub mod sck {
            pub use ::gpio::{PB3, PC10};
        }

        pub mod miso {
            pub use ::gpio::{PB4, PC11};
        }

        pub mod mosi {
            pub use ::gpio::{PB15, PC12};
        }
    }
}

use self::gpio::spi1::sck::*;
use self::gpio::spi1::miso::*;
use self::gpio::spi1::mosi::*;
use self::gpio::spi2::sck::*;
use self::gpio::spi2::miso::*;
use self::gpio::spi2::mosi::*;
use self::gpio::spi3::sck::*;
use self::gpio::spi3::miso::*;
use self::gpio::spi3::mosi::*;

///Describes set of SPI pins
pub trait Pins<SPI, APB> where Self: Sized {
    ///Creates SPI out of pins.
    ///
    ///## Arguments:
    ///
    ///- `spi` - stm32l4x6 SPI's struct.
    ///- `freq` - SPI's frequency.
    ///- `mode` - SPI's mode to use.
    ///- `apb` - APB that corresponds to used pins.
    ///- `clocks` - Used to retrieve configured clock's frequency.
    fn spi(self, spi: SPI, freq: Hertz, mode: Mode, apb: &mut APB, clocks: &Clocks) -> Spi<SPI, Self>;
}

macro_rules! impl_spi_pin {
    ($($SPIx:ident = {
        list: [$($PIN:ident,)+],
        config: {
            APB: {
                name: $APB:ident,
                enr: $enrX:ident,
                rstr: $rstrX:ident,
                en: $spiXen:ident,
                rst: $spiXrst:ident,
                pclk: $pclkX:ident,
            },
            AF: $AF:ident
        }
    },)+) => {
        $(
            impl Pins<$SPIx, $APB> for ($($PIN<::gpio::$AF>,)+) {
                fn spi(self, spi: $SPIx, freq: Hertz, mode: Mode, apb: &mut $APB, clocks: &Clocks) -> Spi<$SPIx, Self> {
                    // Reference: Ch. 42.4.7 Configuration of SPI

                    // enable and/or reset SPI
                    apb.$enrX().modify(|_, w| w.$spiXen().set_bit());
                    apb.$rstrX().modify(|_, w| w.$spiXrst().set_bit());
                    apb.$rstrX().modify(|_, w| w.$spiXrst().clear_bit());

                    //Confire CR1
                    let br = match clocks.$pclkX().0 / freq.0 {
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

                    spi.cr1.write(|w| unsafe {
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

                    //Configure CR2
                    spi.cr2.write(|w| unsafe {
                        //Data size 8 bit
                        w.ds().bits(0b111)
                         .ssoe().set_bit()
                         //RXNE event is generated if the FIFO level is greater than or equal to 1/4 (8-bit)
                         .frxth().set_bit()
                    });

                    Spi {
                        spi,
                        pins: self
                    }
                }
            }

            impl FullDuplex<u8> for Spi<$SPIx, ($($PIN<::gpio::$AF>,)+)> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {
                    let sr = self.spi.sr.read();

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
                            ptr::read_volatile(&self.spi.dr as *const _ as *const u8)
                        });
                    } else {
                        nb::Error::WouldBlock
                    })
                }

                fn send(&mut self, byte: u8) -> nb::Result<(), Error> {
                    let sr = self.spi.sr.read();

                    Err(if sr.ovr().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.modf().bit_is_set() {
                        nb::Error::Other(Error::ModeFault)
                    } else if sr.crcerr().bit_is_set() {
                        nb::Error::Other(Error::Crc)
                    } else if sr.txe().bit_is_set() {
                        // NOTE(write_volatile) see note above
                        unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut u8, byte) }
                        return Ok(());
                    } else {
                        nb::Error::WouldBlock
                    })
                }
            }

            impl ::hal::blocking::spi::transfer::Default<u8> for Spi<$SPIx, ($($PIN<::gpio::$AF>,)+)> {}

            impl ::hal::blocking::spi::write::Default<u8> for Spi<$SPIx, ($($PIN<::gpio::$AF>,)+)> {}
        )+
    };
}

//Configure GPIO for MOSI, MISO and SCK pins
impl_spi_pin!(
    SPI1 = {
        list: [PA5, PA6, PA7,],
        config: {
            APB: {
                name: APB2,
                enr: enr,
                rstr: rstr,
                en: spi1en,
                rst: spi1rst,
                pclk: pclk2,
            },
            AF: AF5
        }
    },
    SPI1 = {
        list: [PB3, PB4, PB5,],
        config: {
            APB: {
                name: APB2,
                enr: enr,
                rstr: rstr,
                en: spi1en,
                rst: spi1rst,
                pclk: pclk2,
            },
            AF: AF5
        }
    },
    SPI2 = {
        list: [PB10, PB14, PB15,],
        config: {
            APB: {
                name: APB1,
                enr: enr1,
                rstr: rstr1,
                en: spi2en,
                rst: spi3rst,
                pclk: pclk1,
            },
            AF: AF5
        }
    },
    SPI2 = {
        list: [PB13, PC2, PC3,],
        config: {
            APB: {
                name: APB1,
                enr: enr1,
                rstr: rstr1,
                en: spi2en,
                rst: spi3rst,
                pclk: pclk1,
            },
            AF: AF5
        }
    },
    SPI3 = {
        list: [PB3, PB4, PB15,],
        config: {
            APB: {
                name: APB1,
                enr: enr1,
                rstr: rstr1,
                //Dunno why but svd has sp3en, not spi3en
                en: sp3en,
                rst: spi3rst,
                pclk: pclk1,
            },
            AF: AF6
        }
    },
    SPI3 = {
        list: [PC10, PC11, PC12,],
        config: {
            APB: {
                name: APB1,
                enr: enr1,
                rstr: rstr1,
                en: sp3en,
                rst: spi3rst,
                pclk: pclk1,
            },
            AF: AF6
        }
    },
);

#[cfg(feature = "STM32L476VG")]
impl_spi_pin!(
    SPI1 = {
        list: [PE13, PE14, PE15,],
        config: {
            APB: {
                name: APB2,
                enr: enr,
                rstr: rstr,
                en: spi1en,
                rst: spi1rst,
                pclk: pclk2,
            },
            AF: AF5
        }
    },
);

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
pub struct Spi<SPI, PINS> {
    pub spi: SPI,
    pub pins: PINS,
}

impl<SPI, PINS> Spi<SPI, PINS> {
    ///Consumes self and returns SPI and PINS
    pub fn into_raw(self) -> (SPI, PINS) {
        (self.spi, self.pins)
    }
}
