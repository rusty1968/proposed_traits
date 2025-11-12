use core::fmt::Debug;

/// Common error kinds for digest operations.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The input data length is not valid for the hash function.
    InvalidInputLength,
    /// The specified hash algorithm is not supported by the hardware or software implementation.
    UnsupportedAlgorithm,
    /// Failed to allocate memory for the hash computation.
    MemoryAllocationFailure,
    /// Failed to initialize the hash computation context.
    InitializationError,
    /// Error occurred while updating the hash computation with new data.
    UpdateError,
    /// Error occurred while finalizing the hash computation.
    FinalizationError,
    /// The hardware accelerator is busy and cannot process the hash computation.
    Busy,
    /// General hardware failure during hash computation.
    HardwareFailure,
    /// The specified output size is not valid for the hash function.
    InvalidOutputSize,
    /// Insufficient permissions to access the hardware or perform the hash computation.
    PermissionDenied,
    /// The hash computation context has not been initialized.
    NotInitialized,
}

/// Registry of supported digest algorithms for protocol negotiation and discovery.
///
/// This trait provides a generic interface for querying and creating digest operations
/// based on algorithm identifiers. It's designed to support protocol-driven scenarios
/// where digest algorithms need to be negotiated or selected at runtime.
pub trait DigestRegistry: ErrorType {
    /// The type of algorithm identifiers used by this registry.
    type AlgorithmId: Copy + Debug + PartialEq;

    /// The type of digest operations created by this registry.
    type DigestOp;

    /// Check if a specific algorithm is supported by this registry.
    ///
    /// # Parameters
    ///
    /// - `algorithm_id`: The identifier of the algorithm to check.
    ///
    /// # Returns
    ///
    /// `true` if the algorithm is supported, `false` otherwise.
    fn supports_algorithm(&self, algorithm_id: Self::AlgorithmId) -> bool;

    /// Get the output size in bytes for a supported algorithm.
    ///
    /// # Parameters
    ///
    /// - `algorithm_id`: The identifier of the algorithm.
    ///
    /// # Returns
    ///
    /// `Some(size)` if the algorithm is supported, `None` otherwise.
    fn get_output_size(&self, algorithm_id: Self::AlgorithmId) -> Option<usize>;

    /// Create a digest operation for the specified algorithm.
    ///
    /// # Parameters
    ///
    /// - `algorithm_id`: The identifier of the algorithm to use.
    ///
    /// # Returns
    ///
    /// A result containing the digest operation, or an error if the algorithm
    /// is not supported or the operation cannot be created.
    fn create_digest(&mut self, algorithm_id: Self::AlgorithmId) -> Result<Self::DigestOp, Self::Error>;

    /// Get a slice of all supported algorithm identifiers.
    ///
    /// # Returns
    ///
    /// A slice containing the identifiers of all algorithms supported by this registry.
    fn supported_algorithms(&self) -> &[Self::AlgorithmId];
}

