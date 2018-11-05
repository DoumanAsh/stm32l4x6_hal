use super::{SCK, MISO, MOSI};

use ::gpio::{AF5};
use ::gpio::stm32l476vg::gpio::{PE13, PE14, PE15, PG9, PG10, PG11};

impl_pins_trait!(1 => {
    TRAIT: SCK,
    AF: AF5,
    PINS: [PE13,]
});
impl_pins_trait!(1 => {
    TRAIT: MISO,
    AF: AF5,
    PINS: [PE14,]
});
impl_pins_trait!(1 => {
    TRAIT: MOSI,
    AF: AF5,
    PINS: [PE15,]
});

impl_pins_trait!(1 => {
    TRAIT: SCK,
    AF: AF5,
    PINS: [PG9,]
});
impl_pins_trait!(1 => {
    TRAIT: MISO,
    AF: AF5,
    PINS: [PG10,]
});
impl_pins_trait!(1 => {
    TRAIT: MOSI,
    AF: AF5,
    PINS: [PG11,]
});


