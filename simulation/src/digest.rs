use proposed_traits::digest::*;
use std::{convert::Infallible, marker::PhantomData};

/// Dummy digest algorithm with 256-bit output.
pub struct DummySha256;

impl DigestAlgorithm for DummySha256 {
    const OUTPUT_BITS: usize = 256;

    type DigestOutput = DigestOutput256;
}

/// Example digest output type for 256-bit digests.
#[derive(Debug)]
pub struct DigestOutput256 {
    pub data: [u8; 32],
}


impl AsRef<[u8]> for DigestOutput256 {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

/// Dummy hardware controller.
struct Inner;

/// Digest controller that owns a reference to the hardware.
pub struct Controller<'r> {
    _inner: &'r mut Inner,
}

impl ErrorType for Controller<'_> {
    type Error = Infallible;
}

impl DigestInit<DummySha256> for Controller<'_> {
    type OpContext<'a> = OpContextImpl<'a>
    where
        Self: 'a;

    fn init<'a>(&'a mut self, _algo: DummySha256) -> Result<Self::OpContext<'a>, Self::Error> {
        Ok(OpContextImpl {
            _marker: PhantomData,
        })
    }
}

/// Digest operation context.
pub struct OpContextImpl<'a> {
    _marker: PhantomData<&'a ()>,
}

impl<'a> ErrorType for OpContextImpl<'a> {
    type Error = Infallible;
}

impl<'a> DigestOp for OpContextImpl<'a> {
    type Output = DigestOutput256;

    fn update(&mut self, _input: &[u8]) -> Result<(), Self::Error> {
        // Simulate update logic
        Ok(())
    }

    fn finalize(self) -> Result<Self::Output, Self::Error> {
        // Simulate final digest output
        Ok(DigestOutput256 {
            data: [0u8; 32],
        })
    }
}
impl DigestCtrlReset for Controller<'_> {
    fn reset(&mut self) -> Result<(), Self::Error> {
        // Simulate reset logic
        Ok(())
    }
}