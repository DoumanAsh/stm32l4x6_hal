//! General Purpose Input / Output
//!
//! Note that GPIO layout is specific to board
//! which is configured through features.


use ::ops::Deref;
use ::marker::PhantomData;

use hal::digital::OutputPin;

use ::stm32l4x6;

use rcc::AHB;

/// Floating input (type state)
pub struct Floating;
/// Pulled down input (type state)
pub struct PullDown;
/// Pulled up input (type state)
pub struct PullUp;
/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
pub struct PushPull;
/// Open drain output (type state)
pub struct OpenDrain;
/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Alternate function 0 (type state)
pub struct AF0;
/// Alternate function 1 (type state)
pub struct AF1;
/// Alternate function 2 (type state)
pub struct AF2;
/// Alternate function 3 (type state)
pub struct AF3;
/// Alternate function 4 (type state)
pub struct AF4;
/// Alternate function 5 (type state)
pub struct AF5;
/// Alternate function 6 (type state)
pub struct AF6;
/// Alternate function 7 (type state)
pub struct AF7;
/// Alternate function 8 (type state)
pub struct AF8;
/// Alternate function 9 (type state)
pub struct AF9;
/// Alternate function 10 (type state)
pub struct AF10;
/// Alternate function 11 (type state)
pub struct AF11;
/// Alternate function 12 (type state)
pub struct AF12;
/// Alternate function 13 (type state)
pub struct AF13;
/// Alternate function 14 (type state)
pub struct AF14;
/// Alternate function 15 (type state)
pub struct AF15;

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
        ///Specific Pin
        pub struct $PXi<MODE>(PhantomData<MODE>);

        impl<MODE> $PXi<MODE> {
            const OFFSET: u32 = 2 * $i;

            #[inline]
            fn set_input_mode(moder: &mut MODER<$GPIOX>) {
                moder.moder().modify(|r, w| unsafe { w.bits(r.bits() & !(0b11 << Self::OFFSET)) });
            }

            /// Configures the pin to operate as a floating input pin
            pub fn into_floating_input(self, moder: &mut MODER<$GPIOX>, pupdr: &mut PUPDR<$GPIOX>) -> $PXi<Input<Floating>> {
                Self::set_input_mode(moder);
                // no pull-up or pull-down
                pupdr.pupdr().modify(|r, w| unsafe { w.bits(r.bits() & !(0b11 << Self::OFFSET)) });

                $PXi(PhantomData)
            }

            /// Configures the pin to operate as a pulled up input pin
            pub fn into_pull_up_input(self, moder: &mut MODER<$GPIOX>, pupdr: &mut PUPDR<$GPIOX>) -> $PXi<Input<PullUp>> {
                Self::set_input_mode(moder);
                // pull-up
                pupdr.pupdr().modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << Self::OFFSET)) | (0b01 << Self::OFFSET)) });

                $PXi(PhantomData)
            }

            #[inline]
            fn set_output_mode(moder: &mut MODER<$GPIOX>) {
                moder.moder().modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << Self::OFFSET)) | (0b01 << Self::OFFSET)) });
            }

            /// Configures the pin to operate as an open drain output pin
            pub fn into_open_drain_output(self, moder: &mut MODER<$GPIOX>, otyper: &mut OTYPER<$GPIOX>) -> $PXi<Output<OpenDrain>> {
                Self::set_output_mode(moder);
                // open drain output
                otyper.otyper().modify(|r, w| unsafe { w.bits(r.bits() | (0b1 << $i)) });

                $PXi(PhantomData)
            }

            /// Configures the pin to operate as an push pull output pin
            pub fn into_push_pull_output(self, moder: &mut MODER<$GPIOX>, otyper: &mut OTYPER<$GPIOX>) -> $PXi<Output<PushPull>> {
                Self::set_output_mode(moder);
                // push pull output
                otyper.otyper().modify(|r, w| unsafe { w.bits(r.bits() & !(0b1 << $i)) });

                $PXi(PhantomData)
            }

            // alternate function mode
            fn set_alt_fun_mode(moder: &mut MODER<$GPIOX>, afr: &mut $AFR<$GPIOX>, af: u32) {
                moder.moder().modify(|r, w| unsafe { w.bits((r.bits() & !(0b11 << Self::OFFSET)) | (0b10 << Self::OFFSET)) });
                afr.afr().modify(|r, w| unsafe { w.bits((r.bits() & !(0b1111 << Self::OFFSET)) | (af << Self::OFFSET)) });
            }

            #[inline]
            /// Configures the ping to operate as alternative function 4
            pub fn into_alt_fun4(self, moder: &mut MODER<$GPIOX>, afr: &mut $AFR<$GPIOX>) -> $PXi<AF4> {
                Self::set_alt_fun_mode(moder, afr, 4);
                $PXi(PhantomData)
            }

            #[inline]
            /// Configures the ping to operate as alternative function 5
            pub fn into_alt_fun5(self, moder: &mut MODER<$GPIOX>, afr: &mut $AFR<$GPIOX>) -> $PXi<AF5> {
                Self::set_alt_fun_mode(moder, afr, 5);
                $PXi(PhantomData)
            }

            #[inline]
            /// Configures the ping to operate as alternative function 6
            pub fn into_alt_fun6(self, moder: &mut MODER<$GPIOX>, afr: &mut $AFR<$GPIOX>) -> $PXi<AF6> {
                Self::set_alt_fun_mode(moder, afr, 6);
                $PXi(PhantomData)
            }

            #[inline]
            /// Configures the ping to operate as alternative function 7
            pub fn into_alt_fun7(self, moder: &mut MODER<$GPIOX>, afr: &mut $AFR<$GPIOX>) -> $PXi<AF7> {
                Self::set_alt_fun_mode(moder, afr, 7);
                $PXi(PhantomData)
            }

            #[inline]
            /// Configures the ping to operate as alternative function 11
            pub fn into_alt_fun11(self, moder: &mut MODER<$GPIOX>, afr: &mut $AFR<$GPIOX>) -> $PXi<AF11> {
                Self::set_alt_fun_mode(moder, afr, 11);
                $PXi(PhantomData)
            }

        }

        impl<MODE> OutputPin for $PXi<Output<MODE>> {
            ///Returns whether high bit is set.
            fn is_high(&self) -> bool {
                !self.is_low()
            }

            ///Returns whether low bit is set.
            fn is_low(&self) -> bool {
                // NOTE(unsafe) atomic read with no side effects
                unsafe { (*$GPIOX::ptr()).odr.read().bits() & (1 << $i) == 0 }
            }

            ///Sets high bit.
            fn set_high(&mut self) {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << $i)) }
            }

            ///Sets low bit.
            fn set_low(&mut self) {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { (*$GPIOX::ptr()).bsrr.write(|w| w.bits(1 << (16 + $i))) }
            }
        }
    }
}

