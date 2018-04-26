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

#![feature(never_type)]
#![no_std]

extern crate cast;
extern crate cortex_m;
extern crate embedded_hal as hal;
extern crate nb;
pub extern crate stm32l4x6;

use core::cmp;
use core::marker;
use core::mem;
use core::ops;

pub mod common;
pub mod config;
pub mod delay;
pub mod flash;
pub mod gpio;
pub mod lcd;
pub mod power;
pub mod rcc;
pub mod time;
pub mod timer;
pub mod serial;
