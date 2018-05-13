//! General Purpose Input / Output
//!
//! This module provides common GPIO definitions that are available on all STM32L4x6 packages. By
//! enabling features, you can use definitions for specific chips which may include additional
//! GPIO lines. In that case, you will probably not want to `use` this module directly, but instead
//! use it re-exported by the chip module.

use marker::PhantomData;
use ops::Deref;

use hal::digital::{
    OutputPin,
    StatefulOutputPin,
    toggleable
};

use stm32l4x6;

use rcc::AHB;

/// Input Mode Trait
/// Implemented only for corresponding structs.
///
/// Note: MUST not be implemented by user.
pub trait InputMode {
    fn modify_pupdr_bits(original: u32, offset: u32) -> u32;
}

/// Floating input (type state)
pub struct Floating;
impl InputMode for Floating {
    #[inline]
    fn modify_pupdr_bits(original: u32, offset: u32) -> u32 {
        original & !(0b11 << offset)
    }
}
/// Pulled down input (type state)
pub struct PullDown;
impl InputMode for PullDown {
    #[inline]
    fn modify_pupdr_bits(original: u32, offset: u32) -> u32 {
        (original & !(0b11 << offset)) | (0b10 << offset)
    }
}
/// Pulled up input (type state)
pub struct PullUp;
impl InputMode for PullUp {
    #[inline]
    fn modify_pupdr_bits(original: u32, offset: u32) -> u32 {
        (original & !(0b11 << offset)) | (0b01 << offset)
    }
}
/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Output Mode Trait
/// Implemented only for corresponding structs.
///
/// Note: MUST not be implemented by user.
pub trait OutputMode {
    fn modify_otyper_bits(original: u32, idx: u8) -> u32;
}

/// Push pull output (type state)
pub struct PushPull;
impl OutputMode for PushPull {
    #[inline]
    fn modify_otyper_bits(original: u32, idx: u8) -> u32 {
        original & !(0b1 << idx)
    }
}
/// Open drain output (type state)
pub struct OpenDrain;
impl OutputMode for OpenDrain {
    #[inline]
    fn modify_otyper_bits(original: u32, idx: u8) -> u32 {
        original | (0b1 << idx)
    }
}
/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Alternate Function Trait
/// Implemented only for corresponding structs.
///
/// Note: MUST not be implemented by user.
pub trait AltFun {
    const NUM: u32;
}

/// Alternate function 0 (type state)
pub struct AF0;
impl AltFun for AF0 {
    const NUM: u32 = 0;
}
/// Alternate function 1 (type state)
pub struct AF1;
impl AltFun for AF1 {
    const NUM: u32 = 1;
}
/// Alternate function 2 (type state)
pub struct AF2;
impl AltFun for AF2 {
    const NUM: u32 = 2;
}
/// Alternate function 3 (type state)
pub struct AF3;
impl AltFun for AF3 {
    const NUM: u32 = 3;
}
/// Alternate function 4 (type state)
pub struct AF4;
impl AltFun for AF4 {
    const NUM: u32 = 4;
}
/// Alternate function 5 (type state)
pub struct AF5;
impl AltFun for AF5 {
    const NUM: u32 = 5;
}
/// Alternate function 6 (type state)
pub struct AF6;
impl AltFun for AF6 {
    const NUM: u32 = 6;
}
/// Alternate function 7 (type state)
pub struct AF7;
impl AltFun for AF7 {
    const NUM: u32 = 7;
}
/// Alternate function 8 (type state)
pub struct AF8;
impl AltFun for AF8 {
    const NUM: u32 = 8;
}
/// Alternate function 9 (type state)
pub struct AF9;
impl AltFun for AF9 {
    const NUM: u32 = 9;
}
/// Alternate function 10 (type state)
pub struct AF10;
impl AltFun for AF10 {
    const NUM: u32 = 10;
}
/// Alternate function 11 (type state)
pub struct AF11;
impl AltFun for AF11 {
    const NUM: u32 = 11;
}
/// Alternate function 12 (type state)
pub struct AF12;
impl AltFun for AF12 {
    const NUM: u32 = 12;
}
/// Alternate function 13 (type state)
pub struct AF13;
impl AltFun for AF13 {
    const NUM: u32 = 13;
}
/// Alternate function 14 (type state)
pub struct AF14;
impl AltFun for AF14 {
    const NUM: u32 = 14;
}
/// Alternate function 15 (type state)
pub struct AF15;
impl AltFun for AF15 {
    const NUM: u32 = 15;
}

