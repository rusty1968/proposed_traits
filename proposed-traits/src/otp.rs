/// Represents the category of an error that occurred during OTP memory operations.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ErrorKind {
    /// The specified address is out of bounds or invalid.
    InvalidAddress,

    /// The memory is locked and cannot be written to.
    MemoryLocked,

    /// A write operation failed due to hardware or timing issues.
    WriteFailed,

    /// A read operation failed due to hardware or timing issues.
    ReadFailed,

    /// The lock operation failed or was not acknowledged.
    LockFailed,

    /// Programming verification failed after write.
    VerificationFailed,

    /// No remaining write attempts for this location.
    WriteExhausted,

    /// Session not established or expired.
    NoSession,

    /// Region is protected and cannot be modified.
    RegionProtected,

    /// Address alignment error.
    AlignmentError,

    /// Buffer length exceeds boundaries.
    BoundaryError,

    /// Hardware access timeout.
    Timeout,

    /// An unspecified or unknown error occurred.
    Unknown,
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

/// A generic trait representing a One-Time Programmable (OTP) memory interface.
///
/// This trait abstracts the basic operations for interacting with OTP memory,
/// which is typically used for storing immutable configuration data such as
/// device IDs, cryptographic keys, or calibration values.
///
/// The trait is generic over the data type `T`, allowing implementations for
/// various word widths (e.g., `u8`, `u16`, `u32`, `u64`).
///
/// # Type Parameters
///
/// - `T`: The data type used for memory operations. Must implement `Copy` and `Default`.
///
/// # Errors
///
/// All methods return a `Result` to handle potential errors such as invalid
/// addresses or attempts to write to locked memory.
pub trait OtpMemory<T>: ErrorType
where
    T: Copy + Default,
{
    /// Reads a value of type `T` from the specified memory address.
    ///
    /// # Parameters
    /// - `address`: The offset from the base address of the OTP memory.
    ///
    /// # Returns
    /// - `Ok(T)`: The value read from memory.
    /// - `Err(OtpError)`: If the address is invalid or inaccessible.
    fn read(&self, address: usize) -> Result<T, Self::Error>;

    /// Writes a value of type `T` to the specified memory address.
    ///
    /// # Parameters
    /// - `address`: The offset from the base address of the OTP memory.
    /// - `data`: The value to write.
    ///
    /// # Returns
    /// - `Ok(())`: If the write was successful.
    /// - `Err(OtpError)`: If the memory is locked or the address is invalid.
    fn write(&mut self, address: usize, data: T) -> Result<(), Self::Error>;

    /// Permanently locks the OTP memory to prevent further writes.
    ///
    /// # Returns
    /// - `Ok(())`: If the lock operation was successful.
    /// - `Err(OtpError)`: If the lock operation failed.
    fn lock(&mut self) -> Result<(), Self::Error>;

    /// Checks whether the OTP memory is currently locked.
    ///
    /// # Returns
    /// - `true`: If the memory is locked.
    /// - `false`: If the memory is still writable.
    fn is_locked(&self) -> bool;
}

/// Optional trait for OTP devices that support session-based access control.
///
/// Session management provides controlled access to OTP operations with proper
/// hardware locking and resource management.
pub trait OtpSession: ErrorType {
    /// Session information type returned when establishing a session
    type SessionInfo;

    /// Establish an OTP session with hardware access
    ///
    /// # Returns
    /// - `Ok(SessionInfo)`: Session established successfully
    /// - `Err(Self::Error)`: Failed to establish session
    fn begin_session(&mut self) -> Result<Self::SessionInfo, Self::Error>;

    /// Terminate the OTP session and release resources
    ///
    /// # Returns
    /// - `Ok(())`: Session terminated successfully
    /// - `Err(Self::Error)`: Failed to terminate session properly
    fn end_session(&mut self) -> Result<(), Self::Error>;

