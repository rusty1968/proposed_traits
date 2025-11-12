// serde.rs

pub trait SerdeError: core::fmt::Debug {
    /// Convert error to a generic error kind
    ///
    /// By using this method, errors freely defined by HAL implementations
    /// can be converted to a set of generic errors upon which generic
    /// code can act.
    fn kind(&self) -> ErrorKind;
}

pub trait ErrorType {
    /// Error type.
    type Error: SerdeError;
}

/// Error kind.
///
/// This represents a common set of digest operation errors. Implementations are
/// free to define more specific or additional error types. However, by providing
/// a mapping to these common errors, generic code can still react to them.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    NotSupported,
    SourceBufferTooSmall,
    DestinationBufferTooSmall,
    Other,
}

/// Endianness selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

/// Trait for endian-aware serialization and deserialization
pub trait ToBytes: ErrorType {
    fn to_bytes(&self, dest: &mut [u8], endian: Endian) -> Result<(), Self::Error>;
}

pub trait FromBytes: ErrorType {
    fn from_bytes(bytes: &[u8], endian: Endian) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