/// Dynamic digest operation trait for runtime algorithm selection.
///
/// This trait provides a generic interface for digest operations that can be 
/// selected and created at runtime based on algorithm identifiers. It supports
/// protocol-driven scenarios where the specific algorithm is negotiated or
/// chosen dynamically.
///
/// ## Object Safety
///
/// This trait is **object safe**, meaning it can be used as a trait object with
/// dynamic dispatch. This enables several important use cases:
///
/// - **Runtime polymorphism**: Store different digest implementations in collections
/// - **Protocol negotiation**: Choose algorithms dynamically based on peer capabilities
/// - **Plugin architectures**: Load digest implementations at runtime
///
/// ### Example: Using as Trait Object
///
/// ```rust
/// use proposed_traits::digest::{DynamicDigestOp, Error, ErrorKind};
/// use core::fmt::Debug;
///
/// // Define a simple error type
/// #[derive(Debug)]
/// enum MyError {
///     BufferTooSmall,
/// }
///
/// impl Error for MyError {
///     fn kind(&self) -> ErrorKind {
///         match self {
///             MyError::BufferTooSmall => ErrorKind::InvalidOutputSize,
///         }
///     }
/// }
///
/// // Example implementation
/// struct MockDigest {
///     algorithm_id: u32,
///     finalized: bool,
/// }
///
/// impl DynamicDigestOp for MockDigest {
///     type Error = MyError;
///     type AlgorithmId = u32;
///
///     fn update(&mut self, _input: &[u8]) -> Result<(), Self::Error> {
///         Ok(())
///     }
///
///     fn finalize(&mut self) -> Result<(), Self::Error> {
///         self.finalized = true;
///         Ok(())
///     }
///
///     fn output_size(&self) -> usize {
///         32
///     }
///
///     fn algorithm_id(&self) -> Self::AlgorithmId {
///         self.algorithm_id
///     }
///
///     fn copy_output(&self, output: &mut [u8]) -> Result<usize, Self::Error> {
///         if output.len() < 32 {
///             return Err(MyError::BufferTooSmall);
///         }
///         // Fill with mock data
///         output[..32].fill(0xAB);
///         Ok(32)
///     }
/// }
///
/// // Can be used as trait object for dynamic dispatch
/// let digest_ops: Vec<Box<dyn DynamicDigestOp<Error = MyError, AlgorithmId = u32>>> = vec![
///     Box::new(MockDigest { algorithm_id: 1, finalized: false }),
///     Box::new(MockDigest { algorithm_id: 2, finalized: false }),
/// ];
///
/// // Runtime algorithm selection
/// fn select_digest(
///     algorithm_id: u32,
///     available: &[Box<dyn DynamicDigestOp<Error = MyError, AlgorithmId = u32>>]
/// ) -> Option<&dyn DynamicDigestOp<Error = MyError, AlgorithmId = u32>> {
///     available.iter()
///         .find(|op| op.algorithm_id() == algorithm_id)
///         .map(|boxed| boxed.as_ref())
/// }
/// ```
///
/// ## Protocol Integration
///
/// This trait is designed to work seamlessly with [`DigestRegistry`] for complete
/// protocol-driven digest algorithm negotiation:
///
/// ```rust
/// # use proposed_traits::digest::{DigestRegistry, DynamicDigestOp, Error, ErrorType, ErrorKind};
/// # use core::fmt::Debug;
/// #
/// # #[derive(Debug)]
/// # enum MyError { UnsupportedAlgorithm, BufferTooSmall }
/// # impl Error for MyError {
/// #     fn kind(&self) -> ErrorKind { ErrorKind::UnsupportedAlgorithm }
/// # }
/// # impl ErrorType for MyRegistry {
/// #     type Error = MyError;
/// # }
/// # struct MyRegistry;
/// # struct MyDigestOp { algorithm_id: u32 }
/// # impl DynamicDigestOp for MyDigestOp {
/// #     type Error = MyError;
/// #     type AlgorithmId = u32;
/// #     fn update(&mut self, _input: &[u8]) -> Result<(), Self::Error> { Ok(()) }
/// #     fn finalize(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     fn output_size(&self) -> usize { 32 }
/// #     fn algorithm_id(&self) -> Self::AlgorithmId { self.algorithm_id }
/// #     fn copy_output(&self, output: &mut [u8]) -> Result<usize, Self::Error> {
/// #         if output.len() < 32 { return Err(MyError::BufferTooSmall); }
/// #         output[..32].fill(0xAB); Ok(32)
/// #     }
/// # }
/// # impl DigestRegistry for MyRegistry {
/// #     type AlgorithmId = u32;
/// #     type DigestOp = Box<dyn DynamicDigestOp<Error = MyError, AlgorithmId = u32>>;
/// #     fn supports_algorithm(&self, _: u32) -> bool { true }
/// #     fn get_output_size(&self, _: u32) -> Option<usize> { Some(32) }
/// #     fn create_digest(&mut self, algorithm_id: u32) -> Result<Self::DigestOp, Self::Error> {
/// #         Ok(Box::new(MyDigestOp { algorithm_id }))
/// #     }
/// #     fn supported_algorithms(&self) -> &[u32] { &[1, 2, 3] }
/// # }
/// // Registry creates trait objects
/// let mut registry = MyRegistry;
/// let algorithm_id = 1u32;
/// let mut digest_op = registry.create_digest(algorithm_id)?;
///
/// // Use through dynamic dispatch
/// let message_data = b"hello world";
/// digest_op.update(message_data)?;
/// digest_op.finalize()?;
/// 
/// // Get the result
/// let mut output = [0u8; 64];
/// let size = digest_op.copy_output(&mut output)?;
/// let digest_result = &output[..size];
/// # Ok::<(), MyError>(())
/// ```
///
/// ## Object Safety Requirements Met
///
/// The trait is object safe because it satisfies all requirements:
///
/// - ✅ **No `Self` parameters**: All methods use `&self`, `&mut self`, or consume `self`
/// - ✅ **No generic methods**: Methods don't have their own generic parameters  
/// - ✅ **No associated constants**: Only associated types are used
/// - ✅ **No `Sized` bound**: The trait doesn't require `Self: Sized`
///
/// ## Usage Patterns
///
/// ### Embedded/No-Std Environments
/// 
/// The trait is designed for embedded use with fixed-size buffers:
///
/// ```rust
/// # use proposed_traits::digest::{DynamicDigestOp, Error, ErrorKind};
/// # use core::fmt::Debug;
/// # #[derive(Debug)]
/// # enum MyError { BufferTooSmall }
/// # impl Error for MyError {
/// #     fn kind(&self) -> ErrorKind { ErrorKind::InvalidOutputSize }
/// # }
/// # struct MockDigest;
/// # impl DynamicDigestOp for MockDigest {
/// #     type Error = MyError;
/// #     type AlgorithmId = u32;
/// #     fn update(&mut self, _input: &[u8]) -> Result<(), Self::Error> { Ok(()) }
/// #     fn finalize(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     fn output_size(&self) -> usize { 32 }
/// #     fn algorithm_id(&self) -> Self::AlgorithmId { 1 }
/// #     fn copy_output(&self, output: &mut [u8]) -> Result<usize, Self::Error> {
/// #         if output.len() < 32 { return Err(MyError::BufferTooSmall); }
/// #         output[..32].fill(0xAB); Ok(32)
/// #     }
/// # }
/// # let mut digest_op = MockDigest;
/// let mut output_buffer = [0u8; 64]; // Fixed-size buffer
/// let bytes_written = digest_op.copy_output(&mut output_buffer)?;
/// let actual_digest = &output_buffer[..bytes_written];
/// assert_eq!(actual_digest.len(), 32);
/// # Ok::<(), MyError>(())
/// ```
///
/// ### Protocol Negotiation
///
/// Perfect for protocols like SPDM, TLS, or SSH where digest algorithms
/// are negotiated dynamically:
///
/// ```rust
/// # use proposed_traits::digest::{DynamicDigestOp, DigestRegistry, Error, ErrorType, ErrorKind};
/// # use core::fmt::Debug;
/// # #[derive(Debug)]
/// # enum MyError { UnsupportedAlgorithm, NegotiationFailed }
/// # impl Error for MyError {
/// #     fn kind(&self) -> ErrorKind { ErrorKind::UnsupportedAlgorithm }
/// # }
/// # impl ErrorType for MyRegistry {
/// #     type Error = MyError;
/// # }
/// # struct MyRegistry { supported: &'static [u32] }
/// # struct MyDigestOp { algorithm_id: u32 }
/// # impl DynamicDigestOp for MyDigestOp {
/// #     type Error = MyError;
/// #     type AlgorithmId = u32;
/// #     fn update(&mut self, _input: &[u8]) -> Result<(), Self::Error> { Ok(()) }
/// #     fn finalize(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     fn output_size(&self) -> usize { 32 }
/// #     fn algorithm_id(&self) -> Self::AlgorithmId { self.algorithm_id }
/// #     fn copy_output(&self, output: &mut [u8]) -> Result<usize, Self::Error> { Ok(32) }
/// # }
/// # impl DigestRegistry for MyRegistry {
/// #     type AlgorithmId = u32;
/// #     type DigestOp = Box<dyn DynamicDigestOp<Error = MyError, AlgorithmId = u32>>;
/// #     fn supports_algorithm(&self, algorithm_id: u32) -> bool {
/// #         self.supported.contains(&algorithm_id)
/// #     }
/// #     fn get_output_size(&self, _: u32) -> Option<usize> { Some(32) }
/// #     fn create_digest(&mut self, algorithm_id: u32) -> Result<Self::DigestOp, Self::Error> {
/// #         if self.supports_algorithm(algorithm_id) {
/// #             Ok(Box::new(MyDigestOp { algorithm_id }))
/// #         } else {
/// #             Err(MyError::UnsupportedAlgorithm)
/// #         }
/// #     }
/// #     fn supported_algorithms(&self) -> &[u32] { self.supported }
/// # }
/// # fn negotiate_with_peer(supported: &[u32]) -> Result<u32, MyError> {
/// #     supported.first().copied().ok_or(MyError::NegotiationFailed)
/// # }
/// # let mut registry = MyRegistry { supported: &[1, 2, 3] };
/// # let supported_algorithms = registry.supported_algorithms();
/// // Negotiate algorithm with peer
/// let negotiated_algorithm = negotiate_with_peer(supported_algorithms)?;
/// 
/// // Create appropriate digest operation
/// let mut digest_op = registry.create_digest(negotiated_algorithm)?;
/// 
/// // Process protocol messages
/// let protocol_message = b"SPDM protocol message";
/// digest_op.update(protocol_message)?;
/// # Ok::<(), MyError>(())
/// ```
pub trait DynamicDigestOp {
    /// The type of error returned by digest operations.
    type Error: Error;
    
