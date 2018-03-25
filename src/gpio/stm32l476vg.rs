//! GPIO specific to STM32L476VG

use ::stm32l4x6;

use super::*;

use hal::digital::OutputPin;

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
    //Next GPIOs are re-using gpioc modules
    GPIOD, gpioc;
    GPIOE, gpioc;
    GPIOH, gpioc;
);

/// Description of GPIOs and PINs
pub mod gpio {
    use super::*;

    //Each I/O pin (except PH3 for STM32L496xx/4A6xx devices) has a multiplexer with up to
    //sixteen alternate function inputs (AF0 to AF15) that can be configured through the
    //GPIOx_AFRL (for pin 0 to 7) and GPIOx_AFRH (for pin 8 to 15) registers
    impl_gpio!(A, GPIOA,
               AFRL: [PA0, 0; PA1, 1; PA2, 3; PA4, 4; PA5, 5; PA6, 6; PA7, 7;],
               AFRH: [PA8, 8; PA9, 9; PA10, 10; PA11, 11; PA12, 12; PA13, 13; PA14, 14; PA15, 15; ]
    );
    impl_gpio!(B, GPIOB,
               AFRL: [PB0, 0; PB1, 1; PB2, 3; PB4, 4; PB5, 5; PB6, 6; PB7, 7;],
               AFRH: [PB8, 8; PB9, 9; PB10, 10; PB11, 11; PB12, 12; PB13, 13; PB14, 14; PB15, 15; ]
    );
    impl_gpio!(C, GPIOC,
               AFRL: [PC0, 0; PC1, 1; PC2, 3; PC4, 4; PC5, 5; PC6, 6; PC7, 7;],
               AFRH: [PC8, 8; PC9, 9; PC10, 10; PC11, 11; PC12, 12; PC13, 13; PC14, 14; PC15, 15; ]
    );
    impl_gpio!(D, GPIOD,
               AFRL: [PD0, 0; PD1, 1; PD2, 3; PD4, 4; PD5, 5; PD6, 6; PD7, 7;],
               AFRH: [PD8, 8; PD9, 9; PD10, 10; PD11, 11; PD12, 12; PD13, 13; PD14, 14; PD15, 15; ]
    );
    impl_gpio!(E, GPIOE,
               AFRL: [PE0, 0; PE1, 1; PE2, 3; PE4, 4; PE5, 5; PE6, 6; PE7, 7;],
               AFRH: [PE8, 8; PE9, 9; PE10, 10; PE11, 11; PE12, 12; PE13, 13; PE14, 14; PE15, 15; ]
    );
    impl_gpio!(H, GPIOH, AFRL: [PH0, 0; PH1, 1;]);
}