    /// Check if a session is currently active
    fn is_session_active(&self) -> bool;
}

/// Optional trait for OTP devices that support multiple memory regions.
///
/// Many OTP devices have different regions for different purposes (data, config, etc.)
/// with different sizes, alignment requirements, and protection levels.
pub trait OtpRegions<T>: ErrorType
where
    T: Copy + Default,
{
    /// Region identifier type
    type Region: Copy + core::fmt::Debug + PartialEq;

    /// Read data from a specific OTP region
    ///
    /// # Parameters
    /// - `region`: The region to read from
    /// - `offset`: Offset within the region
    /// - `buffer`: Buffer to store read data
    ///
    /// # Returns
    /// - `Ok(())`: Data read successfully
    /// - `Err(Self::Error)`: Read operation failed
    fn read_region(&self, region: Self::Region, offset: usize, buffer: &mut [T]) -> Result<(), Self::Error>;

    /// Write data to a specific OTP region
    ///
    /// # Parameters
    /// - `region`: The region to write to
    /// - `offset`: Offset within the region
    /// - `data`: Data to write
    ///
    /// # Returns
    /// - `Ok(())`: Data written successfully
    /// - `Err(Self::Error)`: Write operation failed
    fn write_region(&mut self, region: Self::Region, offset: usize, data: &[T]) -> Result<(), Self::Error>;

    /// Get the capacity of a specific region
    ///
    /// # Parameters
    /// - `region`: The region to query
    ///
    /// # Returns
    /// The capacity of the region in elements of type T
    fn region_capacity(&self, region: Self::Region) -> usize;

    /// Get the alignment requirement for a specific region
    ///
    /// # Parameters
    /// - `region`: The region to query
    ///
    /// # Returns
    /// The alignment requirement in bytes
    fn region_alignment(&self, region: Self::Region) -> usize;
}

/// Optional trait for OTP devices that support protection mechanisms.
///
/// Protection allows locking regions or the entire memory to prevent
/// further modifications, providing security for critical data.
pub trait OtpProtection: ErrorType {
    /// Region identifier type (if regions are supported)
    type Region: Copy + core::fmt::Debug + PartialEq;

    /// Check if a specific region is protected
    ///
    /// # Parameters
    /// - `region`: The region to check
    ///
    /// # Returns
    /// - `Ok(bool)`: Protection status (true = protected)
    /// - `Err(Self::Error)`: Failed to check protection status
    fn is_region_protected(&self, region: Self::Region) -> Result<bool, Self::Error>;

    /// Enable protection for a specific region
    ///
    /// # Parameters
    /// - `region`: The region to protect
    ///
    /// # Returns
    /// - `Ok(())`: Protection enabled successfully
    /// - `Err(Self::Error)`: Failed to enable protection
    fn enable_region_protection(&mut self, region: Self::Region) -> Result<(), Self::Error>;

    /// Check if the entire memory is globally locked
    ///
    /// # Returns
    /// - `Ok(bool)`: Lock status (true = locked)
    /// - `Err(Self::Error)`: Failed to check lock status
    fn is_globally_locked(&self) -> Result<bool, Self::Error>;

    /// Enable global memory lock (typically irreversible)
    ///
    /// This operation permanently locks all OTP regions and usually cannot be undone.
    /// Use with extreme caution.
    ///
    /// # Returns
    /// - `Ok(())`: Global lock enabled successfully
    /// - `Err(Self::Error)`: Failed to enable global lock
    fn enable_global_lock(&mut self) -> Result<(), Self::Error>;
}

