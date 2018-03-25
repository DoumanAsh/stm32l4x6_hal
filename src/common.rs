//! Common primitives for this HAL

/// Extension trait to constrain the peripheral.
pub trait Constrain<T> {
    /// Constrains the peripheral to play nicely with the other abstractions
    fn constrain(self) -> T;
}
