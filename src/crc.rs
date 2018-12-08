//!CRC module

use stm32l4x6::CRC as Inner;
use crate::rcc::AHB;

use core::ptr;
use core::ops;
use core::hash::Hasher;

///Initial value of polynomial.
pub const DEFAULT_POLY: u32 = 0x04C11DB7;
///Initial value for CRC's INIT
pub const DEFAULT_INIT: u32 = 0xFFFF_FFFF;

#[derive(Copy, Clone)]
#[repr(u8)]
///Describes possible polynomial sizes
pub enum PolySize {
    ///Uses 32 bits
    Word = 0x00,
    ///Uses 16 bits
    Half = 0x01,
    ///Uses 8 bits
    Byte = 0x10,
    ///Uses 7 bits
    SevenBit = 0x11,
}

#[derive(Copy, Clone)]
#[repr(u8)]
///Describes possible ways to reverse input
///
///For example: input data 0x1A2B3C4D is used for CRC calculation as:
///
///- 0x58D43CB2 with bit-reversal done by byte
///- 0xD458B23C with bit-reversal done by half-word
///- 0xB23CD458 with bit-reversal done on the full word
pub enum ReverseInput {
    ///Bit order remains the same
    None = 0x00,
    ///Reversal byte by byte
    Byte = 0x01,
    ///Reversal by 2 bytes
    Half = 0x10,
    ///Reversal by 4 bytes
    Word = 0x11,
}

///CRC module
///
///The default polynomial value is the CRC-32 (Ethernet) polynomial: 0x4C11DB7
pub struct CRC {
    inner: Inner
}

impl CRC {
    ///Enables CRC peripheral.
    pub fn enable(ahb: &mut AHB) {
        ahb.enr1().modify(|_, w| w.crcen().set_bit());
        ahb.rstr1().modify(|_, w| w.crcrst().set_bit());
        ahb.rstr1().modify(|_, w| w.crcrst().clear_bit());
    }

    ///Disables CRC peripheral
    pub fn disable(ahb: &mut AHB) {
        ahb.enr1().modify(|_, w| w.crcen().clear_bit());
    }

    ///Creates new instance of CRC calculator.
    ///
    ///Takes ownership over device CRC
    pub fn new(inner: Inner) -> Self {
        Self {
            inner,
        }
    }

    ///Sets new value to polynomial
    ///
    ///To obtain a reliable CRC calculation, the change on-fly of the polynomial value or size can
    ///not be performed during a CRC calculation. As a result, if a CRC calculation is ongoing, the
    ///application must either reset it or perform a CRC_DR read before changing the polynomia
    pub fn set_poly(&mut self, poly: u32) -> &mut Self {
        self.inner.pol.write(|w| unsafe { w.polynomialcoefficients().bits(poly) } );
        self
    }

    ///Sets initial value
    pub fn set_init(&mut self, init: u32) -> &mut Self {
        self.inner.init.write(|w| unsafe { w.crc_init().bits(init) } );
        self
    }

    ///Sets whether to reverse output
    pub fn reverse_output(&mut self, value: bool) -> &mut Self {
        self.inner.cr.modify(|_, w| w.rev_out().bit(value));
        self
    }

    ///Sets polynomial size
    pub fn set_poly_size(&mut self, value: PolySize) -> &mut Self {
        self.inner.cr.modify(|_, w| unsafe { w.polysize().bits(value as u8) });
        self
    }

    ///Sets polynomial size
    pub fn reverse_input(&mut self, value: ReverseInput) -> &mut Self {
        self.inner.cr.modify(|_, w| unsafe { w.rev_in().bits(value as u8) });
        self
    }

    ///Resets CRC
    pub fn reset(&self) {
        self.inner.cr.modify(|_, w| w.reset().set_bit());
    }

    ///Retrieves current result
    pub fn result(&self) -> u32 {
        unsafe {
            ptr::read_volatile(self.dr() as *mut u32)
        }
    }

    ///Consumes self and returns device's CRC
    pub fn into_raw(self) -> Inner {
        self.inner
    }

    fn dr(&self) -> *const u8 {
        &self.inner.dr as *const _ as *const u8
    }
}

impl ops::AddAssign<u32> for CRC {
    fn add_assign(&mut self, value: u32) {
        unsafe {
            ptr::write_volatile(self.dr() as *mut u32, value)
        }
    }
}

impl ops::AddAssign<u16> for CRC {
    fn add_assign(&mut self, value: u16) {
        unsafe {
            ptr::write_volatile(self.dr() as *mut u16, value)
        }
    }
}

impl ops::AddAssign<u8> for CRC {
    fn add_assign(&mut self, value: u8) {
        unsafe {
            ptr::write_volatile(self.dr() as *mut u8, value)
        }
    }
}

impl Hasher for CRC {
    #[inline]
    fn finish(&self) -> u64 {
        self.result() as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut bytes = bytes;

        loop {
            match bytes.len() {
                0 => break,
                1 => {
                    *self += bytes[0];
                    break;
                },
                2 => {
                    *self += unsafe { *(&bytes as *const _ as *const u16) };
                    break;
                }
                3 => {
                    *self += unsafe { *(&bytes as *const _ as *const u16) };
                    *self += bytes[2];
                    break;
                },
                4 => {
                    *self += unsafe { *(&bytes as *const _ as *const u32) };
                    break;
                },
                _ => {
                    *self += unsafe { *(&bytes as *const _ as *const u32) };
                    bytes = &bytes[4..]
                }
            }
        }
    }

    #[inline]
    fn write_u8(&mut self, value: u8) {
        *self += value;
    }

    #[inline]
    fn write_u16(&mut self, value: u16) {
        *self += value;
    }

    #[inline]
    fn write_u32(&mut self, value: u32) {
        *self += value;
    }
}