macro_rules! impl_pins {
    ($GPIOX:ident, $ARF:ident: [$($PXi:ident, $i:expr;)*]) => {
        $(
            impl_pin!($GPIOX, $PXi, $ARF, $i);
         )*
    }
}

///Generic LED
pub struct Led<PIN>(PIN);
impl<PIN: OutputPin> Led<PIN> {
    #[inline]
    /// Turns LED off.
    pub fn off(&mut self) {
        self.0.set_low();
    }
    #[inline]
    /// Checks whether LED is off
    pub fn is_off(&mut self) -> bool {
        self.0.is_low()
    }
    #[inline]
    /// Turns LED on.
    pub fn on(&mut self) {
        self.0.set_high()
    }
    #[inline]
    /// Checks whether LED is on
    pub fn is_on(&mut self) -> bool {
        self.0.is_high()
    }
}

impl<PIN> Deref for Led<PIN> {
    type Target = PIN;

    #[inline]
    fn deref(&self) -> &PIN {
        &self.0
    }
}

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

//Each I/O pin (except PH3 for STM32L496xx/4A6xx devices) has a multiplexer with up to
//sixteen alternate function inputs (AF0 to AF15) that can be configured through the
//GPIOx_AFRL (for pin 0 to 7) and GPIOx_AFRH (for pin 8 to 15) registers
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
#[cfg(feature = "STM32L476VG")]
pub use gpio::stm32l476vg::gpio::*;
