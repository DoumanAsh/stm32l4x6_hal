//!Time related types
macro_rules! impl_struct {
    ($($name:ident,)+) => {
        $(
            #[derive(Clone, Copy)]
            pub struct $name(pub u32);
            impl Into<$name> for u32 {
                fn into(self) -> $name {
                    $name(self)
                }
            }
        )+
    }
}

impl_struct!(Bps, Hertz, KiloHertz, MegaHertz,);

impl Into<Hertz> for KiloHertz {
    fn into(self) -> Hertz {
        Hertz(self.0 * 1_000)
    }
}

impl Into<Hertz> for MegaHertz {
    fn into(self) -> Hertz {
        Hertz(self.0 * 1_000_000)
    }
}

impl Into<KiloHertz> for MegaHertz {
    fn into(self) -> KiloHertz {
        KiloHertz(self.0 * 1_000)
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy)]
pub struct Clocks {
    ///Frequency of AHB bus.
    pub ahb: Hertz,
    ///Frequency of APB1 bus.
    pub apb1: Hertz,
    ///APB1's prescaler
    pub ppre1: u8,
    ///Frequency of APB2 bus.
    pub apb2: Hertz,
    ///APB2's prescaler
    pub ppre2: u8,
    ///Frequency of System clocks.
    pub sys: Hertz
}
