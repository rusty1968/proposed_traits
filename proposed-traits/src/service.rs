use crate::common::{FromBytes, ToBytes};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The operation code is not recognized or supported.
    UnsupportedOperation,
    /// The request payload was invalid or malformed.
    InvalidRequest,
    /// The service encountered an internal failure.
    Internal,
    /// The service is temporarily unavailable.
    Unavailable,
    /// A catch-all for unexpected errors.
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

/// Trait for services that can handle interrupt notifications.
pub trait Interruptible {
    /// Returns the notification bitmask this service is interested in.
    fn notification_mask(&self) -> u32;

    /// Called when an interrupt fires with the matching notification bits.
    fn on_notification(&mut self, irq_bits: u32);
}

pub trait Service: ErrorType {
    type Request: ToBytes;
    type Response: FromBytes;

    fn handle(&mut self, op: u16, request: Self::Request) -> Result<Self::Response, Self::Error>;
}