macro_rules! impl_parts {
    ($($GPIOX:ident, $gpiox:ident;)+) => {
        $(
            use stm32l4x6::$GPIOX;
            impl AFRL<$GPIOX> {
                pub(crate) fn afr(&mut self) -> &stm32l4x6::$gpiox::AFRL {
                    unsafe { &(*$GPIOX::ptr()).afrl }
                }
            }
            impl AFRH<$GPIOX> {
                pub(crate) fn afr(&mut self) -> &stm32l4x6::$gpiox::AFRH {
                    unsafe { &(*$GPIOX::ptr()).afrh }
                }
            }
            impl MODER<$GPIOX> {
                pub(crate) fn moder(&mut self) -> &stm32l4x6::$gpiox::MODER {
                    unsafe { &(*$GPIOX::ptr()).moder }
                }
            }
            impl OTYPER<$GPIOX> {
                pub(crate) fn otyper(&mut self) -> &stm32l4x6::$gpiox::OTYPER {
                    unsafe { &(*$GPIOX::ptr()).otyper }
                }
            }
            impl PUPDR<$GPIOX> {
                pub(crate) fn pupdr(&mut self) -> &stm32l4x6::$gpiox::PUPDR {
                    unsafe { &(*$GPIOX::ptr()).pupdr }
                }
            }
         )+
    }
}

macro_rules! impl_gpio {
    ($name:ident, $GPIOX:ident, $gpioen:ident, $gpiorst:ident) => {
        impl_gpio!($name, $GPIOX, $gpioen, $gpiorst, AFRL: [], AFRH: []);
    };
    ($name:ident, $GPIOX:ident, $gpioen:ident, $gpiorst:ident, AFRL: [$($PXiL:ident, $iL:expr;)*]) => {
        impl_gpio!($name, $GPIOX, $gpioen, $gpiorst, AFRL: [$($PXiL, $iL;)*], AFRH: []);
    };
    ($name:ident, $GPIOX:ident, $gpioen:ident, $gpiorst:ident, AFRL: [$($PXiL:ident, $iL:expr;)*], AFRH: [$($PXiH:ident, $iH:expr;)*]) => {
        impl_pins!($GPIOX, AFRL: [$($PXiL, $iL;)*]);
        impl_pins!($GPIOX, AFRH: [$($PXiH, $iH;)*]);

        #[allow(non_snake_case)]
        ///GPIO
        pub struct $name {
            /// Opaque AFRH register
            pub afrh: AFRH<$GPIOX>,
            /// Opaque AFRL register
            pub afrl: AFRL<$GPIOX>,
            /// Opaque MODER register
            pub moder: MODER<$GPIOX>,
            /// Opaque OTYPER register
            pub otyper: OTYPER<$GPIOX>,
            /// Opaque PUPDR register
            pub pupdr: PUPDR<$GPIOX>,
            $(
                /// Pin
                pub $PXiL: $PXiL<Input<Floating>>,
            )*
            $(
                /// Pin
                pub $PXiH: $PXiH<Input<Floating>>,
            )*
        }

        impl $name {
            ///Creates new instance of GPIO by enabling it on AHB register
            pub fn new(ahb: &mut AHB) -> Self {
                ahb.enr2().modify(|_, w| w.$gpioen().set_bit());
                ahb.rstr2().modify(|_, w| w.$gpiorst().set_bit());
                ahb.rstr2().modify(|_, w| w.$gpiorst().clear_bit());

                Self {
                    afrh: AFRH(PhantomData),
                    afrl: AFRL(PhantomData),
                    moder: MODER(PhantomData),
                    otyper: OTYPER(PhantomData),
                    pupdr: PUPDR(PhantomData),
                    $(
                        $PXiL: $PXiL(PhantomData),
                    )*
                    $(
                        $PXiH: $PXiH(PhantomData),
                    )*
                }
            }
        }

    }
}