/// Optional trait for OTP devices that track write attempts.
///
/// Some OTP technologies have limited write attempts per location,
/// especially for technologies like eFuse or anti-fuse.
pub trait OtpWriteTracking<T>: ErrorType
where
    T: Copy + Default,
{
    /// Get the number of remaining write attempts for a specific address
    ///
    /// # Parameters
    /// - `address`: The address to check
    ///
    /// # Returns
    /// - `Ok(u32)`: Number of remaining write attempts
    /// - `Err(Self::Error)`: Failed to get write attempt count
    fn remaining_writes(&self, address: usize) -> Result<u32, Self::Error>;

    /// Check if a specific address is still writable
    ///
    /// # Parameters
    /// - `address`: The address to check
    ///
    /// # Returns
    /// - `Ok(bool)`: Writability status (true = writable)
    /// - `Err(Self::Error)`: Failed to check writability
    fn is_writable(&self, address: usize) -> Result<bool, Self::Error>;

    /// Get the total number of write attempts allowed for this device
    ///
    /// # Returns
    /// The maximum number of write attempts per location
    fn max_write_attempts(&self) -> u32;
}

/// Optional trait for OTP devices that support verification operations.
///
/// Verification ensures that programmed data matches the intended values,
/// which is critical for reliability in OTP memory.
pub trait OtpVerification<T>: ErrorType
where
    T: Copy + Default + PartialEq,
{
    /// Verify that data at a specific address matches expected values
    ///
    /// # Parameters
    /// - `address`: The address to verify
    /// - `expected`: The expected data values
    ///
    /// # Returns
    /// - `Ok(())`: Verification successful, data matches
    /// - `Err(Self::Error)`: Verification failed or data mismatch
    fn verify(&self, address: usize, expected: &[T]) -> Result<(), Self::Error>;

    /// Program and verify data in a single operation
    ///
    /// This method must be implemented by the specific OTP device as it requires
    /// access to both programming and verification capabilities.
    ///
    /// # Parameters
    /// - `address`: The address to program
    /// - `data`: The data to program
    ///
    /// # Returns
    /// - `Ok(())`: Programming and verification successful
    /// - `Err(Self::Error)`: Programming or verification failed
    fn program_and_verify(&mut self, address: usize, data: &[T]) -> Result<(), Self::Error>;
}

/// Optional trait for OTP devices that provide chip identification.
///
/// This trait allows querying hardware information and version details,
/// which is useful for feature detection and compatibility checks.
pub trait OtpIdentification: ErrorType {
    /// Chip version or identifier type
    type ChipVersion: Copy + core::fmt::Debug + PartialEq;

    /// Get the chip version or hardware identifier
    ///
    /// # Returns
    /// - `Ok(ChipVersion)`: Hardware version information
    /// - `Err(Self::Error)`: Failed to read chip identification
    fn get_chip_version(&self) -> Result<Self::ChipVersion, Self::Error>;

    /// Check if a specific feature is supported by this chip version
    ///
    /// # Parameters
    /// - `feature`: Feature identifier to check
    ///
    /// # Returns
    /// - `Ok(bool)`: Feature support status (true = supported)
    /// - `Err(Self::Error)`: Failed to check feature support
    fn is_feature_supported(&self, feature: &str) -> Result<bool, Self::Error>;
}

/// Optional trait for OTP devices that support bulk memory operations.
///
/// Bulk operations can provide better performance and atomic behavior
/// when working with larger amounts of data.
pub trait OtpBulkOperations<T>: ErrorType
where
    T: Copy + Default,
{
    /// Read multiple values from consecutive addresses
    ///
    /// # Parameters
    /// - `start_address`: Starting address for the read operation
    /// - `buffer`: Buffer to store the read data
    ///
    /// # Returns
    /// - `Ok(())`: All data read successfully
    /// - `Err(Self::Error)`: Read operation failed
    fn read_bulk(&self, start_address: usize, buffer: &mut [T]) -> Result<(), Self::Error>;

    /// Write multiple values to consecutive addresses
    ///
    /// This operation may be atomic depending on the hardware implementation.
    ///
    /// # Parameters
    /// - `start_address`: Starting address for the write operation
    /// - `data`: Data to write
    ///
    /// # Returns
    /// - `Ok(())`: All data written successfully
    /// - `Err(Self::Error)`: Write operation failed
    fn write_bulk(&mut self, start_address: usize, data: &[T]) -> Result<(), Self::Error>;

    /// Get the maximum bulk operation size supported by the device
    ///
    /// # Returns
    /// Maximum number of elements that can be processed in a single bulk operation
    fn max_bulk_size(&self) -> usize;
}