    /// The type of algorithm identifier for this digest operation.
    type AlgorithmId: Copy + Debug + PartialEq;

    /// Updates the digest state with the provided input data.
    ///
    /// # Parameters
    ///
    /// - `input`: A byte slice containing the data to hash.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn update(&mut self, input: &[u8]) -> Result<(), Self::Error>;

    /// Finalizes the digest computation.
    ///
    /// This method finalizes the digest computation internally. After calling
    /// this method, use `copy_output()` to retrieve the computed digest.
    /// This design allows the trait to remain object-safe by avoiding 
    /// consuming `self` by value.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure of finalization.
    fn finalize(&mut self) -> Result<(), Self::Error>;

    /// Returns the output size in bytes for this digest algorithm.
    ///
    /// # Returns
    ///
    /// The number of bytes this digest algorithm produces.
    fn output_size(&self) -> usize;

    /// Returns the algorithm identifier for this digest operation.
    ///
    /// # Returns
    ///
    /// The algorithm identifier that was used to create this operation.
    fn algorithm_id(&self) -> Self::AlgorithmId;
    
    /// Copies the current digest output to the provided buffer.
    ///
    /// This method should be called after finalize() to retrieve the result.
    /// The implementation should copy the digest output to the buffer starting
    /// at offset 0, up to the output_size() number of bytes.
    ///
    /// # Parameters
    ///
    /// - `output`: A mutable byte slice to copy the digest output into.
    ///
    /// # Returns
    ///
    /// A result containing the number of bytes copied, or an error.
    fn copy_output(&self, output: &mut [u8]) -> Result<usize, Self::Error>;
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

/// Trait representing a digest algorithm and its output characteristics.
pub trait DigestAlgorithm {
    /// The number of bits in the digest output.
    const OUTPUT_BITS: usize;