macro_rules! impl_pin {
    ($GPIOX:ident, $PXi:ident, $AFR:ident, $i:expr) => {
        /// Specific Pin
        pub struct $PXi<MODE>(PhantomData<MODE>);

        impl<MODE> $PXi<MODE> {
            const OFFSET: u32 = 2 * $i;

            /// Configures the PIN to operate as Input Pin according to Mode.
            pub fn into_input<Mode: InputMode>(self, moder: &mut MODER<$GPIOX>, pupdr: &mut PUPDR<$GPIOX>) -> $PXi<Input<Mode>> {
                moder.moder().modify(|r, w| unsafe { w.bits(r.bits() & !(0b11 << Self::OFFSET)) });
                pupdr.pupdr().modify(|r, w| unsafe { w.bits(Mode::modify_pupdr_bits(r.bits(), Self::OFFSET)) });

                $PXi(PhantomData)
            }

            /// Configures the PIN to operate as Output Pin according to Mode.
            pub fn into_output<Mode: OutputMode>(self, moder: &mut MODER<$GPIOX>, otyper: &mut OTYPER<$GPIOX>) -> $PXi<Output<Mode>> {
                moder
                    .moder()
                    .modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << Self::OFFSET)) | (0b01 << Self::OFFSET)) });
                otyper.otyper().modify(|r, w| unsafe { w.bits(Mode::modify_otyper_bits(r.bits(), $i)) });

                $PXi(PhantomData)
            }

            /// Configures the PIN to operate as Alternate Function.
            pub fn into_alt_fun<AF: AltFun>(self, moder: &mut MODER<$GPIOX>, afr: &mut $AFR<$GPIOX>) -> $PXi<AF> {
                moder
                    .moder()
                    .modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << Self::OFFSET)) | (0b10 << Self::OFFSET)) });
                afr.afr()
                    .modify(|r, w| unsafe { w.bits((r.bits() & !(0b1111 << Self::OFFSET)) | (AF::NUM << Self::OFFSET)) });

                $PXi(PhantomData)
            }
        }

        impl<MODE> OutputPin for $PXi<Output<MODE>> {
            /// Sets high bit.
            fn set_high(&mut self) {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << $i)) }
            }

            /// Sets low bit.
            fn set_low(&mut self) {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << (16 + $i))) }
            }
        }

        impl<MODE> StatefulOutputPin for $PXi<Output<MODE>> {
            /// Returns whether high bit is set.
            fn is_set_high(&self) -> bool {
                !self.is_set_low()
            }

            /// Returns whether low bit is set.
            fn is_set_low(&self) -> bool {
                // NOTE(unsafe) atomic read with no side effects
                unsafe { (*$GPIOX::ptr()).odr.read().bits() & (1 << $i) == 0 }
            }
        }
    };
}

macro_rules! impl_pins {
    ($GPIOX:ident, $ARF:ident: [$($PXi:ident, $i:expr;)*]) => {
        $(
            impl_pin!($GPIOX, $PXi, $ARF, $i);
         )*
    }
}

/// Generic LED
pub struct Led<PIN>(PIN);
impl<PIN: OutputPin + StatefulOutputPin> Led<PIN> {
    #[inline]
    /// Turns LED off.
    pub fn off(&mut self) {
        self.0.set_low();
    }
    #[inline]
    /// Checks whether LED is off
    pub fn is_off(&mut self) -> bool {
        self.0.is_set_low()
    }
    #[inline]
    /// Turns LED on.
    pub fn on(&mut self) {
        self.0.set_high()
    }
    #[inline]
    /// Checks whether LED is on
    pub fn is_on(&mut self) -> bool {
        self.0.is_set_high()
    }
}