/// Optional trait for OTP devices with advanced memory layout information.
///
/// This trait provides detailed information about memory organization,
/// which is useful for optimizing data placement and understanding limitations.
pub trait OtpMemoryLayout: ErrorType {
    /// Region identifier type
    type Region: Copy + core::fmt::Debug + PartialEq;

    /// Get the total memory capacity in bytes
    fn total_capacity(&self) -> usize;

    /// Get the minimum alignment requirement for write operations
    ///
    /// # Returns
    /// Alignment requirement in bytes (e.g., 1, 4, 8)
    fn write_alignment(&self) -> usize;

    /// Get the size of the minimum programmable unit
    ///
    /// # Returns
    /// Size in bytes of the smallest unit that can be programmed independently
    fn programming_granularity(&self) -> usize;

    /// List all available memory regions
    ///
    /// Returns an iterator over available regions. The exact collection type
    /// depends on the implementation (could be array, slice, or heap-allocated).
    ///
    /// # Returns
    /// - `Ok(regions)`: Iterator over available regions
    /// - `Err(Self::Error)`: Failed to enumerate regions
    fn list_regions(&self) -> Result<&[Self::Region], Self::Error>;

    /// Get detailed information about a specific region
    ///
    /// # Parameters
    /// - `region`: The region to query
    ///
    /// # Returns
    /// - `Ok((start_addr, size, alignment))`: Region details
    /// - `Err(Self::Error)`: Failed to get region information
    fn get_region_info(&self, region: Self::Region) -> Result<(usize, usize, usize), Self::Error>;
}

/// Enhanced error information for OTP operations.
///
/// This enum extends the basic ErrorKind with additional context
/// that can help with debugging and error recovery.
#[derive(Debug, Clone, PartialEq)]
pub enum OtpErrorInfo {
    /// Basic error with just the kind
    Simple(ErrorKind),
    /// Error with additional context message
    WithMessage(ErrorKind, &'static str),
    /// Error with address information
    WithAddress(ErrorKind, usize),
    /// Error with both address and message
    WithContext(ErrorKind, usize, &'static str),
}

impl OtpErrorInfo {
    /// Get the underlying error kind
    pub fn kind(&self) -> ErrorKind {
        match self {
            OtpErrorInfo::Simple(kind) => *kind,
            OtpErrorInfo::WithMessage(kind, _) => *kind,
            OtpErrorInfo::WithAddress(kind, _) => *kind,
            OtpErrorInfo::WithContext(kind, _, _) => *kind,
        }
    }

    /// Get the associated address if available
    pub fn address(&self) -> Option<usize> {
        match self {
            OtpErrorInfo::WithAddress(_, addr) => Some(*addr),
            OtpErrorInfo::WithContext(_, addr, _) => Some(*addr),
            _ => None,
        }
    }

    /// Get the associated message if available
    pub fn message(&self) -> Option<&'static str> {
        match self {
            OtpErrorInfo::WithMessage(_, msg) => Some(msg),
            OtpErrorInfo::WithContext(_, _, msg) => Some(msg),
            _ => None,
        }
    }
}

/// Optional trait for OTP devices that support different data word sizes.
///
/// This trait allows working with OTP memories that support multiple
/// access patterns (byte, word, double-word) on the same device.
pub trait OtpMultiWidth: ErrorType {
    /// Read an 8-bit value
    fn read_u8(&self, address: usize) -> Result<u8, Self::Error>;
    
