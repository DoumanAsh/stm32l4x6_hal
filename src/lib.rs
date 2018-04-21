//! HAL for the STM32L4x6 family of microcontrollers
//!
//! This is an implementation of the [`embedded-hal`] traits for the STM32L4x6 family of
//! microcontrollers.
//!
//! [`embedded-hal`]: https://github.com/japaric/embedded-hal
//!
//! # Usage
//!
//! To build applications (binary crates) using this crate follow the [cortex-m-quickstart]
//! instructions and add this crate as a dependency in step number 5 and make sure you enable the
//! "rt" Cargo feature of this crate.
//!
//! [cortex-m-quickstart]: https://docs.rs/cortex-m-quickstart/~0.2.3

#![no_std]

extern crate cast;
extern crate cortex_m;
extern crate embedded_hal as hal;
extern crate nb;
pub extern crate stm32l4x6;

use core::cmp;
use core::ops;
use core::marker;
use core::mem;

pub mod common;
pub mod power;
pub mod flash;
pub mod time;
pub mod config;
pub mod rcc;
pub mod delay;
pub mod gpio;
pub mod timer;
pub mod lcd;
