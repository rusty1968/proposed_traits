#![cfg_attr(not(feature = "std"), no_std)]

pub mod smbus;
pub mod spi_device_driver;

pub use spi_device_driver::{Config, SpiDeviceDriver};