    /// Read a 16-bit value
    fn read_u16(&self, address: usize) -> Result<u16, Self::Error>;
    
    /// Read a 32-bit value
    fn read_u32(&self, address: usize) -> Result<u32, Self::Error>;
    
    /// Read a 64-bit value
    fn read_u64(&self, address: usize) -> Result<u64, Self::Error>;
    
    /// Write an 8-bit value
    fn write_u8(&mut self, address: usize, data: u8) -> Result<(), Self::Error>;
    
    /// Write a 16-bit value
    fn write_u16(&mut self, address: usize, data: u16) -> Result<(), Self::Error>;
    
    /// Write a 32-bit value
    fn write_u32(&mut self, address: usize, data: u32) -> Result<(), Self::Error>;
    
    /// Write a 64-bit value
    fn write_u64(&mut self, address: usize, data: u64) -> Result<(), Self::Error>;
    
    /// Get the native word width of the device in bytes
    fn native_width(&self) -> usize;
}

/// Optional trait for OTP devices that support "soak" programming.
///
/// Soak programming is a technique for programming difficult bits that may not
/// respond to normal programming pulses. It applies extended programming timing
/// to ensure reliable programming of all bits, especially those that are harder
/// to program due to process variations or physical characteristics.
///
/// This technique is commonly used in semiconductor programming and is not
/// specific to any particular vendor or technology.
pub trait OtpSoakProgramming<T>: ErrorType
where
    T: Copy + Default,
{
    /// Configuration for soak programming parameters
    type SoakConfig: Copy + core::fmt::Debug;

    /// Program data using extended "soak" timing for difficult bits
    ///
    /// Soak programming applies longer programming pulses to ensure that
    /// difficult-to-program bits are reliably written. This is particularly
    /// useful for:
    /// - Bits that fail normal programming due to process variations
    /// - Critical data that requires maximum reliability
    /// - OTP technologies sensitive to programming timing
    ///
    /// # Parameters
    /// - `address`: The address to program
    /// - `data`: The data to program
    /// - `config`: Soak programming configuration (timing, pulse width, etc.)
    ///
    /// # Returns
    /// - `Ok(())`: Soak programming completed successfully
    /// - `Err(Self::Error)`: Programming failed even with extended timing
    fn soak_program(&mut self, address: usize, data: &[T], config: Self::SoakConfig) -> Result<(), Self::Error>;

    /// Get the default soak programming configuration
    ///
    /// Returns a conservative configuration suitable for most use cases.
    /// Users can customize this for specific requirements.
    ///
    /// # Returns
    /// Default soak programming configuration
    fn default_soak_config(&self) -> Self::SoakConfig;

    /// Check if soak programming is available for a specific address
    ///
    /// Some OTP devices may have restrictions on which regions or addresses
    /// support soak programming.
    ///
    /// # Parameters
    /// - `address`: The address to check
    ///
    /// # Returns
    /// - `Ok(bool)`: Soak programming availability (true = available)
    /// - `Err(Self::Error)`: Failed to check availability
    fn is_soak_available(&self, address: usize) -> Result<bool, Self::Error>;

    /// Verify data and apply soak programming if verification fails
    ///
    /// This method combines normal programming with automatic fallback to
    /// soak programming for any bits that fail verification.
    ///
    /// # Parameters
    /// - `address`: The address to program
    /// - `data`: The data to program
    /// - `config`: Soak programming configuration for fallback
    ///
    /// # Returns
    /// - `Ok(())`: Programming and verification successful
    /// - `Err(Self::Error)`: Programming failed even with soak programming
    fn program_with_soak_fallback(&mut self, address: usize, data: &[T], config: Self::SoakConfig) -> Result<(), Self::Error>;

    /// Get recommended soak configuration for specific data patterns
    ///
    /// Some data patterns may require different soak parameters for optimal
    /// programming reliability.
    ///
    /// # Parameters
    /// - `data`: The data pattern to be programmed
    ///
    /// # Returns
    /// - `Ok(SoakConfig)`: Recommended configuration for this data
    /// - `Err(Self::Error)`: Failed to determine optimal configuration
    fn recommend_soak_config(&self, data: &[T]) -> Result<Self::SoakConfig, Self::Error>;
}

/// Example implementation demonstrating composable soak programming.
///
/// This shows how the `OtpSoakProgramming` trait can be implemented
/// alongside the basic `OtpMemory` trait to provide enhanced programming
/// capabilities for difficult bits.
pub mod examples {
    use super::*;