impl<PIN: OutputPin> OutputPin for Led<PIN> {
    #[inline]
    fn set_high(&mut self) {
        self.0.set_high();
    }
    #[inline]
    fn set_low(&mut self) {
        self.0.set_low();
    }
}

impl<PIN: StatefulOutputPin> StatefulOutputPin for Led<PIN> {
    #[inline]
    fn is_set_high(&self) -> bool {
        self.0.is_set_high()
    }
    #[inline]
    fn is_set_low(&self) -> bool {
        self.0.is_set_low()
    }
}

impl<PIN: OutputPin + StatefulOutputPin> toggleable::Default for Led<PIN> {}

impl<PIN> Deref for Led<PIN> {
    type Target = PIN;

    #[inline]
    fn deref(&self) -> &PIN {
        &self.0
    }
}

#[allow(unused_macros)]
macro_rules! define_led {
    ($(#[$attr:meta])* $name:ident, $typ:ty) => {
        $(#[$attr])*
        pub type $name = Led<$typ>;
        impl Led<$typ> {
            #[inline]
            ///Creates a new instance of LED.
            ///
            ///Defined only for these PINs that can be used as LED.
            pub fn new(pin: $typ) -> Self {
                Led(pin)
            }
        }
    }
}

/// Opaque AFRL register
pub struct AFRL<GPIO>(PhantomData<GPIO>);
/// Opaque AFRH register
pub struct AFRH<GPIO>(PhantomData<GPIO>);
/// Opaque MODER register
pub struct MODER<GPIO>(PhantomData<GPIO>);
/// Opaque OTYPER register
pub struct OTYPER<GPIO>(PhantomData<GPIO>);
/// Opaque PUPDR register
pub struct PUPDR<GPIO>(PhantomData<GPIO>);

impl_parts!(
    GPIOA, gpioa;
    GPIOB, gpiob;
    GPIOC, gpioc;
    );

// Each I/O pin (except PH3 for STM32L496xx/4A6xx devices) has a multiplexer with up to
// sixteen alternate function inputs (AF0 to AF15) that can be configured through the
// GPIOx_AFRL (for pin 0 to 7) and GPIOx_AFRH (for pin 8 to 15) registers
//
// The GPIO ports (and pins) enumerated here are exposed on all package variants of the STM32L4x6.
// Larger chips have more pins, and so have additional definitions in their respective modules.
impl_gpio!(A, GPIOA, gpioaen, gpioarst,
           AFRL: [PA0, 0; PA1, 1; PA2, 2; PA3, 3; PA4, 4; PA5, 5; PA6, 6; PA7, 7;],
           AFRH: [PA8, 8; PA9, 9; PA10, 10; PA11, 11; PA12, 12; PA13, 13; PA14, 14; PA15, 15; ]
          );
impl_gpio!(B, GPIOB, gpioben, gpiobrst,
           AFRL: [PB0, 0; PB1, 1; PB2, 2; PB3, 3; PB4, 4; PB5, 5; PB6, 6; PB7, 7;],
           AFRH: [PB8, 8; PB9, 9; PB10, 10; PB11, 11; PB12, 12; PB13, 13; PB14, 14; PB15, 15; ]
          );
impl_gpio!(C, GPIOC, gpiocen, gpiocrst,
           AFRL: [PC0, 0; PC1, 1; PC2, 2; PC3, 3; PC4, 4; PC5, 5; PC6, 6; PC7, 7;],
           AFRH: [PC8, 8; PC9, 9; PC10, 10; PC11, 11; PC12, 12; PC13, 13; PC14, 14; PC15, 15; ]
          );

#[cfg(feature = "STM32L476VG")]
pub mod stm32l476vg;

#[cfg(feature = "STM32L496AG")]
pub mod stm32l496ag;
