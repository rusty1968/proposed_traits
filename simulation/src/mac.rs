// === MacAlgorithm Trait ===

use std::marker::PhantomData;


use hmac::{Hmac, Mac};

use proposed_traits::mac::{MacOp, Error, MacInit, MacAlgorithm, ErrorType, ErrorKind};
use sha2::Sha256;

pub struct HmacSha256;

impl MacAlgorithm for HmacSha256 {
    const OUTPUT_BITS: usize = 256;
    type MacOutput = [u8; 32];
    type Key = [u8; 32];
}

// === Custom Error Type ===

#[derive(Debug)]
pub enum MacError {
    Init,
    Update,
    Finalize,
}

impl Error for MacError {
    fn kind(&self) -> ErrorKind {
        match self {
            MacError::Init => ErrorKind::InitializationError,
            MacError::Update => ErrorKind::UpdateError,
            MacError::Finalize => ErrorKind::FinalizationError,
        }
    }
}

// === MacInit Implementation ===

pub struct HmacEngine;

impl ErrorType for HmacEngine {
    type Error = MacError;
}

impl MacInit<HmacSha256> for HmacEngine {
    type OpContext<'a> = HmacContext<'a> where Self: 'a;

    fn init<'a>(
        &'a mut self,
        _algo: HmacSha256,
        key: &<HmacSha256 as MacAlgorithm>::Key,
    ) -> Result<Self::OpContext<'a>, Self::Error> {
        let mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| MacError::Init)?;
        Ok(HmacContext { mac, _marker: PhantomData })
    }
}

// === MacOp Implementation ===

pub struct HmacContext<'a> {
    _marker: PhantomData<&'a ()>,
    mac: Hmac<Sha256>,
}

impl<'a> ErrorType for HmacContext<'a> {
    type Error = MacError;
}

impl<'a> MacOp for HmacContext<'a> {
    type Output = [u8; 32];

    fn update(&mut self, input: &[u8]) -> Result<(), Self::Error> {
        Mac::update(&mut self.mac, input);
        Ok(())
    }

    fn finalize(self) -> Result<Self::Output, Self::Error> {
        let result = self.mac.finalize().into_bytes();
        Ok(result.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_sha256_mac_computation() {
        // Sample key and message
        let key = [0x0b; 32]; // 32-byte key
        let message = b"The quick brown fox jumps over the lazy dog";

        // Initialize the MAC engine
        let mut engine = HmacEngine;
        let mut context = engine
            .init(HmacSha256, &key)
            .expect("Failed to initialize HMAC context");

        // Update with message
        context.update(message).expect("Failed to update MAC");

        // Finalize and get the MAC output
        let mac = context.finalize().expect("Failed to finalize MAC");

        // Expected HMAC-SHA256 output for this key/message pair
        let expected = [
            0xf7, 0xbc, 0x83, 0xf4, 0x30, 0x53, 0x84, 0x24,
            0xb1, 0x4f, 0xb6, 0x9a, 0xc7, 0x7a, 0x4f, 0x3d,
            0xb1, 0x0e, 0xd2, 0x4f, 0x09, 0x33, 0x8d, 0x3b,
            0x73, 0x2c, 0xb2, 0x3c, 0x6e, 0xc4, 0x7b, 0xe2,
        ];

        assert_eq!(mac, expected, "MAC output does not match expected value");
    }
}