    /// Example soak programming configuration
    #[derive(Debug, Copy, Clone)]
    pub struct SoakConfig {
        /// Extended programming pulse duration (in microseconds)
        pub pulse_duration_us: u32,
        /// Number of programming attempts with extended timing
        pub retry_count: u8,
        /// Verification delay after programming (in microseconds)
        pub verify_delay_us: u32,
    }

    impl Default for SoakConfig {
        fn default() -> Self {
            Self {
                pulse_duration_us: 100,  // 100μs extended pulse
                retry_count: 3,          // Up to 3 retry attempts
                verify_delay_us: 10,     // 10μs settle time before verify
            }
        }
    }

    /// Example error type for demonstration
    #[derive(Debug, Copy, Clone)]
    pub enum ExampleOtpError {
        InvalidAddress,
        WriteFailed,
        VerificationFailed,
        SoakNotSupported,
    }

    impl Error for ExampleOtpError {
        fn kind(&self) -> ErrorKind {
            match self {
                ExampleOtpError::InvalidAddress => ErrorKind::InvalidAddress,
                ExampleOtpError::WriteFailed => ErrorKind::WriteFailed,
                ExampleOtpError::VerificationFailed => ErrorKind::VerificationFailed,
                ExampleOtpError::SoakNotSupported => ErrorKind::Unknown,
            }
        }
    }

    /// Example OTP device that implements both basic and soak programming
    pub struct ExampleOtpDevice {
        memory: [u32; 1024],
        locked: bool,
        soak_supported_regions: core::ops::Range<usize>,
    }

    impl ExampleOtpDevice {
        pub fn new() -> Self {
            Self {
                memory: [0; 1024],
                locked: false,
                soak_supported_regions: 0..512,  // First half supports soak
            }
        }
    }

    impl ErrorType for ExampleOtpDevice {
        type Error = ExampleOtpError;
    }

    impl OtpMemory<u32> for ExampleOtpDevice {
        fn read(&self, address: usize) -> Result<u32, Self::Error> {
            if address >= self.memory.len() {
                return Err(ExampleOtpError::InvalidAddress);
            }
            Ok(self.memory[address])
        }

        fn write(&mut self, address: usize, data: u32) -> Result<(), Self::Error> {
            if address >= self.memory.len() {
                return Err(ExampleOtpError::InvalidAddress);
            }
            if self.locked {
                return Err(ExampleOtpError::WriteFailed);
            }
            
            self.memory[address] = data;
            Ok(())
        }

        fn lock(&mut self) -> Result<(), Self::Error> {
            self.locked = true;
            Ok(())
        }

        fn is_locked(&self) -> bool {
            self.locked
        }
    }

    impl OtpSoakProgramming<u32> for ExampleOtpDevice {
        type SoakConfig = SoakConfig;

