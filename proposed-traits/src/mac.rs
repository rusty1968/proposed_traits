use core::fmt::Debug;

/// Common error kinds for MAC operations (reused from digest operations).
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The input data length is not valid for the MAC function.
    InvalidInputLength,
    /// The specified MAC algorithm is not supported by the hardware or software implementation.
    UnsupportedAlgorithm,
    /// Failed to allocate memory for the MAC computation.
    MemoryAllocationFailure,
    /// Failed to initialize the MAC computation context.
    InitializationError,
    /// Error occurred while updating the MAC computation with new data.
    UpdateError,
    /// Error occurred while finalizing the MAC computation.
    FinalizationError,
    /// The hardware accelerator is busy and cannot process the MAC computation.
    Busy,
    /// General hardware failure during MAC computation.
    HardwareFailure,
    /// The specified output size is not valid for the MAC function.
    InvalidOutputSize,
    /// Insufficient permissions to access the hardware or perform the MAC computation.
    PermissionDenied,
    /// The MAC computation context has not been initialized.
    NotInitialized,
}

/// Trait for converting implementation-specific errors into a common error kind.
pub trait Error: Debug {
    /// Returns a generic error kind corresponding to the specific error.
    fn kind(&self) -> ErrorKind;
}

impl Error for core::convert::Infallible {
    fn kind(&self) -> ErrorKind {
        match *self {}
    }
}

/// Trait for types that associate with a specific error type.
pub trait ErrorType {
    /// The associated error type.
    type Error: Error;
}

/// Trait representing a MAC algorithm and its output characteristics.
pub trait MacAlgorithm {
    /// The number of bits in the MAC output.
    const OUTPUT_BITS: usize;

    /// The type representing the MAC output.
    type MacOutput;

    /// The type representing the key used for MAC computation.
    type Key;
}

/// Trait for initializing a MAC operation for a specific algorithm.
pub trait MacInit<A: MacAlgorithm>: ErrorType {
    /// The type representing the operational context for the MAC.
    type OpContext<'a>: MacOp<Output = A::MacOutput>
    where
        Self: 'a;

    /// Initializes the MAC operation with the specified algorithm and key.
    ///
    /// # Parameters
    ///
    /// - `algo`: A zero-sized type representing the MAC algorithm to use.
    /// - `key`: A reference to the key used for the MAC computation.
    ///
    /// # Returns
    ///
    /// A result containing the operational context for the MAC, or an error.
    fn init<'a>(&'a mut self, algo: A, key: &A::Key) -> Result<Self::OpContext<'a>, Self::Error>;
}

/// Optional trait for resetting a MAC context to its initial state.
pub trait MacCtrlReset: ErrorType {
    /// Resets the MAC context.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn reset(&mut self) -> Result<(), Self::Error>;
}

/// Trait for performing MAC operations.
pub trait MacOp: ErrorType {
    /// The type of the MAC output.
    type Output;

    /// Updates the MAC state with the provided input data.
    ///
    /// # Parameters
    ///
    /// - `input`: A byte slice containing the data to authenticate.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn update(&mut self, input: &[u8]) -> Result<(), Self::Error>;

    /// Finalizes the MAC computation and returns the result.
    ///
    /// # Returns
    ///
    /// A result containing the MAC output, or an error.
    fn finalize(self) -> Result<Self::Output, Self::Error>;
}
