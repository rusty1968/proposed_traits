// Enum-based digest polymorphism example using DigestInit and DigestEngine
// Uses the actual digest traits and sha2 crate for real hashing
use sha2::{Sha256, Sha384, Sha512, Digest};
use proposed_traits::digest::{DigestInit, DigestOp, DigestAlgorithm, ErrorType};
use core::convert::Infallible;

// --- Marker types for algorithms ---
#[derive(Copy, Clone, Debug)]
pub struct Sha2_256;
#[derive(Copy, Clone, Debug)]
pub struct Sha2_384;
#[derive(Copy, Clone, Debug)]
pub struct Sha2_512;

impl DigestAlgorithm for Sha2_256 {
    const OUTPUT_BITS: usize = 256;
    type DigestOutput = [u8; 32];
}
impl DigestAlgorithm for Sha2_384 {
    const OUTPUT_BITS: usize = 384;
    type DigestOutput = [u8; 48];
}
impl DigestAlgorithm for Sha2_512 {
    const OUTPUT_BITS: usize = 512;
    type DigestOutput = [u8; 64];
}

// --- Concrete op/context types ---
pub struct Sha256Op(Sha256);
pub struct Sha384Op(Sha384);
pub struct Sha512Op(Sha512);

impl ErrorType for Sha256Op {
    type Error = Infallible;
}
impl ErrorType for Sha384Op {
    type Error = Infallible;
}
impl ErrorType for Sha512Op {
    type Error = Infallible;
}

impl DigestOp for Sha256Op {
    type Output = [u8; 32];
    fn update(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.0.update(data);
        Ok(())
    }
    fn finalize(self) -> Result<Self::Output, Self::Error> {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&self.0.finalize());
        Ok(arr)
    }
}
impl DigestOp for Sha384Op {
    type Output = [u8; 48];
    fn update(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.0.update(data);
        Ok(())
    }
    fn finalize(self) -> Result<Self::Output, Self::Error> {
        let mut arr = [0u8; 48];
        arr.copy_from_slice(&self.0.finalize());
        Ok(arr)
    }
}
impl DigestOp for Sha512Op {
    type Output = [u8; 64];
    fn update(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.0.update(data);
        Ok(())
    }
    fn finalize(self) -> Result<Self::Output, Self::Error> {
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&self.0.finalize());
        Ok(arr)
    }
}

// --- DigestEngine implementing DigestInit ---
pub struct DigestEngine;

impl ErrorType for DigestEngine {
    type Error = Infallible;
}

impl DigestInit<Sha2_256> for DigestEngine {
    type OpContext<'a> = Sha256Op where Self: 'a;
    fn init<'a>(&'a mut self, _algo: Sha2_256) -> Result<Self::OpContext<'a>, Self::Error> {
        Ok(Sha256Op(Sha256::new()))
    }
}
impl DigestInit<Sha2_384> for DigestEngine {
    type OpContext<'a> = Sha384Op where Self: 'a;
    fn init<'a>(&'a mut self, _algo: Sha2_384) -> Result<Self::OpContext<'a>, Self::Error> {
        Ok(Sha384Op(Sha384::new()))
    }
}
impl DigestInit<Sha2_512> for DigestEngine {
    type OpContext<'a> = Sha512Op where Self: 'a;
    fn init<'a>(&'a mut self, _algo: Sha2_512) -> Result<Self::OpContext<'a>, Self::Error> {
        Ok(Sha512Op(Sha512::new()))
    }
}

// --- Enum for dynamic selection ---
#[derive(Copy, Clone, Debug)]
pub enum DigestAlgoId { Sha256, Sha384, Sha512 }

pub enum DynamicDigestOp {
    Sha256(Sha256Op),
    Sha384(Sha384Op),
    Sha512(Sha512Op),
}

impl DynamicDigestOp {
    pub fn update(&mut self, data: &[u8]) {
        match self {
            DynamicDigestOp::Sha256(op) => { let _ = op.update(data); },
            DynamicDigestOp::Sha384(op) => { let _ = op.update(data); },
            DynamicDigestOp::Sha512(op) => { let _ = op.update(data); },
        }
    }
    pub fn finalize(self) -> Vec<u8> {
        match self {
            DynamicDigestOp::Sha256(op) => op.finalize().unwrap().to_vec(),
            DynamicDigestOp::Sha384(op) => op.finalize().unwrap().to_vec(),
            DynamicDigestOp::Sha512(op) => op.finalize().unwrap().to_vec(),
        }
    }
}

// --- Factory using DigestInit ---
fn new_dynamic_digest_op(engine: &mut DigestEngine, algo: DigestAlgoId) -> DynamicDigestOp {
    match algo {
        DigestAlgoId::Sha256 => DynamicDigestOp::Sha256(engine.init(Sha2_256).unwrap()),
        DigestAlgoId::Sha384 => DynamicDigestOp::Sha384(engine.init(Sha2_384).unwrap()),
        DigestAlgoId::Sha512 => DynamicDigestOp::Sha512(engine.init(Sha2_512).unwrap()),
    }
}

// --- Example usage ---
fn main() {
    let mut engine = DigestEngine;
    let algos = [DigestAlgoId::Sha256, DigestAlgoId::Sha384, DigestAlgoId::Sha512];
    let data = b"enum-based digest!";
    for algo in &algos {
        let mut op = new_dynamic_digest_op(&mut engine, *algo);
        op.update(data);
        let digest = op.finalize();
        println!("{:?}: {}", algo, hex::encode(digest));
    }
}

// --- Debug for DigestAlgoId ---
impl std::fmt::Display for DigestAlgoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DigestAlgoId::Sha256 => write!(f, "SHA-256"),
            DigestAlgoId::Sha384 => write!(f, "SHA-384"),
            DigestAlgoId::Sha512 => write!(f, "SHA-512"),
        }
    }
}
