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
    GPIOH, gpioc;
    );

/// Description of GPIOs and PINs
pub mod gpio {
    pub use super::super::*;
    use super::*;

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
