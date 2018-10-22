//! GPIO specific to STM32L476VG
//!
//! To use these definitions, enable the "STM32L476VG" feature, and include like so:
//!
//! ```rust
//! use stm32l4x6_hal::gpio::stm32l476vg::gpio;
//! ```

use stm32l4x6;

use super::*;

impl_parts!(
    GPIOD, gpioc;
    GPIOE, gpioc;
    GPIOF, gpioc;
    GPIOG, gpioc;
    GPIOH, gpioc;
    );

/// Description of GPIOs and PINs
pub mod gpio {
    pub use super::super::*;
    use super::*;

    impl_gpio!(D, GPIOD, gpioden, gpiodrst,
               AFRL: [PD0, 0; PD1, 1; PD2, 2; PD3, 3; PD4, 4; PD5, 5; PD6, 6; PD7, 7;],
               AFRH: [PD8, 8; PD9, 9; PD10, 10; PD11, 11; PD12, 12; PD13, 13; PD14, 14; PD15, 15; ]
    );
    impl_gpio!(E, GPIOE,  gpioeen, gpioerst,
               AFRL: [PE0, 0; PE1, 1; PE2, 2; PE3, 3; PE4, 4; PE5, 5; PE6, 6; PE7, 7;],
               AFRH: [PE8, 8; PE9, 9; PE10, 10; PE11, 11; PE12, 12; PE13, 13; PE14, 14; PE15, 15; ]
    );
    impl_gpio!(F, GPIOF, gpiofen, gpiofrst,
               AFRL: [PF0, 0; PF1, 1; PF2, 2; PF3, 3; PF4, 4; PF5, 5; PF6, 6; PF7, 7;],
               AFRH: [PF8, 8; PF9, 9; PF10, 10; PF11, 11; PF12, 12; PF13, 13; PF14, 14; PF15, 15; ]
    );
    impl_gpio!(G, GPIOG,  gpiogen, gpiogrst,
               AFRL: [PG0, 0; PG1, 1; PG2, 2; PG3, 3; PG4, 4; PG5, 5; PG6, 6; PG7, 7;],
               AFRH: [PG8, 8; PG9, 9; PG10, 10; PG11, 11; PG12, 12; PG13, 13; PG14, 14; PG15, 15; ]
    );
    impl_gpio!(H, GPIOH, gpiohen, gpiohrst,
               AFRL: [PH0, 0; PH1, 1;],
               AFRH: []);
}

/// Description of LEDs
pub mod led {
    use super::{gpio, Led, Output, PushPull};

    define_led!(
        /// User LED with Red color.
        Led4,
        gpio::PB2<Output<PushPull>>
    );

    define_led!(
        /// User LED with Green color.
        Led5,
        gpio::PE8<Output<PushPull>>
    );

    /// Retrieve all LEDs
    pub fn leds(mut gpio_b: gpio::B, mut gpio_e: gpio::E) -> (Led4, Led5) {
        let led4 = gpio_b.PB2.into_output::<PushPull>(&mut gpio_b.moder, &mut gpio_b.otyper);
        let led5 = gpio_e.PE8.into_output::<PushPull>(&mut gpio_e.moder, &mut gpio_e.otyper);

        (Led4::new(led4), Led5::new(led5))
    }
}
