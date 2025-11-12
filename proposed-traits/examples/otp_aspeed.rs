//! ASPEED-specific OTP (One-Time Programmable) memory traits
//!
//! This module provides specialized traits for ASPEED SoC OTP memory controllers,
//! extending the generic OTP interface to support ASPEED-specific features such as:
//! - Multiple memory regions (data, configuration, strap)
//! - Hardware strap programming with limited write attempts
//! - Protection mechanisms and security features
//! - Session-based access control
//!
//! # Supported ASPEED SoCs
//! - AST1030A0/A1
//! - AST1035A1
//! - AST1060A1/A2

use core::fmt::Debug;
use proposed_traits::otp::{ErrorType, Error, ErrorKind, OtpSession, OtpRegions};

/// ASPEED-specific OTP error kinds extending the generic error kinds
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum AspeedErrorKind {
    /// Generic OTP error
    Generic(ErrorKind),
    /// Strap bit has no remaining write attempts
    StrapExhausted,
    /// Invalid strap bit offset  
    InvalidStrapBit,
    /// Unsupported chip version
    UnsupportedChip,
    /// Hardware is in locked state
    HardwareLocked,
    /// Invalid image format or checksum
    InvalidImage,
}

/// ASPEED-specific error type that implements the generic Error trait
#[derive(Debug, Copy, Clone)]
pub struct AspeedOtpError {
    kind: AspeedErrorKind,
}

impl AspeedOtpError {
    pub fn new(kind: AspeedErrorKind) -> Self {
        Self { kind }
    }
    
    pub fn aspeed_kind(&self) -> AspeedErrorKind {
        self.kind
    }
}

impl Error for AspeedOtpError {
    fn kind(&self) -> ErrorKind {
        match self.kind {
            AspeedErrorKind::Generic(kind) => kind,
            AspeedErrorKind::StrapExhausted => ErrorKind::WriteExhausted,
            AspeedErrorKind::InvalidStrapBit => ErrorKind::InvalidAddress,
            AspeedErrorKind::UnsupportedChip => ErrorKind::Unknown,
            AspeedErrorKind::HardwareLocked => ErrorKind::MemoryLocked,
            AspeedErrorKind::InvalidImage => ErrorKind::Unknown,
        }
    }
}

impl core::convert::From<ErrorKind> for AspeedOtpError {
    fn from(kind: ErrorKind) -> Self {
        Self::new(AspeedErrorKind::Generic(kind))
    }
}

/// ASPEED chip version information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspeedChipVersion {
    /// AST1030 A0 revision
    Ast1030A0,
    /// AST1030 A1 revision
    Ast1030A1,
    /// AST1035 A1 revision
    Ast1035A1,
    /// AST1060 A1 revision
    Ast1060A1,
    /// AST1060 A2 revision
    Ast1060A2,
    /// Unknown or unsupported version
    Unknown,
}

/// Memory region types in ASPEED OTP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspeedOtpRegion {
    /// Data region (2048 double-words, 0x0000-0x0FFF)
    Data,
    /// Configuration region (32 double-words, 0x800-0x81F)
    Configuration,
    /// Strap region (64 bits, multiple programming options)
    Strap,
    /// SCU protection region (2 double-words, 0x1C-0x1D)
    ScuProtection,
}

/// Protection status for different OTP regions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtectionStatus {
    /// Memory lock status (prevents all modifications)
    pub memory_locked: bool,
    /// Key return protection status
    pub key_protected: bool,
    /// Strap region protection status
    pub strap_protected: bool,
    /// Configuration region protection status
    pub config_protected: bool,
    /// Data region protection status
    pub data_protected: bool,
    /// Security region protection status
    pub security_protected: bool,
    /// Security region size in bytes
    pub security_size: u32,
}

/// Strap bit programming status
#[derive(Debug, Clone, Copy)]
pub struct StrapStatus {
    /// Current strap bit value
    pub value: bool,
    /// Programming options available
    pub options: [u8; 7],
    /// Remaining write attempts
    pub remaining_writes: u8,
    /// Next writable option
    pub writable_option: u8,
    /// Protection status for this strap bit
    pub protected: bool,
}

/// Session information provided during OTP session establishment
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Chip version detected
    pub chip_version: AspeedChipVersion,
    /// Version name string
    pub version_name: [u8; 10],
    /// Current protection status
    pub protection_status: ProtectionStatus,
    /// Tool version information
    pub tool_version: [u8; 32],
    /// Software revision ID
    pub software_revision: u32,
    /// Number of cryptographic keys stored
    pub key_count: u32,
}



/// Core ASPEED OTP session management trait
///
/// This trait extends the generic OtpSession with ASPEED-specific session information
/// and protection status management.
pub trait AspeedOtpSession: ErrorType + OtpSession<SessionInfo = SessionInfo> {
    /// Get current protection status
    ///
    /// # Returns
    /// - `Ok(ProtectionStatus)`: Current protection status
    /// - `Err(Self::Error)`: Failed to read protection status
    ///
    /// # Errors
    /// - No active session
    /// - Hardware access failure
    fn get_protection_status(&self) -> Result<ProtectionStatus, Self::Error>;
}

/// ASPEED OTP data region operations
///
/// The data region provides 2048 double-words (8KB) of general-purpose OTP storage
/// suitable for cryptographic keys, certificates, and user data.
/// 
/// This trait extends the generic OtpRegions trait for data region specific operations.
pub trait AspeedOtpData: AspeedOtpSession + OtpRegions<u32, Region = AspeedOtpRegion> {
    /// Read data from the OTP data region
    ///
    /// # Arguments
    /// - `offset`: Word offset within the data region (0-2047)
    /// - `buffer`: Buffer to store read data
    ///
    /// # Returns
    /// - `Ok(())`: Data read successfully
    /// - `Err(Self::Error)`: Read operation failed
    ///
    /// # Errors
    /// - No active session
    /// - Invalid offset or buffer length
    /// - Hardware access failure
    /// - Boundary violation (offset + buffer.len() > 2048)
    fn read_data(&self, offset: u32, buffer: &mut [u32]) -> Result<(), Self::Error> {
        self.read_region(AspeedOtpRegion::Data, offset as usize, buffer)
    }

