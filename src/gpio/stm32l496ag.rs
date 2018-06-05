//! GPIO specific to STM32L496AG
//!
//! To use these definitions, enable the "STM32L496AG" feature, and include like so:
//!
//! ```rust
//! use stm32l4x6_hal::gpio::stm32l496ag::gpio;
//! ```

use stm32l4x6;

use super::*;

impl_parts!(
    GPIOF, gpioc;
    GPIOG, gpioc;
    GPIOH, gpioc;
    );

/// Description of GPIOs and PINs
pub mod gpio {
    pub use super::super::*;
    use super::*;

    impl_gpio!(F, GPIOF, gpiofen, gpiofrst,
               AFRL: [PF0, 0; PF1, 1; PF2, 2; PF3, 3; PF4, 4; PF5, 5; PF6, 6; PF7, 7;],
               AFRH: [PF8, 8; PF9, 9; PF10, 10; PF11, 11; PF12, 12; PF13, 13; PF14, 14; PF15, 15; ]
    );
    impl_gpio!(G, GPIOG,  gpiogen, gpiogrst,
               AFRL: [PG0, 0; PG1, 1; PG2, 2; PG3, 3; PG4, 4; PG5, 5; PG6, 6; PG7, 7;],
               AFRH: [PG8, 8; PG9, 9; PG10, 10; PG11, 11; PG12, 12; PG13, 13; PG14, 14; PG15, 15; ]
    );
    impl_gpio!(H, GPIOH, gpiohen, gpiohrst,
               AFRL: [PH0, 0; PH1, 1; PH2, 2; PH4, 4; PH5, 5; PH6, 6; PH7, 7;],
               AFRH: [PH8, 8; PH9, 9; PH10, 10; PH11, 11; PH12, 12; PH13, 13; PH14, 14; PH15, 15; ]
    );
}