    /// The type representing the digest output.
    type DigestOutput: AsRef<[u8]>;
}

/// Trait for initializing a digest operation for a specific algorithm.
pub trait DigestInit<A: DigestAlgorithm>: ErrorType {
    /// The type representing the operational context for the digest.
    type OpContext<'a>: DigestOp<Output = A::DigestOutput>
    where
        Self: 'a;

    /// Initializes the digest operation with the specified algorithm.
    ///
    /// # Parameters
    ///
    /// - `algo`: A zero-sized type representing the digest algorithm to use.
    ///   While this parameter is technically redundant (since the algorithm type
    ///   is already specified in the trait bound), it serves important purposes:
    ///   * **API clarity**: Makes call sites self-documenting
    ///   * **Type inference**: Helps the compiler choose the correct implementation
    ///   * **Consistency**: Follows established patterns in cryptographic APIs
    ///
    /// # Returns
    ///
    /// A result containing the operational context for the digest, or an error.
    fn init<'a>(&'a mut self, algo: A) -> Result<Self::OpContext<'a>, Self::Error>;
}

/// Optional trait for resetting a digest context to its initial state.
pub trait DigestCtrlReset: ErrorType {
    /// Resets the digest context.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn reset(&mut self) -> Result<(), Self::Error>;
}

/// Trait for performing digest operations.
pub trait DigestOp: ErrorType {
    /// The type of the digest output.
    type Output;

    /// Updates the digest state with the provided input data.
    ///
    /// # Parameters
    ///
    /// - `input`: A byte slice containing the data to hash.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    fn update(&mut self, input: &[u8]) -> Result<(), Self::Error>;

    /// Finalizes the digest computation and returns the result.
    ///
    /// # Returns
    ///
    /// A result containing the digest output, or an error.
    fn finalize(self) -> Result<Self::Output, Self::Error>;
}