    /// Program data to the OTP data region
    ///
    /// # Arguments
    /// - `offset`: Word offset within the data region (0-2047)
    /// - `data`: Data to program
    ///
    /// # Returns
    /// - `Ok(())`: Data programmed and verified successfully
    /// - `Err(Self::Error)`: Programming operation failed
    ///
    /// # Errors
    /// - No active session
    /// - Data region is protected
    /// - Invalid offset or data length
    /// - Programming verification failed
    /// - Hardware programming failure
    /// - Boundary violation (offset + data.len() > 2048)
    fn program_data(&mut self, offset: u32, data: &[u32]) -> Result<(), Self::Error> {
        self.write_region(AspeedOtpRegion::Data, offset as usize, data)
    }

    /// Get the maximum data region capacity in words
    fn data_capacity(&self) -> u32 {
        2048
    }
}

/// ASPEED OTP configuration region operations
///
/// The configuration region provides 32 double-words (128 bytes) for system
/// configuration data, boot parameters, and security settings.
pub trait AspeedOtpConfig: AspeedOtpSession {
    /// Read configuration data
    ///
    /// # Arguments
    /// - `offset`: Word offset within the configuration region (0-31)
    /// - `buffer`: Buffer to store read configuration data
    ///
    /// # Returns
    /// - `Ok(())`: Configuration read successfully
    /// - `Err(Self::Error)`: Read operation failed
    ///
    /// # Errors
    /// - No active session
    /// - Invalid offset or buffer length
    /// - Hardware access failure
    /// - Boundary violation (offset + buffer.len() > 32)
    fn read_config(&self, offset: u32, buffer: &mut [u32]) -> Result<(), Self::Error>;

    /// Program configuration data
    ///
    /// # Arguments
    /// - `offset`: Word offset within the configuration region (0-31)
    /// - `data`: Configuration data to program
    ///
    /// # Returns
    /// - `Ok(())`: Configuration programmed successfully
    /// - `Err(Self::Error)`: Programming operation failed
    ///
    /// # Errors
    /// - No active session
    /// - Configuration region is protected
    /// - Invalid offset or data length
    /// - Programming verification failed
    /// - Boundary violation (offset + data.len() > 32)
    fn program_config(&mut self, offset: u32, data: &[u32]) -> Result<(), Self::Error>;

    /// Get the maximum configuration region capacity in words
    fn config_capacity(&self) -> u32 {
        32
    }
}

/// ASPEED OTP strap programming operations
///
/// Hardware straps control SoC pin multiplexing, boot modes, and other
/// hardware configuration. Each strap bit has limited write attempts.
pub trait AspeedOtpStrap: AspeedOtpSession {
    /// Read all strap bits
    ///
    /// # Arguments
    /// - `buffer`: 2-word buffer to store strap bits (64 bits total)
    ///
    /// # Returns
    /// - `Ok(())`: Strap bits read successfully
    /// - `Err(Self::Error)`: Read operation failed
    ///
    /// # Errors
    /// - No active session
    /// - Invalid buffer size (must be exactly 2 words)
    /// - Hardware access failure
    fn read_straps(&self, buffer: &mut [u32; 2]) -> Result<(), Self::Error>;

    /// Get status for a specific strap bit
    ///
    /// # Arguments
    /// - `bit_offset`: Strap bit number (0-63)
    ///
    /// # Returns
    /// - `Ok(StrapStatus)`: Current status of the strap bit
    /// - `Err(Self::Error)`: Failed to get strap status
    ///
    /// # Errors
    /// - No active session
    /// - Invalid strap bit offset (> 63)
    /// - Hardware access failure
    fn get_strap_status(&self, bit_offset: u8) -> Result<StrapStatus, Self::Error>;

    /// Program a single strap bit
    ///
    /// # Arguments
    /// - `bit_offset`: Strap bit number (0-63)
    /// - `value`: Value to program (true or false)
    ///
    /// # Returns
    /// - `Ok(())`: Strap bit programmed successfully
    /// - `Err(Self::Error)`: Programming operation failed
    ///
    /// # Errors
    /// - No active session
    /// - Strap region is protected
    /// - Invalid strap bit offset
    /// - No remaining write attempts for this bit
    /// - Programming verification failed
    fn program_strap_bit(&mut self, bit_offset: u8, value: bool) -> Result<(), Self::Error>;

    /// Get total number of strap bits
    fn strap_bit_count(&self) -> u8 {
        64
    }
}

/// ASPEED OTP image programming operations
///
/// Supports programming complete OTP images containing data, configuration,
/// and strap settings with verification and validation.
pub trait AspeedOtpImage: AspeedOtpSession + AspeedOtpData + AspeedOtpConfig + AspeedOtpStrap {
    /// Program a complete OTP image
    ///
    /// This method programs all regions (data, config, strap) from a single
    /// image with proper verification and validation.
    ///
    /// # Arguments
    /// - `image_data`: Complete OTP image data
    ///
    /// # Returns
    /// - `Ok(())`: Image programmed successfully
    /// - `Err(Self::Error)`: Image programming failed
    ///
    /// # Errors
    /// - No active session
    /// - Invalid image format or checksum
    /// - One or more regions are protected
    /// - Programming verification failed
    fn program_image(&mut self, image_data: &[u8]) -> Result<(), Self::Error>;

    /// Validate an OTP image without programming
    ///
    /// # Arguments
    /// - `image_data`: OTP image data to validate
    ///
    /// # Returns
    /// - `Ok(())`: Image is valid and can be programmed
    /// - `Err(Self::Error)`: Image validation failed
    ///
    /// # Errors
    /// - Invalid image format
    /// - Checksum mismatch
    /// - Unsupported chip version in image
    /// - Image too large for available OTP space
    fn validate_image(&self, image_data: &[u8]) -> Result<(), Self::Error>;
}

/// ASPEED OTP security and protection operations
///
/// Provides advanced security features including protection control,
/// key management, and secure boot support.
pub trait AspeedOtpSecurity: AspeedOtpSession {
    /// Enable protection for a specific region
    ///
    /// # Arguments
    /// - `region`: Region to protect
    ///
    /// # Returns
    /// - `Ok(())`: Protection enabled successfully
    /// - `Err(Self::Error)`: Failed to enable protection
    ///
    /// # Errors
    /// - No active session
    /// - Invalid region
    /// - Hardware access failure
    /// - Region already protected
    fn enable_region_protection(&mut self, region: AspeedOtpRegion) -> Result<(), Self::Error>;

