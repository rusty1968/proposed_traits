use crate::{
    common::{FromBytes, ToBytes},
    digest::DigestAlgorithm,
};
use core::fmt::Debug;

pub trait Error: core::fmt::Debug {
    /// Convert error to a generic error kind
    ///
    /// By using this method, errors freely defined by HAL implementations
    /// can be converted to a set of generic errors upon which generic
    /// code can act.
    fn kind(&self) -> ErrorKind;
}

impl Error for core::convert::Infallible {
    fn kind(&self) -> ErrorKind {
        match *self {}
    }
}

pub trait ErrorType {
    /// Error type.
    type Error: Error;
}

/// Error kind.
///
/// This represents a common set of digest operation errors. Implementations are
/// free to define more specific or additional error types. However, by providing
/// a mapping to these common errors, generic code can still react to them.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    Busy,
    InvalidSignature,
    KeyGenError,
    SigningError,
    Other,
}

pub trait EccPrivateKey<'a>: ToBytes + FromBytes {
    /// Optional method to securely zero the key material.
    fn zeroize(&mut self);
}

/// A trait representing an abstract elliptic curve with associated types for cryptographic operations.
///
/// This trait defines the core components required for elliptic curve cryptography (ECC),
/// including the digest algorithm used for hashing, the scalar field element type, and the
/// point representation on the curve.
///
/// # Associated Types
///
/// - `DigestType`: A type implementing the `DigestAlgorithm` trait, used for cryptographic hashing.
/// - `Scalar`: The scalar field element type, typically used in scalar multiplication.
/// - `Point`: The type representing a point on the elliptic curve.
///
/// # Example
///
/// ```ignore
/// struct MyCurve;
///
/// impl Curve for MyCurve {
///     type DigestType = Sha256;
///     type Scalar = MyScalar;
/// }
/// ```
///
/// This trait is intended to be implemented by specific elliptic curve types to provide
/// a unified interface for cryptographic operations.
pub trait Curve {
    type DigestType: DigestAlgorithm;
    type Scalar: ToBytes + FromBytes;
}

pub trait PrivateKeyForCurve<C: Curve> {
    fn zeroize(&mut self);
}

pub trait SignatureForCurve<C: Curve>: ToBytes + FromBytes {
    fn r(&self) -> &C::Scalar;
    fn s(&self) -> &C::Scalar;
    fn new(r: C::Scalar, s: C::Scalar) -> Self;
}

/// A trait representing a public key associated with a specific elliptic curve.
pub trait PubKeyForCurve<C: Curve>: ToBytes + FromBytes {
    fn x(&self) -> &C::Scalar;
    fn y(&self) -> &C::Scalar;

    fn new(x: C::Scalar, y: C::Scalar) -> Self;
}

/// Trait for ECDSA key generation over a specific elliptic curve.
pub trait EcdsaKeyGen<C: Curve>: ErrorType {
    /// The type representing the private key for the curve.
    type PrivateKey: PrivateKeyForCurve<C>;

    /// The type representing the public key for the curve.
    type PublicKey: PubKeyForCurve<C>;

    /// Generates an ECDSA key pair.
    ///
    /// # Parameters
    /// - `rng`: A cryptographically secure random number generator.
    ///
    /// # Returns
    /// A tuple containing the generated private key and public key.
    fn generate_key_pair<R>(
        &mut self,
        rng: &mut R,
    ) -> Result<(Self::PrivateKey, Self::PublicKey), Self::Error>
    where
        R: rand_core::RngCore + rand_core::CryptoRng;
}

/// Trait for ECDSA signing using a digest algorithm.
pub trait EcdsaSign<C: Curve>: ErrorType {
    /// The type representing the private key for the curve.
    type PrivateKey: PrivateKeyForCurve<C>;

    /// The type representing the signature for the curve.
    type Signature: SignatureForCurve<C>;

    /// Signs a digest produced by a compatible hash function.
    ///
    /// # Parameters
    /// - `private_key`: The private key used for signing.
    /// - `digest`: The digest output from a hash function.
    /// - `rng`: A cryptographically secure random number generator.
    fn sign<R>(
        &mut self,
        private_key: &Self::PrivateKey,
        digest: <<C as Curve>::DigestType as DigestAlgorithm>::DigestOutput,
        rng: &mut R,
    ) -> Result<Self::Signature, Self::Error>
    where
        R: rand_core::RngCore + rand_core::CryptoRng;
}

/// Trait for ECDSA signature verification using a digest algorithm.
pub trait EcdsaVerify<C: Curve>: ErrorType {
    /// The type representing the public key for the curve.
    type PublicKey: PubKeyForCurve<C>;

    /// The type representing the signature for the curve.
    type Signature: SignatureForCurve<C>;

    /// Verifies a signature against a digest.
    ///
    /// # Parameters
    /// - `public_key`: The public key used for verification.
    /// - `digest`: The digest output from a hash function.
    /// - `signature`: The signature to verify.
    fn verify(
        &mut self,
        public_key: &Self::PublicKey,
        digest: <<C as Curve>::DigestType as DigestAlgorithm>::DigestOutput,
        signature: &Self::Signature,
    ) -> Result<(), Self::Error>;
}
