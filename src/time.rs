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
    sysclk: Hertz,
}

impl Clocks {
    /// Returns the system (core) frequency
    pub fn sys_clock(&self) -> Hertz {
        self.sysclk
    }
}