    /// Check if a region is protected
    ///
    /// # Arguments
    /// - `region`: Region to check
    ///
    /// # Returns
    /// - `Ok(bool)`: Protection status (true = protected)
    /// - `Err(Self::Error)`: Failed to check protection status
    fn is_region_protected(&self, region: AspeedOtpRegion) -> Result<bool, Self::Error>;

    /// Enable global memory lock (irreversible)
    ///
    /// This operation permanently locks all OTP regions and cannot be undone.
    /// Use with extreme caution.
    ///
    /// # Returns
    /// - `Ok(())`: Memory lock enabled successfully
    /// - `Err(Self::Error)`: Failed to enable memory lock
    ///
    /// # Errors
    /// - No active session
    /// - Hardware access failure
    /// - Memory already locked
    fn enable_memory_lock(&mut self) -> Result<(), Self::Error>;

    /// Check if memory is globally locked
    ///
    /// # Returns
    /// - `Ok(bool)`: Lock status (true = locked)
    /// - `Err(Self::Error)`: Failed to check lock status
    fn is_memory_locked(&self) -> Result<bool, Self::Error>;

    /// Get number of cryptographic keys stored
    ///
    /// # Returns
    /// - `Ok(u32)`: Number of keys stored in OTP
    /// - `Err(Self::Error)`: Failed to get key count
    fn get_key_count(&self) -> Result<u32, Self::Error>;

    /// Enable soak programming mode for difficult bits
    ///
    /// Soak mode uses extended programming timing for bits that are
    /// difficult to program with standard timing.
    ///
    /// # Arguments
    /// - `enable`: True to enable soak mode, false to disable
    ///
    /// # Returns
    /// - `Ok(())`: Soak mode setting applied successfully
    /// - `Err(Self::Error)`: Failed to set soak mode
    fn set_soak_mode(&mut self, enable: bool) -> Result<(), Self::Error>;
}

/// Full ASPEED OTP controller interface
///
/// This convenience trait combines all ASPEED OTP capabilities into a single
/// interface for devices that support all features.
pub trait AspeedOtpController:
    AspeedOtpSession + AspeedOtpData + AspeedOtpConfig + AspeedOtpStrap + AspeedOtpImage + AspeedOtpSecurity
{
    /// Get the detected chip version
    fn chip_version(&self) -> AspeedChipVersion;

    /// Get a human-readable version string
    fn version_string(&self) -> &str;

    /// Perform a complete OTP health check
    ///
    /// This method validates the OTP controller state, protection settings,
    /// and performs basic functionality tests.
    ///
    /// # Returns
    /// - `Ok(())`: OTP controller is healthy
    /// - `Err(Self::Error)`: Health check failed
    fn health_check(&mut self) -> Result<(), Self::Error>;
}

// Automatic implementation for types that implement all required traits
impl<T> AspeedOtpController for T
where
    T: AspeedOtpSession
        + AspeedOtpData
        + AspeedOtpConfig
        + AspeedOtpStrap
        + AspeedOtpImage
        + AspeedOtpSecurity,
{
    fn chip_version(&self) -> AspeedChipVersion {
        // Default implementation - should be overridden by implementors
        AspeedChipVersion::Unknown
    }

    fn version_string(&self) -> &str {
        // Default implementation - should be overridden by implementors
        "Unknown"
    }

    fn health_check(&mut self) -> Result<(), Self::Error> {
        // Default implementation performs basic checks
        // Implementors should override with comprehensive testing
        
        // Check if session can be established
        let _session_info = self.begin_session()?;
        
        // Check protection status
        let _protection = self.get_protection_status()?;
        
        // Check memory lock status
        let _locked = self.is_memory_locked()?;
        
        // End session
        self.end_session()?;
        
        Ok(())
    }
}

// Example controller for demonstration purposes (not in test module)
pub struct ExampleOtpController;

// Example error type
#[derive(Debug, Clone, Copy)]
pub struct ExampleError;

impl proposed_traits::otp::Error for ExampleError {
    fn kind(&self) -> proposed_traits::otp::ErrorKind {
        proposed_traits::otp::ErrorKind::WriteFailed
    }
}

impl From<ErrorKind> for ExampleError {
    fn from(_kind: ErrorKind) -> Self {
        ExampleError
    }
}

impl ErrorType for ExampleOtpController {
    type Error = ExampleError;
}

impl proposed_traits::otp::OtpSession for ExampleOtpController {
    type SessionInfo = SessionInfo;

    fn begin_session(&mut self) -> Result<SessionInfo, Self::Error> {
        Ok(SessionInfo {
            chip_version: AspeedChipVersion::Ast1060A1,
            version_name: *b"AST1060A1\0",
            protection_status: ProtectionStatus {
                memory_locked: false,
                key_protected: false,
                strap_protected: false,
                config_protected: false,
                data_protected: false,
                security_protected: false,
                security_size: 0,
            },
            tool_version: [0; 32],
            software_revision: 0x12345678,
            key_count: 5,
        })
    }

    fn end_session(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_session_active(&self) -> bool {
        true
    }
}

impl proposed_traits::otp::OtpRegions<u32> for ExampleOtpController {
    type Region = AspeedOtpRegion;

