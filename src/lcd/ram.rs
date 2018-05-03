use super::LCD;

pub trait Index {
    type RamType;

    fn ram(lcd: &LCD) -> &Self::RamType;
    fn write(lcd: &mut LCD, data: u32);
}

macro_rules! define_index {
    ($(#[$attr:meta])* $name:ident: $ram_type:ty, $access:ident) => {
        $(#[$attr])*
        pub struct $name;
        impl Index for $name {
            type RamType = $ram_type;

            fn ram(lcd: &LCD) -> &Self::RamType {
                &lcd.inner.$access
            }

            fn write(lcd: &mut LCD, data: u32) {
                Self::ram(lcd).write(|w| unsafe { w.bits(data) })
            }
        }
    }
}

/// RAM index accessors.
pub mod index {
    use super::Index;
    use super::LCD;
    use stm32l4x6;

    define_index!(
        /// Access RAM0
        Zero: stm32l4x6::lcd::RAM_COM0,
        ram_com0
    );
    define_index!(
        /// Access RAM1
        One: stm32l4x6::lcd::RAM_COM1,
        ram_com1
    );
    define_index!(
        /// Access RAM2
        Two: stm32l4x6::lcd::RAM_COM2,
        ram_com2
    );
    define_index!(
        /// Access RAM3
        Three: stm32l4x6::lcd::RAM_COM3,
        ram_com3
    );
    define_index!(
        /// Access RAM4
        Four: stm32l4x6::lcd::RAM_COM4,
        ram_com4
    );
    define_index!(
        /// Access RAM5
        Five: stm32l4x6::lcd::RAM_COM5,
        ram_com5
    );
    define_index!(
        /// Access RAM6
        Six: stm32l4x6::lcd::RAM_COM6,
        ram_com6
    );
    define_index!(
        /// Access RAM7
        Seven: stm32l4x6::lcd::RAM_COM7,
        ram_com7
    );
}
