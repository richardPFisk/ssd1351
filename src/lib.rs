#![no_std]
#![allow(clippy::result_unit_err)]

extern crate embedded_hal as hal;

pub mod builder;
pub mod command;
pub mod display;
pub mod mode;
pub mod prelude;
pub mod properties;

#[cfg(feature = "async")]
pub mod async_interface;
#[cfg(feature = "async")]
pub mod async_display;
#[cfg(feature = "async")]
pub mod async_command;
#[cfg(feature = "async")]
pub mod async_builder;
