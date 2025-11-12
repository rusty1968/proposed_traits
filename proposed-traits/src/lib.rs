#![no_std]
#![deny(unsafe_code)]

pub mod common;
pub mod digest;
pub mod ecdsa;
pub mod mac;
pub mod rsa;

pub mod block_device;
pub mod i2c_target;
pub mod i3c_master;
pub mod i3c_target;
pub mod system_control;

pub mod client;
pub mod otp;
pub mod service;
pub mod symm_cipher;
