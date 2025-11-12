use proposed_traits::digest::*;
use zerocopy::{Immutable, IntoBytes};

#[derive(Copy, Clone, PartialEq, Eq, IntoBytes, Immutable)]
#[repr(C)]
pub struct Digest<const N: usize> {
    pub value: [u32; N],
}


// Implement AsRef<[u8]> for Digest
impl<const N: usize> AsRef<[u8]> for Digest<N> {
    fn as_ref(&self) -> &[u8] {
        // Safety: This is safe because u32 and u8 have the same alignment
        // and we are not changing the underlying data.
        let byte_ptr = self.value.as_ptr() as *const u8;
        let byte_len = N * core::mem::size_of::<u32>();
        unsafe { std::slice::from_raw_parts(byte_ptr, byte_len) }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sha2_256;
impl DigestAlgorithm for Sha2_256 {
    const OUTPUT_BITS: usize = 256usize;
    type DigestOutput = Digest<{Self::OUTPUT_BITS / 32}>;
}
#[derive(Clone, Copy, Debug)]
pub struct Sha2_384;
impl DigestAlgorithm for Sha2_384 {
    const OUTPUT_BITS: usize = 384usize;
    type DigestOutput = Digest<{Self::OUTPUT_BITS / 32}>;
}
#[derive(Clone, Copy, Debug)]
pub struct Sha2_512;
impl DigestAlgorithm for Sha2_512 {
    const OUTPUT_BITS: usize = 512;
    type DigestOutput = Digest<{Self::OUTPUT_BITS / 32}>;
}

#[derive(Clone, Copy, Debug)]
pub struct Sha3_224;
impl DigestAlgorithm for Sha3_224 {
    const OUTPUT_BITS: usize = 224usize;
    type DigestOutput = Digest<{Self::OUTPUT_BITS / 32}>;
}
#[derive(Clone, Copy, Debug)]
pub struct Sha3_256;
impl DigestAlgorithm for Sha3_256 {
    const OUTPUT_BITS: usize = 256usize;
    type DigestOutput = Digest<{Self::OUTPUT_BITS / 32}>;
}
#[derive(Clone, Copy, Debug)]
pub struct Sha3_384;
impl DigestAlgorithm for Sha3_384 {
    const OUTPUT_BITS: usize = 384usize;
    type DigestOutput = Digest<{Self::OUTPUT_BITS / 32}>;
}
#[derive(Clone, Copy, Debug)]
pub struct Sha3_512;
impl DigestAlgorithm for Sha3_512 {
    const OUTPUT_BITS: usize = 512;
    type DigestOutput = Digest<{Self::OUTPUT_BITS / 32}>;
}


#[derive(Copy, Clone)]
pub enum HashAlgo {
    SHA1,
    SHA224,
    SHA256,
    SHA384,
    SHA512,
    SHA512_224,
    SHA512_256,
}


pub struct Controller;

impl Controller {

     pub fn configure(&mut self, _: HashAlgo) {
         // Configure the controller for the specified hash algorithm
     }

     pub fn read_digest<const N: usize>(&self) -> [u32; N] {
         [0; N]
     }
    
    
    pub fn start_hash_operation(&mut self) {

    }

    pub fn wait_for_done(&mut self) {
        // Wait for the hash operation to complete
    }
}

#[derive(Debug)]
pub struct MyDigestError(ErrorKind);

impl Error for MyDigestError {
    fn kind(&self) -> ErrorKind {
        self.0
    }
}

impl ErrorType for Controller {
    type Error = MyDigestError;
}

impl DigestInit<Sha2_256> for Controller {
    type OpContext<'a> = ControllerOpContext<'a, Sha2_256>
    where
        Self: 'a;

    fn init<'a>(&'a mut self, algo: Sha2_256) -> Result<Self::OpContext<'a>, Self::Error> {
        self.configure(HashAlgo::SHA256);
        Ok(ControllerOpContext {
            controller: self,
            algo,
        })
    }
}
impl DigestCtrlReset for Controller {
    fn reset(&mut self) -> Result<(), Self::Error> {
        // Reset the controller state
        Ok(())
    }
}

pub struct ControllerOpContext<'a, A: DigestAlgorithm> {
    controller: &'a mut Controller,
    algo: A,
}

impl<'a, A: DigestAlgorithm> ErrorType for ControllerOpContext<'_, A> {
    type Error = MyDigestError;
}


impl<'a, A: DigestAlgorithm> DigestOp for ControllerOpContext<'a, A> {
    type Output = A::DigestOutput;

    fn update(&mut self, _input: &[u8]) -> Result<(), Self::Error> {
        self.controller.start_hash_operation();
        // Process input data
        Ok(())
    }

    fn finalize(self) -> Result<Self::Output, Self::Error> {
        self.controller.wait_for_done();
        let value = self.controller.read_digest();
        Ok(Digest { value }.into())
    }
}
