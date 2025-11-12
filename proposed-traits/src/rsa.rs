use core::num::NonZeroU32;

use crate::common::{FromBytes, ToBytes};

pub enum PaddingMode {
    Pkcs1v15,
    Pss,
}

pub enum RsaSize {
    Size2048,
    Size3072,
    Size4096,
    Other(NonZeroU32),
}

pub enum ErrorKind {
    InvalidLength,
    SignError,
    VerifyError,
}

pub trait Error: core::fmt::Debug {
    /// Convert error to a generic error kind
    ///
    /// By using this method, errors freely defined by HAL implementations
    /// can be converted to a set of generic errors upon which generic
    /// code can act.
    fn kind(&self) -> ErrorKind;
}

/// RSA error type trait.
///
// This just defines the error type, to be used by the other  traits.
pub trait ErrorType {
    /// Error type
    type Error: Error;
}

pub trait RsaKeys {
    type PrivateKey;
    type PublicKey;
}

pub trait RsaSignature {
    type Signature: ToBytes + FromBytes;
}

pub trait RsaMessage {
    /// Type representing the message to be signed.
    ///
    /// This associated type allows implementers to define the format and semantics
    /// of the message that will be signed. It provides flexibility to support various
    /// data representations, such as raw byte buffers, structured types, or hardware-specific
    /// memory regions.
    ///
    /// Importantly, it is up to the implementer to decide whether the message represents:
    /// - A raw message payload to be hashed and signed internally.
    /// - A precomputed hash digest that should be signed directly.
    /// - Some other form of data, depending on the cryptographic policy or hardware constraints.
    ///
    /// This design enables the trait to be adaptable across different platforms and
    /// use cases without enforcing a fixed message format or signing strategy.
    type Message: ToBytes + FromBytes;
}

pub trait RsaKeyGen: ErrorType + RsaKeys {
    fn generate_keys(bits: RsaSize) -> Result<(Self::PrivateKey, Self::PublicKey), Self::Error>;
}

/// Trait for RSA signing operations.
pub trait RsaSign: ErrorType + RsaKeys + RsaSignature + RsaMessage {
    /// Signs a message using the given private key and padding mode.
    ///
    /// # Arguments
    ///
    /// * `private_key` - The private key to use for signing.
    /// * `message` - The message to sign.
    /// * `padding_mode` - The padding scheme to use.
    ///
    /// # Returns
    ///
    /// The generated signature, or an error.
    fn sign(
        &mut self,
        private_key: &Self::PrivateKey,
        message: Self::Message,
        padding_mode: PaddingMode,
    ) -> Result<Self::Signature, Self::Error>;
}

/// Trait for RSA signature verification.
pub trait RsaVerify: ErrorType + RsaKeys + RsaSignature + RsaMessage {
    /// Verifies a signature against a message and public key.
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key to use for verification.
    /// * `message` - The message that was signed.
    /// * `padding_mode` - The padding scheme used during signing.
    /// * `signature` - The signature to verify.
    ///
    /// # Returns
    ///
    /// The signature if verification is successful, or an error.
    fn verify(
        &mut self,
        public_key: &Self::PublicKey,
        message: Self::Message,
        padding_mode: PaddingMode,
        signature: &Self::Signature,
    ) -> Result<Self::Signature, Self::Error>;
}
