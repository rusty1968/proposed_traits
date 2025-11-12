use crate::common::{FromBytes, ToBytes};
use core::fmt::Debug;

/// Marker trait for all cipher modes.
pub trait CipherMode: core::fmt::Debug + Clone + Copy {}

/// Marker trait for block cipher modes (e.g., CBC, CTR).
pub trait BlockCipherMode: CipherMode {}

/// Marker trait for AEAD modes (e.g., GCM, CCM).
pub trait AeadCipherMode: CipherMode {}

/// Marker trait for stream cipher modes (e.g., ChaCha20).
pub trait StreamCipherMode: CipherMode {}

/// Common error kinds for symmetric cipher operations.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Failed to initialize the cipher context.
    InitializationError,

    /// General hardware failure during cipher operation.
    HardwareFailure,

    /// Insufficient permissions to access hardware or perform the operation.
    PermissionDenied,

    /// The cipher context is in an invalid or uninitialized state.
    InvalidState,

    /// The input data is invalid (e.g., wrong length or format).
    InvalidInput,

    /// The specified algorithm or mode is not supported.
    UnsupportedAlgorithm,

    /// Key or IV is invalid or missing.
    KeyError,
}

/// Trait for converting implementation-specific errors into a generic [`ErrorKind`].
pub trait Error: Debug {
    /// Returns a generic error kind corresponding to the specific error.
    fn kind(&self) -> ErrorKind;
}

/// Trait for associating a type with an error type.
pub trait ErrorType {
    /// The associated error type.
    type Error: Error;
}

/// Trait for symmetric cipher algorithms.
pub trait SymmetricCipher: ErrorType {
    /// The key type used for initialization.
    type Key: FromBytes + ToBytes;

    /// The nonce or IV type.
    type Nonce: FromBytes + ToBytes;

    /// The plaintext and ciphertext types.
    type PlainText: FromBytes + ToBytes;
    type CipherText: FromBytes + ToBytes;
}

/// Trait for initializing a cipher with a specific mode.
pub trait CipherInit<M: CipherMode>: SymmetricCipher {
    /// The operational context for performing encryption/decryption.
    type CipherContext<'a>: CipherOp<M>
    where
        Self: 'a;

    /// Initializes the cipher with the given parameters.
    ///
    /// # Parameters
    ///
    /// - `key`: A reference to the key used for the cipher.
    /// - `nonce`: A reference to the nonce or IV used for the cipher.
    /// - `mode`: The cipher mode to use.
    ///
    /// # Returns
    ///
    /// A result containing the operational context or an error.
    fn init<'a>(
        &'a mut self,
        key: &Self::Key,
        nonce: &Self::Nonce,
        mode: M,
    ) -> Result<Self::CipherContext<'a>, Self::Error>;
}

/// Trait for basic encryption/decryption operations.
pub trait CipherOp<M: CipherMode>: SymmetricCipher + ErrorType {
    /// Encrypts the given plaintext.
    ///
    /// # Parameters
    ///
    /// - `plaintext`: The data to encrypt.
    ///
    /// # Returns
    ///
    /// A result containing the ciphertext or an error.
    fn encrypt(&mut self, plaintext: Self::PlainText) -> Result<Self::CipherText, Self::Error>;

    /// Decrypts the given ciphertext.
    ///
    /// # Parameters
    ///
    /// - `ciphertext`: The data to decrypt.
    ///
    /// # Returns
    ///
    /// A result containing the plaintext or an error.
    fn decrypt(&mut self, ciphertext: Self::CipherText) -> Result<Self::PlainText, Self::Error>;
}

/// Optional trait for cipher contexts that support resetting to their initial state.
pub trait ResettableCipherOp: ErrorType {
    /// Resets the cipher context.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn reset(&mut self) -> Result<(), Self::Error>;
}

/// Optional trait for cipher contexts that support rekeying.
pub trait CipherRekey<K>: ErrorType {
    /// Rekeys the cipher context with a new key.
    ///
    /// # Parameters
    ///
    /// - `new_key`: A reference to the new key.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn rekey(&mut self, new_key: &K) -> Result<(), Self::Error>;
}

/// Trait for AEAD operations (e.g., AES-GCM).
pub trait AeadCipherOp: SymmetricCipher + ErrorType {
    /// The associated data type for AEAD.
    type AssociatedData: FromBytes + ToBytes;

    /// The tag type for AEAD.
    type Tag: FromBytes + ToBytes;

    /// Encrypts the given plaintext with associated data.
    ///
    /// # Parameters
    ///
    /// - `plaintext`: The data to encrypt.
    /// - `associated_data`: The associated data to authenticate.
    ///
    /// # Returns
    ///
    /// A result containing the ciphertext and authentication tag or an error.
    fn encrypt_aead(
        &mut self,
        plaintext: Self::PlainText,
        associated_data: Self::AssociatedData,
    ) -> Result<(Self::CipherText, Self::Tag), Self::Error>;

    /// Decrypts the given ciphertext with associated data and authentication tag.
    ///
    /// # Parameters
    ///
    /// - `ciphertext`: The data to decrypt.
    /// - `associated_data`: The associated data to authenticate.
    /// - `tag`: The authentication tag.
    ///
    /// # Returns
    ///
    /// A result containing the plaintext or an error.
    fn decrypt_aead(
        &mut self,
        ciphertext: Self::CipherText,
        associated_data: Self::AssociatedData,
        tag: Self::Tag,
    ) -> Result<Self::PlainText, Self::Error>;
}