        fn soak_program(&mut self, address: usize, data: &[u32], config: Self::SoakConfig) -> Result<(), Self::Error> {
            // Check if soak programming is available for this address
            if !self.is_soak_available(address)? {
                return Err(ExampleOtpError::SoakNotSupported);
            }

            for (i, &value) in data.iter().enumerate() {
                let addr = address + i;
                if addr >= self.memory.len() {
                    return Err(ExampleOtpError::InvalidAddress);
                }

                // Simulate extended programming with retries
                let mut success = false;
                for _attempt in 0..config.retry_count {
                    // Simulate extended programming pulse
                    // In a real implementation, this would configure hardware timing
                    self.memory[addr] = value;
                    
                    // Simulate verification delay
                    // In a real implementation, this would be a hardware delay
                    
                    // Verify the programming was successful
                    if self.memory[addr] == value {
                        success = true;
                        break;
                    }
                }

                if !success {
                    return Err(ExampleOtpError::VerificationFailed);
                }
            }

            Ok(())
        }

        fn default_soak_config(&self) -> Self::SoakConfig {
            SoakConfig::default()
        }

        fn is_soak_available(&self, address: usize) -> Result<bool, Self::Error> {
            Ok(self.soak_supported_regions.contains(&address))
        }

        fn program_with_soak_fallback(&mut self, address: usize, data: &[u32], config: Self::SoakConfig) -> Result<(), Self::Error> {
            // Try normal programming first
            for (i, &value) in data.iter().enumerate() {
                let addr = address + i;
                if let Err(_) = self.write(addr, value) {
                    // Normal programming failed, try soak programming
                    return self.soak_program(address, data, config);
                }
                
                // Verify the normal programming worked
                if self.read(addr)? != value {
                    // Verification failed, try soak programming
                    return self.soak_program(address, data, config);
                }
            }
            
            Ok(())
        }

        fn recommend_soak_config(&self, data: &[u32]) -> Result<Self::SoakConfig, Self::Error> {
            // Simple heuristic: if data has many bits set, use longer pulses
            let total_bits = data.len() * 32;
            let set_bits: u32 = data.iter().map(|x| x.count_ones()).sum();
            let density = set_bits as f32 / total_bits as f32;

            let config = if density > 0.8 {
                // High bit density: use extended timing
                SoakConfig {
                    pulse_duration_us: 200,
                    retry_count: 5,
                    verify_delay_us: 20,
                }
            } else if density > 0.5 {
                // Medium bit density: use moderate timing
                SoakConfig {
                    pulse_duration_us: 150,
                    retry_count: 4,
                    verify_delay_us: 15,
                }
            } else {
                // Low bit density: use standard timing
                self.default_soak_config()
            };

            Ok(config)
        }
    }

    /// Example usage demonstrating the composable traits
    #[cfg(test)]
    mod usage_examples {
        use super::{ExampleOtpDevice, OtpMemory, OtpSoakProgramming};

        #[test]
        fn basic_otp_usage() {
            let mut device = ExampleOtpDevice::new();
            
            // Normal OTP operations
            assert_eq!(device.read(0).unwrap(), 0);
            device.write(0, 0x12345678).unwrap();
            assert_eq!(device.read(0).unwrap(), 0x12345678);
        }

        #[test]
        fn soak_programming_usage() {
            let mut device = ExampleOtpDevice::new();
            
            // Use soak programming for difficult data
            let difficult_data = [0xFFFFFFFF, 0xAAAAAAAA, 0x55555555];
            let config = device.default_soak_config();
            
            device.soak_program(0, &difficult_data, config).unwrap();
            
            // Verify the data was programmed correctly
            for (i, &expected) in difficult_data.iter().enumerate() {
                assert_eq!(device.read(i).unwrap(), expected);
            }
        }

        #[test]
        fn automatic_fallback_usage() {
            let mut device = ExampleOtpDevice::new();
            
            // This will try normal programming first, then fall back to soak if needed
            let data = [0xDEADBEEF, 0xCAFEBABE];
            let config = device.recommend_soak_config(&data).unwrap();
            
            device.program_with_soak_fallback(0, &data, config).unwrap();
            
            // Verify programming was successful
            assert_eq!(device.read(0).unwrap(), 0xDEADBEEF);
            assert_eq!(device.read(1).unwrap(), 0xCAFEBABE);
        }
    }
}
