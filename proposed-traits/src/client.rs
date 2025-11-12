use crate::common::{FromBytes, ToBytes};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The target service could not be reached.
    NoRoute,
    /// The request was malformed or invalid.
    InvalidRequest,
    /// The response could not be parsed.
    InvalidResponse,
    /// The operation timed out.
    Timeout,
    /// The target service returned an error.
    RemoteError,
    /// An unspecified or unexpected error occurred.
    Other,
}

pub trait Error: core::fmt::Debug {
    /// Convert error to a generic error kind
    ///
    /// By using this method, errors freely defined by Algo implementations
    /// can be converted to a set of generic errors upon which generic
    /// code can act.
    fn kind(&self) -> ErrorKind;
}

impl Error for core::convert::Infallible {
    /// Convert error to a generic Mac error kind.
    ///
    /// By using this method, Mac errors freely defined by Algo implementations
    /// can be converted to a set of generic I2C errors upon which generic
    /// code can act.    
    fn kind(&self) -> ErrorKind {
        match *self {}
    }
}

pub trait ErrorType {
    /// Error type.
    type Error: Error;
}

/// An abstraction over a message port that enables sending serialized requests
/// to a target service and receiving deserialized responses over a communication channel.
pub trait Client: ErrorType {
    /// Sends a request to a specific target service and waits for a response.
    ///
    /// # Arguments
    ///
    /// * `service_id` - The destination service identifier (e.g., port ID or handle).
    /// * `op` - An operation code or selector.
    /// * `request` - The request payload to serialize and send.
    ///
    /// # Returns
    ///
    /// A deserialized response of type `RS`, or an error.
    fn call<RQ, RS>(&self, service_id: u32, op: u16, request: &RQ) -> Result<RS, Self::Error>
    where
        RQ: ToBytes,
        RS: FromBytes;
}