    fn read_region(&self, _region: Self::Region, _offset: usize, _buffer: &mut [u32]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write_region(&mut self, _region: Self::Region, _offset: usize, _data: &[u32]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn region_capacity(&self, region: Self::Region) -> usize {
        match region {
            AspeedOtpRegion::Data => 2048,
            AspeedOtpRegion::Configuration => 32,
            AspeedOtpRegion::Strap => 2,
            AspeedOtpRegion::ScuProtection => 2,
        }
    }

    fn region_alignment(&self, _region: Self::Region) -> usize {
        4
    }
}

impl AspeedOtpSession for ExampleOtpController {
    fn get_protection_status(&self) -> Result<ProtectionStatus, Self::Error> {
        Ok(ProtectionStatus {
            memory_locked: false,
            key_protected: false,
            strap_protected: false,
            config_protected: false,
            data_protected: false,
            security_protected: false,
            security_size: 0,
        })
    }
}

impl AspeedOtpData for ExampleOtpController {
    fn read_data(&self, _offset: u32, _buffer: &mut [u32]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn program_data(&mut self, _offset: u32, _data: &[u32]) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl AspeedOtpConfig for ExampleOtpController {
    fn read_config(&self, _offset: u32, _buffer: &mut [u32]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn program_config(&mut self, _offset: u32, _data: &[u32]) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl AspeedOtpStrap for ExampleOtpController {
    fn read_straps(&self, _buffer: &mut [u32; 2]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_strap_status(&self, _bit_offset: u8) -> Result<StrapStatus, Self::Error> {
        Ok(StrapStatus {
            value: false,
            options: [0; 7],
            remaining_writes: 3,
            writable_option: 1,
            protected: false,
        })
    }

    fn program_strap_bit(&mut self, _bit_offset: u8, _value: bool) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl AspeedOtpImage for ExampleOtpController {
    fn program_image(&mut self, _image_data: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_image(&self, _image_data: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl AspeedOtpSecurity for ExampleOtpController {
    fn enable_region_protection(&mut self, _region: AspeedOtpRegion) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_region_protected(&self, _region: AspeedOtpRegion) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn enable_memory_lock(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_memory_locked(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }

    fn get_key_count(&self) -> Result<u32, Self::Error> {
        Ok(5)
    }

    fn set_soak_mode(&mut self, _enable: bool) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proposed_traits::otp::ErrorType;

    // Mock error type for testing
    #[derive(Debug, Clone, Copy)]
    struct MockError;

    impl Error for MockError {
        fn kind(&self) -> ErrorKind {
            ErrorKind::WriteFailed
        }
    }

    // Mock OTP controller for testing
    struct MockOtpController;

    impl ErrorType for MockOtpController {
        type Error = MockError;
    }

    impl OtpSession for MockOtpController {
        type SessionInfo = SessionInfo;

        fn begin_session(&mut self) -> Result<SessionInfo, Self::Error> {
            Ok(SessionInfo {
                chip_version: AspeedChipVersion::Ast1060A1,
                version_name: *b"AST1060A1\0",
                protection_status: ProtectionStatus {
                    memory_locked: false,
                    key_protected: false,
                    strap_protected: false,
                    config_protected: false,
                    data_protected: false,
                    security_protected: false,
                    security_size: 0,
                },
                tool_version: [0; 32],
                software_revision: 0x12345678,
                key_count: 5,
            })
        }

        fn end_session(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn is_session_active(&self) -> bool {
            true
        }
    }

    impl AspeedOtpSession for MockOtpController {
        fn get_protection_status(&self) -> Result<ProtectionStatus, Self::Error> {
            Ok(ProtectionStatus {
                memory_locked: false,
                key_protected: false,
                strap_protected: false,
                config_protected: false,
                data_protected: false,
                security_protected: false,
                security_size: 0,
            })
        }
    }

    impl OtpRegions<u32> for MockOtpController {
        type Region = AspeedOtpRegion;

        fn read_region(&self, _region: Self::Region, _offset: usize, _buffer: &mut [u32]) -> Result<(), Self::Error> {
            Ok(())
        }

        fn write_region(&mut self, _region: Self::Region, _offset: usize, _data: &[u32]) -> Result<(), Self::Error> {
            Ok(())
        }

        fn region_capacity(&self, region: Self::Region) -> usize {
            match region {
                AspeedOtpRegion::Data => 2048,
                AspeedOtpRegion::Configuration => 32,
                AspeedOtpRegion::Strap => 2,
                AspeedOtpRegion::ScuProtection => 2,
            }
        }

        fn region_alignment(&self, _region: Self::Region) -> usize {
            4 // 4-byte alignment for u32
        }
    }

    impl AspeedOtpData for MockOtpController {
        fn read_data(&self, _offset: u32, _buffer: &mut [u32]) -> Result<(), Self::Error> {
            Ok(())
        }

        fn program_data(&mut self, _offset: u32, _data: &[u32]) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl AspeedOtpConfig for MockOtpController {
        fn read_config(&self, _offset: u32, _buffer: &mut [u32]) -> Result<(), Self::Error> {
            Ok(())
        }

        fn program_config(&mut self, _offset: u32, _data: &[u32]) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl AspeedOtpStrap for MockOtpController {
        fn read_straps(&self, _buffer: &mut [u32; 2]) -> Result<(), Self::Error> {
            Ok(())
        }

        fn get_strap_status(&self, _bit_offset: u8) -> Result<StrapStatus, Self::Error> {
            Ok(StrapStatus {
                value: false,
                options: [0; 7],
                remaining_writes: 3,
                writable_option: 1,
                protected: false,
            })
        }

        fn program_strap_bit(&mut self, _bit_offset: u8, _value: bool) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl AspeedOtpImage for MockOtpController {
        fn program_image(&mut self, _image_data: &[u8]) -> Result<(), Self::Error> {
            Ok(())
        }

        fn validate_image(&self, _image_data: &[u8]) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl AspeedOtpSecurity for MockOtpController {
        fn enable_region_protection(&mut self, _region: AspeedOtpRegion) -> Result<(), Self::Error> {
            Ok(())
        }

        fn is_region_protected(&self, _region: AspeedOtpRegion) -> Result<bool, Self::Error> {
            Ok(false)
        }

        fn enable_memory_lock(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn is_memory_locked(&self) -> Result<bool, Self::Error> {
            Ok(false)
        }

        fn get_key_count(&self) -> Result<u32, Self::Error> {
            Ok(5)
        }

        fn set_soak_mode(&mut self, _enable: bool) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn test_mock_controller_implements_full_interface() {
        let mut controller = MockOtpController;
        
        // Test that it implements AspeedOtpController
        assert_eq!(controller.chip_version(), AspeedChipVersion::Unknown);
        assert_eq!(controller.version_string(), "Unknown");
        
        // Test health check
        controller.health_check().unwrap();
    }

    #[test]
    fn test_session_info_creation() {
        let session_info = SessionInfo {
            chip_version: AspeedChipVersion::Ast1060A1,
            version_name: *b"AST1060A1\0",
            protection_status: ProtectionStatus {
                memory_locked: false,
                key_protected: true,
                strap_protected: false,
                config_protected: false,
                data_protected: false,
                security_protected: true,
                security_size: 1024,
            },
            tool_version: [0; 32],
            software_revision: 0x12345678,
            key_count: 10,
        };

        assert_eq!(session_info.chip_version, AspeedChipVersion::Ast1060A1);
        assert!(session_info.protection_status.key_protected);
        assert!(!session_info.protection_status.memory_locked);
        assert_eq!(session_info.key_count, 10);
    }

    #[test]
    fn test_region_types() {
        let regions = [
            AspeedOtpRegion::Data,
            AspeedOtpRegion::Configuration,
            AspeedOtpRegion::Strap,
            AspeedOtpRegion::ScuProtection,
        ];

        for region in &regions {
            // Test that regions can be compared
            assert_eq!(*region, *region);
        }
    }

    #[test]
    fn test_chip_versions() {
        let versions = [
            AspeedChipVersion::Ast1030A0,
            AspeedChipVersion::Ast1030A1,
            AspeedChipVersion::Ast1035A1,
            AspeedChipVersion::Ast1060A1,
            AspeedChipVersion::Ast1060A2,
            AspeedChipVersion::Unknown,
        ];

        for version in &versions {
            // Test that versions can be compared
            assert_eq!(*version, *version);
        }
    }
}

/// Application layer abstractions and usage examples for ASPEED OTP
///
/// This module demonstrates how the ASPEED OTP traits are typically used
/// in real-world applications, showing practical patterns and abstractions.
pub mod application_layer {
    use super::*;

    /// Device configuration data structure for manufacturing
    #[derive(Debug, Clone)]
    pub struct DeviceConfig {
        pub device_id: u32,
        pub serial_number: u64,
        pub mac_address: [u8; 6],
        pub calibration_data: [u32; 16],
        pub boot_mode: u8,
        pub feature_flags: u32,
    }

    impl DeviceConfig {
        pub fn to_words(&self) -> Vec<u32> {
            let mut words = Vec::new();
            words.push(self.device_id);
            words.push((self.serial_number >> 32) as u32);
            words.push(self.serial_number as u32);
            
            // Pack MAC address into a word
            let mac_word = ((self.mac_address[0] as u32) << 24)
                | ((self.mac_address[1] as u32) << 16)
                | ((self.mac_address[2] as u32) << 8)
                | (self.mac_address[3] as u32);
            words.push(mac_word);
            
            let mac_word2 = ((self.mac_address[4] as u32) << 8)
                | (self.mac_address[5] as u32);
            words.push(mac_word2);
            
            words.extend_from_slice(&self.calibration_data);
            words.push(((self.boot_mode as u32) << 24) | self.feature_flags);
            
            words
        }

        pub fn data_offset() -> u32 { 0 }
        pub fn data_size() -> usize { 22 } // words
    }

    /// Cryptographic keys for secure applications
    #[derive(Debug, Clone)]
    pub struct CryptoKeys {
        pub aes_key: [u32; 8],      // 256-bit AES key
        pub rsa_public_key: [u32; 64], // 2048-bit RSA public key
        pub ecdsa_key: [u32; 12],   // P-384 ECDSA key
        pub hmac_key: [u32; 8],     // HMAC key
    }

    impl CryptoKeys {
        pub fn to_words(&self) -> Vec<u32> {
            let mut words = Vec::new();
            words.extend_from_slice(&self.aes_key);
            words.extend_from_slice(&self.rsa_public_key);
            words.extend_from_slice(&self.ecdsa_key);
            words.extend_from_slice(&self.hmac_key);
            words
        }

        pub fn security_offset() -> u32 { 1024 } // Start at word 1024 in data region
        pub fn security_size() -> usize { 92 } // words
    }

    /// Manufacturing data combining all programming requirements
    #[derive(Debug, Clone)]
    pub struct ManufacturingData {
        pub config: DeviceConfig,
        pub keys: CryptoKeys,
        pub strap_settings: [bool; 64],
        pub hardware_config: [u32; 8],
    }

    /// High-level application service for ASPEED OTP operations
    pub trait AspeedOtpApplicationService {
        type Error;

        /// Store device configuration with verification
        fn store_device_config(&mut self, config: &DeviceConfig) -> Result<(), Self::Error>;

        /// Load device configuration
        fn load_device_config(&self) -> Result<DeviceConfig, Self::Error>;

        /// Store cryptographic keys securely with protection
        fn store_crypto_keys(&mut self, keys: &CryptoKeys) -> Result<(), Self::Error>;

        /// Program manufacturing data with high reliability
        fn program_manufacturing_data(&mut self, data: &ManufacturingData) -> Result<(), Self::Error>;

        /// Configure hardware straps for boot and pin settings
        fn configure_hardware_straps(&mut self, strap_settings: &[bool; 64]) -> Result<(), Self::Error>;

        /// Lock device for production use (irreversible)
        fn finalize_device_for_production(&mut self) -> Result<(), Self::Error>;

        /// Validate device programming integrity
        fn validate_device_integrity(&self) -> Result<ValidationReport, Self::Error>;

        /// Get device status and health information
        fn get_device_status(&self) -> Result<DeviceStatus, Self::Error>;
    }

    /// Device validation report
    #[derive(Debug, Clone)]
    pub struct ValidationReport {
        pub config_valid: bool,
        pub keys_valid: bool,
        pub straps_valid: bool,
        pub protection_enabled: bool,
        pub errors: Vec<String>,
        pub warnings: Vec<String>,
    }

    /// Device status information
    #[derive(Debug, Clone)]
    pub struct DeviceStatus {
        pub chip_version: AspeedChipVersion,
        pub memory_locked: bool,
        pub regions_protected: u8, // Bitmask of protected regions
        pub key_count: u32,
        pub programming_attempts: u32,
        pub health_score: u8, // 0-100
    }

    /// Implementation of the application service for any ASPEED OTP controller
    impl<T> AspeedOtpApplicationService for T
    where
        T: AspeedOtpController,
        T::Error: From<ErrorKind>,
    {
        type Error = T::Error;

        fn store_device_config(&mut self, config: &DeviceConfig) -> Result<(), Self::Error> {
            let _session = self.begin_session()?;
            
            // Check if config region is already protected
            if self.is_region_protected(AspeedOtpRegion::Configuration)? {
                return Err(ErrorKind::RegionProtected.into());
            }

            let config_words = config.to_words();
            self.program_config(0, &config_words)?;

            // Verify the configuration was written correctly
            let mut read_buffer = vec![0u32; config_words.len()];
            self.read_config(0, &mut read_buffer)?;
            
            if read_buffer != config_words {
                return Err(ErrorKind::VerificationFailed.into());
            }

            self.end_session()
        }

        fn load_device_config(&self) -> Result<DeviceConfig, Self::Error> {
            let mut buffer = vec![0u32; DeviceConfig::data_size()];
            self.read_config(0, &mut buffer)?;

            // Parse the configuration from words
            let device_id = buffer[0];
            let serial_number = ((buffer[1] as u64) << 32) | (buffer[2] as u64);
            
            let mac_word1 = buffer[3];
            let mac_word2 = buffer[4];
            let mac_address = [
                (mac_word1 >> 24) as u8,
                (mac_word1 >> 16) as u8,
                (mac_word1 >> 8) as u8,
                mac_word1 as u8,
                (mac_word2 >> 8) as u8,
                mac_word2 as u8,
            ];

            let mut calibration_data = [0u32; 16];
            calibration_data.copy_from_slice(&buffer[5..21]);

            let boot_and_features = buffer[21];
            let boot_mode = (boot_and_features >> 24) as u8;
            let feature_flags = boot_and_features & 0x00FFFFFF;

            Ok(DeviceConfig {
                device_id,
                serial_number,
                mac_address,
                calibration_data,
                boot_mode,
                feature_flags,
            })
        }

        fn store_crypto_keys(&mut self, keys: &CryptoKeys) -> Result<(), Self::Error> {
            let _session = self.begin_session()?;

            // Store keys in the secure part of the data region
            let key_words = keys.to_words();
            self.program_data(CryptoKeys::security_offset(), &key_words)?;

            // Use soak programming for critical security data
            self.set_soak_mode(true)?;
            
            // Verify keys were programmed correctly
            let mut read_buffer = vec![0u32; key_words.len()];
            self.read_data(CryptoKeys::security_offset(), &mut read_buffer)?;
            
            if read_buffer != key_words {
                return Err(T::Error::from(ErrorKind::VerificationFailed));
            }

            // Protect the data region containing keys
            self.enable_region_protection(AspeedOtpRegion::Data)?;

            self.set_soak_mode(false)?;
            self.end_session()
        }

        fn program_manufacturing_data(&mut self, data: &ManufacturingData) -> Result<(), Self::Error> {
            let _session = self.begin_session()?;

            // Enable soak mode for reliable manufacturing programming
            self.set_soak_mode(true)?;

            // 1. Program device configuration
            self.store_device_config(&data.config)?;

            // 2. Program cryptographic keys
            self.store_crypto_keys(&data.keys)?;

            // 3. Program hardware configuration
            self.program_config(24, &data.hardware_config)?;

            // 4. Configure hardware straps
            self.configure_hardware_straps(&data.strap_settings)?;

            self.set_soak_mode(false)?;
            self.end_session()
        }

        fn configure_hardware_straps(&mut self, strap_settings: &[bool; 64]) -> Result<(), Self::Error> {
            // Program each strap bit that needs to be set
            for (bit_index, &should_set) in strap_settings.iter().enumerate() {
                if should_set {
                    // Check if this strap bit has remaining writes
                    let status = self.get_strap_status(bit_index as u8)?;
                    if status.remaining_writes == 0 {
                        return Err(T::Error::from(ErrorKind::WriteExhausted));
                    }

                    self.program_strap_bit(bit_index as u8, true)?;
                }
            }

            Ok(())
        }

        fn finalize_device_for_production(&mut self) -> Result<(), Self::Error> {
            let _session = self.begin_session()?;

            // Protect all critical regions
            self.enable_region_protection(AspeedOtpRegion::Data)?;
            self.enable_region_protection(AspeedOtpRegion::Configuration)?;
            self.enable_region_protection(AspeedOtpRegion::Strap)?;

            // Enable global memory lock (irreversible)
            self.enable_memory_lock()?;

            self.end_session()
        }

        fn validate_device_integrity(&self) -> Result<ValidationReport, Self::Error> {
            let mut report = ValidationReport {
                config_valid: false,
                keys_valid: false,
                straps_valid: false,
                protection_enabled: false,
                errors: Vec::new(),
                warnings: Vec::new(),
            };

            // Validate configuration
            match self.load_device_config() {
                Ok(config) => {
                    report.config_valid = true;
                    if config.device_id == 0 {
                        report.warnings.push("Device ID is zero".to_string());
                    }
                }
                Err(_) => {
                    report.errors.push("Failed to load device configuration".to_string());
                }
            }

            // Check protection status
            match self.get_protection_status() {
                Ok(protection) => {
                    report.protection_enabled = protection.data_protected 
                        && protection.config_protected 
                        && protection.strap_protected;
                    
                    if !protection.memory_locked {
                        report.warnings.push("Memory not locked for production".to_string());
                    }
                }
                Err(_) => {
                    report.errors.push("Failed to check protection status".to_string());
                }
            }

            // Validate key storage
            let mut key_buffer = vec![0u32; CryptoKeys::security_size()];
            match self.read_data(CryptoKeys::security_offset(), &mut key_buffer) {
                Ok(_) => {
                    // Simple validation - check if keys are not all zeros
                    report.keys_valid = key_buffer.iter().any(|&word| word != 0);
                    if !report.keys_valid {
                        report.warnings.push("Security keys appear to be empty".to_string());
                    }
                }
                Err(_) => {
                    report.errors.push("Failed to read security keys".to_string());
                }
            }

            // Validate strap settings
            let mut strap_buffer = [0u32; 2];
            match self.read_straps(&mut strap_buffer) {
                Ok(_) => {
                    report.straps_valid = true;
                    // Check for common strap configuration issues
                    if strap_buffer[0] == 0 && strap_buffer[1] == 0 {
                        report.warnings.push("All strap bits are zero - check configuration".to_string());
                    }
                }
                Err(_) => {
                    report.errors.push("Failed to read strap settings".to_string());
                }
            }

            Ok(report)
        }

        fn get_device_status(&self) -> Result<DeviceStatus, Self::Error> {
            let protection_status = self.get_protection_status()?;
            let key_count = self.get_key_count()?;

            // Calculate protection bitmask
            let mut regions_protected = 0u8;
            if protection_status.data_protected { regions_protected |= 0x01; }
            if protection_status.config_protected { regions_protected |= 0x02; }
            if protection_status.strap_protected { regions_protected |= 0x04; }

            // Calculate health score based on various factors
            let mut health_score = 100u8;
            if !protection_status.memory_locked { health_score -= 20; }
            if regions_protected != 0x07 { health_score -= 15; }
            if key_count == 0 { health_score -= 25; }

            Ok(DeviceStatus {
                chip_version: self.chip_version(),
                memory_locked: protection_status.memory_locked,
                regions_protected,
                key_count,
                programming_attempts: 0, // Would be tracked by implementation
                health_score,
            })
        }
    }

    /// Manufacturing workflow example
    pub fn manufacturing_workflow<T>(
        controller: &mut T,
        manufacturing_data: ManufacturingData,
    ) -> Result<(), <T as ErrorType>::Error> 
    where
        T: AspeedOtpController + AspeedOtpApplicationService<Error = <T as ErrorType>::Error>,
        <T as ErrorType>::Error: From<ErrorKind>,
    {
        println!("Starting manufacturing workflow...");

        // Step 1: Validate the controller is ready
        println!("1. Validating OTP controller...");
        controller.health_check()?;

        // Step 2: Check if device is already programmed
        println!("2. Checking device programming status...");
        let status = controller.get_device_status()?;
        if status.memory_locked {
            return Err(ErrorKind::MemoryLocked.into());
        }

        // Step 3: Program manufacturing data
        println!("3. Programming manufacturing data...");
        controller.program_manufacturing_data(&manufacturing_data)?;

        // Step 4: Validate programming
        println!("4. Validating programming integrity...");
        let validation = controller.validate_device_integrity()?;
        if !validation.errors.is_empty() {
            println!("Validation errors: {:?}", validation.errors);
            return Err(ErrorKind::VerificationFailed.into());
        }

        // Step 5: Finalize for production
        println!("5. Finalizing device for production...");
        controller.finalize_device_for_production()?;

        // Step 6: Final verification
        println!("6. Final verification...");
        let final_status = controller.get_device_status()?;
        if !final_status.memory_locked {
            return Err(ErrorKind::LockFailed.into());
        }

        println!("Manufacturing workflow completed successfully!");
        println!("Device health score: {}/100", final_status.health_score);

        Ok(())
    }

    /// Secure application example - key provisioning service
    pub fn provision_security_keys<T>(
        controller: &mut T,
        _device_cert: &[u8],
        _root_ca: &[u8],
    ) -> Result<(), <T as ErrorType>::Error> 
    where
        T: AspeedOtpController + AspeedOtpApplicationService<Error = <T as ErrorType>::Error>,
        <T as ErrorType>::Error: From<ErrorKind>,
    {
        println!("Starting secure key provisioning...");

        // Generate or derive keys (simplified for example)
        let crypto_keys = CryptoKeys {
            aes_key: [0x12345678; 8], // In practice, this would be securely generated
            rsa_public_key: [0xDEADBEEF; 64],
            ecdsa_key: [0xCAFEBABE; 12],
            hmac_key: [0xFEEDFACE; 8],
        };

        // Store keys with maximum security
        controller.store_crypto_keys(&crypto_keys)?;

        // Verify key storage
        let validation = controller.validate_device_integrity()?;
        if !validation.keys_valid {
            return Err(ErrorKind::VerificationFailed.into());
        }

        println!("Security keys provisioned successfully");
        Ok(())
    }

    /// Device configuration service for field deployment
    pub fn configure_field_device<T>(
        controller: &mut T,
        network_config: &DeviceConfig,
    ) -> Result<(), <T as ErrorType>::Error> 
    where
        T: AspeedOtpController + AspeedOtpApplicationService<Error = <T as ErrorType>::Error>,
        <T as ErrorType>::Error: From<ErrorKind>,
    {
        // Check if device is already locked
        let status = controller.get_device_status()?;
        if status.memory_locked {
            println!("Device is production-locked, configuration cannot be changed");
            return Err(ErrorKind::MemoryLocked.into());
        }

        // Store field configuration
        controller.store_device_config(network_config)?;

        // Validate configuration
        let loaded_config = controller.load_device_config()?;
        if loaded_config.device_id != network_config.device_id {
            return Err(ErrorKind::VerificationFailed.into());
        }

        println!("Field device configured successfully");
        println!("Device ID: 0x{:08X}", loaded_config.device_id);
        println!("Serial Number: {}", loaded_config.serial_number);

        Ok(())
    }
}

/// Example demonstrating ASPEED OTP controller usage
fn main() {
    use application_layer::AspeedOtpApplicationService;
    
    println!("ASPEED OTP Controller Example");
    println!("============================");
    
    // Create an example ASPEED OTP controller
    let mut controller = ExampleOtpController;
    
    // Basic trait demonstrations
    println!("1. Establishing OTP session...");
    match controller.begin_session() {
        Ok(session_info) => {
            println!("   ✓ Session established successfully");
            println!("   Chip version: {:?}", session_info.chip_version);
            println!("   Software revision: 0x{:08X}", session_info.software_revision);
            println!("   Key count: {}", session_info.key_count);
        }
        Err(_) => println!("   ✗ Failed to establish session"),
    }
    
    // Demonstrate data operations
    println!("\n2. Testing data region operations...");
    match controller.read_data(0, &mut [0u32; 4]) {
        Ok(_) => println!("   ✓ Data read successful"),
        Err(_) => println!("   ✗ Data read failed"),
    }
    
    let test_data = [0x12345678, 0xDEADBEEF, 0xCAFEBABE, 0xFEEDFACE];
    match controller.program_data(0, &test_data) {
        Ok(_) => println!("   ✓ Data programming successful"),
        Err(_) => println!("   ✗ Data programming failed"),
    }
    
    // Demonstrate strap operations
    println!("\n3. Testing strap operations...");
    let mut strap_buffer = [0u32; 2];
    match controller.read_straps(&mut strap_buffer) {
        Ok(_) => println!("   ✓ Strap read successful"),
        Err(_) => println!("   ✗ Strap read failed"),
    }
    
    match controller.get_strap_status(5) {
        Ok(status) => {
            println!("   ✓ Strap bit 5 status:");
            println!("     Value: {}", status.value);
            println!("     Remaining writes: {}", status.remaining_writes);
            println!("     Protected: {}", status.protected);
        }
        Err(_) => println!("   ✗ Failed to get strap status"),
    }
    
    // Demonstrate soak programming
    println!("\n4. Testing soak programming...");
    match controller.set_soak_mode(true) {
        Ok(_) => println!("   ✓ Soak mode enabled"),
        Err(_) => println!("   ✗ Failed to enable soak mode"),
    }
    
    // Demonstrate protection features
    println!("\n5. Testing protection features...");
    match controller.get_protection_status() {
        Ok(status) => {
            println!("   ✓ Protection status:");
            println!("     Memory locked: {}", status.memory_locked);
            println!("     Data protected: {}", status.data_protected);
            println!("     Config protected: {}", status.config_protected);
            println!("     Strap protected: {}", status.strap_protected);
        }
        Err(_) => println!("   ✗ Failed to get protection status"),
    }
    
    // Demonstrate health check
    println!("\n6. Running health check...");
    match controller.health_check() {
        Ok(_) => println!("   ✓ Health check passed"),
        Err(_) => println!("   ✗ Health check failed"),
    }

    // Application Layer Demonstrations
    println!("\n");
    println!("APPLICATION LAYER DEMONSTRATIONS");
    println!("===============================");

    // Demonstrate device configuration
    println!("\n7. Device Configuration Example...");
    let device_config = application_layer::DeviceConfig {
        device_id: 0x12345678,
        serial_number: 0x1234567890ABCDEF,
        mac_address: [0x02, 0x42, 0xAC, 0x11, 0x00, 0x01],
        calibration_data: [0x1000; 16],
        boot_mode: 0x01,
        feature_flags: 0x00FF00FF,
    };

    match controller.store_device_config(&device_config) {
        Ok(_) => {
            println!("   ✓ Device configuration stored successfully");
            match controller.load_device_config() {
                Ok(loaded_config) => {
                    println!("   ✓ Configuration verified:");
                    println!("     Device ID: 0x{:08X}", loaded_config.device_id);
                    println!("     Serial: 0x{:016X}", loaded_config.serial_number);
                    println!("     MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", 
                             loaded_config.mac_address[0], loaded_config.mac_address[1],
                             loaded_config.mac_address[2], loaded_config.mac_address[3],
                             loaded_config.mac_address[4], loaded_config.mac_address[5]);
                }
                Err(_) => println!("   ✗ Failed to verify configuration"),
            }
        }
        Err(_) => println!("   ✗ Failed to store device configuration"),
    }

    // Demonstrate crypto key storage
    println!("\n8. Cryptographic Key Storage Example...");
    let crypto_keys = application_layer::CryptoKeys {
        aes_key: [0xDEADBEEF; 8],
        rsa_public_key: [0xCAFEBABE; 64],
        ecdsa_key: [0xFEEDFACE; 12],
        hmac_key: [0x12345678; 8],
    };

    match controller.store_crypto_keys(&crypto_keys) {
        Ok(_) => println!("   ✓ Cryptographic keys stored and protected"),
        Err(_) => println!("   ✗ Failed to store cryptographic keys"),
    }

    // Demonstrate device status reporting
    println!("\n9. Device Status and Health Reporting...");
    match controller.get_device_status() {
        Ok(status) => {
            println!("   ✓ Device status:");
            println!("     Chip version: {:?}", status.chip_version);
            println!("     Memory locked: {}", status.memory_locked);
            println!("     Protected regions: 0b{:03b}", status.regions_protected);
            println!("     Key count: {}", status.key_count);
            println!("     Health score: {}/100", status.health_score);
        }
        Err(_) => println!("   ✗ Failed to get device status"),
    }

    // Demonstrate integrity validation
    println!("\n10. Device Integrity Validation...");
    match controller.validate_device_integrity() {
        Ok(report) => {
            println!("   ✓ Validation report:");
            println!("     Config valid: {}", report.config_valid);
            println!("     Keys valid: {}", report.keys_valid);
            println!("     Straps valid: {}", report.straps_valid);
            println!("     Protection enabled: {}", report.protection_enabled);
            if !report.errors.is_empty() {
                println!("     Errors: {:?}", report.errors);
            }
            if !report.warnings.is_empty() {
                println!("     Warnings: {:?}", report.warnings);
            }
        }
        Err(_) => println!("   ✗ Failed to validate device integrity"),
    }

    // Demonstrate manufacturing workflow
    println!("\n11. Manufacturing Workflow Example...");
    let manufacturing_data = application_layer::ManufacturingData {
        config: device_config,
        keys: crypto_keys,
        strap_settings: {
            let mut straps = [false; 64];
            straps[0] = true;  // Boot from SPI
            straps[5] = true;  // Enable security features
            straps[12] = true; // Set clock configuration
            straps
        },
        hardware_config: [0x11111111; 8],
    };

    println!("   Manufacturing workflow would:");
    println!("   - Validate controller readiness");
    println!("   - Program device configuration");
    println!("   - Store cryptographic keys securely");
    println!("   - Configure hardware straps");
    println!("   - Enable protection mechanisms");
    println!("   - Perform final validation");
    println!("   - Lock device for production");

    // Demonstrate field configuration
    println!("\n12. Field Device Configuration Example...");
    let field_config = application_layer::DeviceConfig {
        device_id: 0x87654321,
        serial_number: 0xFEDCBA0987654321,
        mac_address: [0x02, 0x42, 0xAC, 0x11, 0x00, 0x02],
        calibration_data: [0x2000; 16],
        boot_mode: 0x02,
        feature_flags: 0xFF00FF00,
    };

    println!("   Field configuration would:");
    println!("   - Check device lock status");
    println!("   - Configure network parameters");
    println!("   - Set operational modes");
    println!("   - Validate configuration");

    // Clean up
    println!("\n13. Terminating session...");
    match controller.end_session() {
        Ok(_) => println!("   ✓ Session terminated successfully"),
        Err(_) => println!("   ✗ Failed to terminate session"),
    }
    
    println!("\nExample completed successfully!");
    println!("\nThis example demonstrates:");
    println!("BASIC TRAITS:");
    println!("- Session-based access control");
    println!("- Multi-region OTP operations (data, config, strap)");
    println!("- Strap bit programming with status tracking");
    println!("- Soak programming for difficult bits");
    println!("- Protection mechanisms and status queries");
    println!("- Health checking and error handling");
    
    println!("\nAPPLICATION LAYER:");
    println!("- Device configuration management");
    println!("- Secure cryptographic key storage");
    println!("- Manufacturing workflow automation");
    println!("- Field device configuration");
    println!("- Integrity validation and reporting");
    println!("- Status monitoring and health scoring");
    
    println!("\nThe ASPEED OTP traits extend the generic OTP traits");
    println!("to provide vendor-specific functionality while maintaining");
    println!("compatibility with generic OTP application code.");
    println!("\nApplication layer abstractions demonstrate how to build");
    println!("higher-level services on top of the composable trait system.");
}
