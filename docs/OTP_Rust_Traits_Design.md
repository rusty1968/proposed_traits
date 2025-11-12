# OTP Rust Traits Design Document

## Table of Contents
1. [Overview](#overview)
2. [Design Philosophy](#design-philosophy)
3. [Core Trait Architecture](#core-trait-architecture)
4. [Composable Trait System](#composable-trait-system)
5. [Error Handling](#error-handling)
6. [ASPEED Implementation Example](#aspeed-implementation-example)
7. [Application Layer Trait Usage](#application-layer-trait-usage)
8. [Trait Composition Patterns](#trait-composition-patterns)
9. [Implementation Guidelines](#implementation-guidelines)
10. [Future Extensions](#future-extensions)

## Overview

This document describes the actual OTP (One-Time Programmable) memory trait system implemented in this project. The design emphasizes **composability over hierarchy**, allowing implementations to opt into specific capabilities rather than implementing large, monolithic interfaces.

### Key Implementation Features
- **Composable trait design**: Small, focused traits that can be combined as needed
- **Zero-cost abstractions**: Traits compile to efficient code with no runtime overhead
- **Type safety**: Compile-time guarantees for memory safety and correctness
- **Hardware agnostic**: Core traits work across different OTP implementations
- **Optional capabilities**: Advanced features are opt-in through separate traits

## Design Philosophy

Our trait design differs from traditional hierarchical approaches by embracing **composition over inheritance**. Instead of requiring implementations to support all possible OTP features, we provide:

1. **Core functionality** through `OtpMemory<T>` - the basic read/write/lock interface
2. **Optional capabilities** through separate traits like `OtpSession`, `OtpRegions`, `OtpProtection`
3. **Specialized features** through traits like `OtpSoakProgramming`, `OtpWriteTracking`

This approach allows:
- Simple devices to implement only what they support
- Complex devices to compose multiple capabilities
- New features to be added without breaking existing implementations
- Clear separation of concerns

## Core Trait Architecture

### Base Trait: OtpMemory<T>

The foundation of our OTP system is the `OtpMemory<T>` trait, which provides basic read/write/lock functionality:

```rust
pub trait OtpMemory<T>: ErrorType
where
    T: Copy + Default,
{
    /// Reads a value of type `T` from the specified memory address.
    fn read(&self, address: usize) -> Result<T, Self::Error>;

    /// Writes a value of type `T` to the specified memory address.
    fn write(&mut self, address: usize, data: T) -> Result<(), Self::Error>;

    /// Permanently locks the OTP memory to prevent further writes.
    fn lock(&mut self) -> Result<(), Self::Error>;

    /// Checks whether the OTP memory is currently locked.
    fn is_locked(&self) -> bool;
}
```

### Error Handling Foundation

All traits extend `ErrorType` for consistent error handling:

```rust
pub trait ErrorType {
    type Error: Error;
}

pub trait Error: core::fmt::Debug {
    fn kind(&self) -> ErrorKind;
}
```

This allows each implementation to define its own error types while providing a common interface for error categorization.

## Composable Trait System

### Session Management

For devices requiring session-based access control:

```rust
pub trait OtpSession: ErrorType {
    type SessionInfo;

    /// Establish an OTP session with hardware access
    fn begin_session(&mut self) -> Result<Self::SessionInfo, Self::Error>;

    /// Terminate the OTP session and release resources
    fn end_session(&mut self) -> Result<(), Self::Error>;

    /// Check if a session is currently active
    fn is_session_active(&self) -> bool;
}
```

### Region-Based Access

For devices with multiple memory regions:

```rust
pub trait OtpRegions<T>: ErrorType
where
    T: Copy + Default,
{
    type Region: Copy + core::fmt::Debug + PartialEq;

    /// Read data from a specific OTP region
    fn read_region(&self, region: Self::Region, offset: usize, buffer: &mut [T]) -> Result<(), Self::Error>;

    /// Write data to a specific OTP region
    fn write_region(&mut self, region: Self::Region, offset: usize, data: &[T]) -> Result<(), Self::Error>;

    /// Get the capacity of a specific region
    fn region_capacity(&self, region: Self::Region) -> usize;

    /// Get the alignment requirement for a specific region
    fn region_alignment(&self, region: Self::Region) -> usize;
}
```

### Protection and Security

For devices with protection mechanisms:

```rust
pub trait OtpProtection: ErrorType {
    type Region: Copy + core::fmt::Debug + PartialEq;

    /// Check if a specific region is protected
    fn is_region_protected(&self, region: Self::Region) -> Result<bool, Self::Error>;

    /// Enable protection for a specific region
    fn enable_region_protection(&mut self, region: Self::Region) -> Result<(), Self::Error>;

    /// Check if the entire memory is globally locked
    fn is_globally_locked(&self) -> Result<bool, Self::Error>;

    /// Enable global memory lock (typically irreversible)
    fn enable_global_lock(&mut self) -> Result<(), Self::Error>;
}
```

### Soak Programming

For devices supporting extended programming techniques:

```rust
pub trait OtpSoakProgramming<T>: ErrorType
where
    T: Copy + Default,
{
    type SoakConfig: Copy + core::fmt::Debug;

    /// Program data using extended "soak" timing for difficult bits
    fn soak_program(&mut self, address: usize, data: &[T], config: Self::SoakConfig) -> Result<(), Self::Error>;

    /// Get the default soak programming configuration
    fn default_soak_config(&self) -> Self::SoakConfig;

    /// Check if soak programming is available for a specific address
    fn is_soak_available(&self, address: usize) -> Result<bool, Self::Error>;

    /// Program and verify data with automatic soak fallback
    fn program_with_soak_fallback(&mut self, address: usize, data: &[T], config: Self::SoakConfig) -> Result<(), Self::Error>;
}
```

### Additional Capability Traits

The system includes several other optional traits:

- **`OtpWriteTracking<T>`**: Track remaining write attempts for each location
- **`OtpVerification<T>`**: Enhanced verification and program-verify operations
- **`OtpIdentification`**: Chip version and feature detection
- **`OtpBulkOperations<T>`**: Optimized bulk read/write operations
- **`OtpMemoryLayout`**: Detailed memory organization information
- **`OtpMultiWidth`**: Support for multiple data word sizes
## Error Handling

### ErrorKind Enumeration

The system uses a comprehensive error categorization:

```rust
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ErrorKind {
    InvalidAddress,
    MemoryLocked,
    WriteFailed,
    ReadFailed,
    LockFailed,
    VerificationFailed,
    WriteExhausted,
    NoSession,
    RegionProtected,
    AlignmentError,
    BoundaryError,
    Timeout,
    Unknown,
}
```

### Enhanced Error Information

For implementations requiring detailed error context:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum OtpErrorInfo {
    Simple(ErrorKind),
    WithMessage(ErrorKind, &'static str),
    WithAddress(ErrorKind, usize),
    WithContext(ErrorKind, usize, &'static str),
}
```

## ASPEED Implementation Example

Our ASPEED-specific implementation demonstrates how to compose multiple traits for a complex OTP device:

```rust
pub struct AspeedOtp {
    registers: AspeedOtpRegisters,
    session_active: bool,
    data_region_info: RegionInfo,
    config_region_info: RegionInfo,
    strap_region_info: RegionInfo,
    protection_state: ProtectionState,
}

impl ErrorType for AspeedOtp {
    type Error = AspeedOtpError;
}

impl OtpMemory<u32> for AspeedOtp {
    // Basic read/write/lock operations
}

impl OtpSession for AspeedOtp {
    type SessionInfo = SessionInfo;
    // Session management implementation
}

impl OtpRegions<u32> for AspeedOtp {
    type Region = AspeedRegion;
    // Region-based access implementation
}

impl OtpProtection for AspeedOtp {
    type Region = AspeedRegion;
    // Protection and security implementation
}

impl OtpSoakProgramming<u32> for AspeedOtp {
    type SoakConfig = AspeedSoakConfig;
    // Soak programming implementation
}
```

### ASPEED-Specific Types

```rust
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AspeedRegion {
    Data,
    Config,
    Strap,
}

#[derive(Debug, Copy, Clone)]
pub struct AspeedSoakConfig {
    pub pulse_duration: u32,
    pub max_retries: u8,
    pub verify_margin: u32,
}

#[derive(Debug)]
pub enum AspeedOtpError {
    InvalidRegion,
    SessionNotActive,
    RegionProtected,
    SoakFailed,
    HardwareError,
}
```

## Trait Composition Patterns

### Simple Device Implementation

A minimal OTP device only needs to implement the core trait:

```rust
struct SimpleOtp {
    memory: [u8; 1024],
    locked: bool,
}

impl ErrorType for SimpleOtp {
    type Error = SimpleOtpError;
}

impl OtpMemory<u8> for SimpleOtp {
    // Basic functionality only
}
```

### Advanced Device Implementation

A complex device can compose multiple capabilities:

```rust
struct AdvancedOtp {
    // Hardware abstraction layer
    hal: HardwareLayer,
    // Session management
    session: SessionManager,
    // Region information
    regions: RegionManager,
    // Protection state
    protection: ProtectionManager,
}

// Implement multiple traits as needed
impl OtpMemory<u32> for AdvancedOtp { /* ... */ }
impl OtpSession for AdvancedOtp { /* ... */ }
impl OtpRegions<u32> for AdvancedOtp { /* ... */ }
impl OtpProtection for AdvancedOtp { /* ... */ }
impl OtpSoakProgramming<u32> for AdvancedOtp { /* ... */ }
impl OtpWriteTracking<u32> for AdvancedOtp { /* ... */ }
```

### Generic Programming with Bounds

Code can be written generically over specific trait combinations:

```rust
fn program_with_verification<T, D>(
    device: &mut D, 
    address: usize, 
    data: &[T]
) -> Result<(), D::Error>
where
    T: Copy + Default + PartialEq,
    D: OtpMemory<T> + OtpVerification<T>,
{
    device.program_and_verify(address, data)
}

fn secure_program_with_session<T, D>(
    device: &mut D,
    region: D::Region,
    offset: usize,
    data: &[T]
) -> Result<(), D::Error>
where
    T: Copy + Default,
    D: OtpMemory<T> + OtpSession + OtpRegions<T>,
{
    let _session = device.begin_session()?;
    device.write_region(region, offset, data)?;
    device.end_session()
}
## Implementation Guidelines

### 1. Start Simple, Add Complexity

Begin with just `OtpMemory<T>`:

```rust
struct MyOtpDevice {
    // Hardware abstraction
    registers: RegisterMap,
}

impl ErrorType for MyOtpDevice {
    type Error = MyOtpError;
}

impl OtpMemory<u32> for MyOtpDevice {
    fn read(&self, address: usize) -> Result<u32, Self::Error> {
        // Basic implementation
    }
    
    fn write(&mut self, address: usize, data: u32) -> Result<(), Self::Error> {
        // Basic implementation
    }
    
    fn lock(&mut self) -> Result<(), Self::Error> {
        // Basic implementation
    }
    
    fn is_locked(&self) -> bool {
        // Basic implementation
    }
}
```

### 2. Add Capabilities as Needed

When you need session management, add the trait:

```rust
impl OtpSession for MyOtpDevice {
    type SessionInfo = MySessionInfo;
    
    fn begin_session(&mut self) -> Result<Self::SessionInfo, Self::Error> {
        // Session implementation
    }
    
    // ... other session methods
}
```

### 3. Define Domain-Specific Error Types

Create meaningful error types for your domain:

```rust
#[derive(Debug)]
pub enum MyOtpError {
    HardwareTimeout,
    RegionMismatch { expected: Region, found: Region },
    ProtectionViolation { region: Region },
    SoakProgrammingFailed { attempts: u8 },
}

impl Error for MyOtpError {
    fn kind(&self) -> ErrorKind {
        match self {
            MyOtpError::HardwareTimeout => ErrorKind::Timeout,
            MyOtpError::RegionMismatch { .. } => ErrorKind::InvalidAddress,
            MyOtpError::ProtectionViolation { .. } => ErrorKind::RegionProtected,
            MyOtpError::SoakProgrammingFailed { .. } => ErrorKind::WriteFailed,
        }
    }
}
```

### 4. Use Type-Safe Region Identifiers

Define strong types for regions:

```rust
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MyRegion {
    UserData,
    Configuration,
    SecurityKeys,
    CalibrationData,
}

impl MyRegion {
    fn base_address(self) -> usize {
        match self {
            MyRegion::UserData => 0x0000,
            MyRegion::Configuration => 0x2000,
            MyRegion::SecurityKeys => 0x2100,
            MyRegion::CalibrationData => 0x2200,
        }
    }
    
    fn size(self) -> usize {
        match self {
            MyRegion::UserData => 0x2000,       // 8KB
            MyRegion::Configuration => 0x100,   // 256 bytes
            MyRegion::SecurityKeys => 0x100,    // 256 bytes
            MyRegion::CalibrationData => 0x100, // 256 bytes
        }
    }
}
```

### 5. Leverage the Type System

Use generic programming to write reusable code:

```rust
// Works with any device that supports basic operations
fn backup_data<T, D>(device: &D, address: usize, count: usize) -> Result<Vec<T>, D::Error>
where
    T: Copy + Default,
    D: OtpMemory<T>,
{
    let mut backup = Vec::with_capacity(count);
    for i in 0..count {
        backup.push(device.read(address + i)?);
    }
    Ok(backup)
}

// Works with any device that supports regions
fn verify_region<T, D>(
    device: &D, 
    region: D::Region, 
    expected: &[T]
) -> Result<bool, D::Error>
where
    T: Copy + Default + PartialEq,
    D: OtpRegions<T>,
{
    let mut buffer = vec![T::default(); expected.len()];
    device.read_region(region, 0, &mut buffer)?;
    Ok(buffer == expected)
}
```

## Future Extensions

The composable trait design allows for easy extension with new capabilities:

### Potential Future Traits

1. **`OtpTelemetry`**: Monitoring and statistics collection
2. **`OtpEncryption`**: Built-in encryption for sensitive data
3. **`OtpWearLeveling`**: Wear leveling algorithms for limited-write technologies
4. **`OtpCompression`**: Data compression before programming
5. **`OtpVersioning`**: Version control for OTP data
6. **`OtpBackup`**: Backup and restore capabilities
7. **`OtpMirroring`**: Redundant storage across multiple devices

### Extension Example

```rust
pub trait OtpTelemetry: ErrorType {
    type Metrics: core::fmt::Debug;
    
    /// Get current device metrics
    fn get_metrics(&self) -> Result<Self::Metrics, Self::Error>;
    
    /// Reset metrics counters
    fn reset_metrics(&mut self) -> Result<(), Self::Error>;
    
    /// Get historical usage data
    fn get_usage_history(&self) -> Result<Vec<UsageEntry>, Self::Error>;
}

pub struct UsageEntry {
    pub timestamp: u64,
    pub operation: Operation,
    pub address: usize,
    pub success: bool,
}
```

This trait could be added to existing implementations without breaking changes, demonstrating the flexibility of the composable approach.

### Backward Compatibility

The non-exhaustive error enumeration and composable trait design ensure that:

1. New error kinds can be added without breaking existing code
2. New traits can be added without requiring existing implementations to change
3. Existing generic code continues to work with enhanced implementations
4. Migration paths exist for adding new capabilities gradually

This design provides a solid foundation for OTP memory abstractions that can evolve with changing requirements while maintaining type safety and performance.
---

## Summary

This document describes the actual composable OTP trait system implemented in this project. The key advantages of this approach are:

### Benefits of Composable Design

1. **Incremental Implementation**: Start with basic functionality and add features as needed
2. **Clear Separation of Concerns**: Each trait has a single, well-defined responsibility
3. **Type Safety**: Compile-time guarantees prevent common programming errors
4. **Flexibility**: Implementations can choose which capabilities to support
5. **Extensibility**: New traits can be added without breaking existing code
6. **Zero-Cost Abstractions**: Traits compile to efficient code with no runtime overhead

### Real-World Usage

The ASPEED implementation demonstrates how these traits compose in practice:

```rust
// Simple usage - just basic OTP operations
let mut simple_device = SimpleOtp::new();
simple_device.write(0, 0x12345678)?;
let value = simple_device.read(0)?;

// Advanced usage - full feature set
let mut aspeed = AspeedOtp::new(base_address);
let session = aspeed.begin_session()?;
aspeed.write_region(AspeedRegion::Data, 0, &data)?;
aspeed.enable_region_protection(AspeedRegion::Data)?;
aspeed.soak_program(address, &difficult_data, soak_config)?;
aspeed.end_session()?;
```

This design provides a solid foundation for OTP memory abstractions that can evolve with changing requirements while maintaining type safety, performance, and developer ergonomics.

/// Stream-based OTP operations for large data
pub trait OtpStream {
    type Item;
    type Error;
    
    /// Create a stream for reading large amounts of data
    fn read_stream(&mut self, start_addr: u32, len: u32) -> impl Stream<Item = Result<Self::Item, Self::Error>>;
    
    /// Create a sink for writing large amounts of data
    fn write_sink(&mut self, start_addr: u32) -> impl Sink<Self::Item, Error = Self::Error>;
}
```

### Compile-Time Configuration

```rust
// src/const_generic.rs

/// Compile-time configured OTP device
pub struct TypedOtp<const DATA_SIZE: usize, const CONFIG_SIZE: usize, const STRAP_BITS: usize> {
    // Implementation
}

impl<const DATA_SIZE: usize, const CONFIG_SIZE: usize, const STRAP_BITS: usize> 
    TypedOtp<DATA_SIZE, CONFIG_SIZE, STRAP_BITS> 
{
    /// Compile-time address validation
    pub const fn validate_address(addr: usize) -> bool {
        addr < DATA_SIZE
    }
    
    /// Type-safe read with compile-time bounds checking
    pub fn read_checked<const ADDR: usize>(&mut self) -> OtpResult<u32> 
    where
        Assert<{ADDR < DATA_SIZE}>: True
    {
        self.read(ADDR as u32)
    }
}

/// Compile-time assertion helper
pub struct Assert<const CHECK: bool>;
pub trait True {}
impl True for Assert<true> {}

/// Type aliases for specific ASPEED variants
pub type Ast1030Otp = TypedOtp<2048, 32, 64>;
pub type Ast1060Otp = TypedOtp<2048, 32, 64>;
```

## Conclusion

This Rust traits design provides a comprehensive, type-safe, and performant abstraction for OTP memory management. The trait-based architecture offers:

- **Compile-time safety**: Rust's type system prevents common hardware programming errors
- **Zero-cost abstractions**: Traits compile away to efficient machine code
- **Composability**: Traits can be mixed and matched for different use cases  
- **Testability**: Mock implementations and property-based testing support
- **Extensibility**: Easy to add new chip variants and features
- **Memory safety**: Ownership system prevents dangling pointers and data races

The design follows Rust best practices while providing the flexibility needed for embedded systems programming and hardware abstraction.

## Application Layer Trait Usage

In practice, application code typically uses **combinations of traits** rather than single traits, depending on the specific requirements. Here are the common patterns for application layer usage:

### Primary Application Layer Traits

Most applications would use these fundamental trait combinations:

```rust
// Basic OTP operations - almost always needed
OtpMemory<T>           // Read, write, lock, is_locked

// Session management - for secure/controlled access
OtpSession             // begin_session, end_session, is_session_active

// Region-based access - for organized memory layout
OtpRegions<T>          // read_region, write_region, region_capacity
```

### Application Usage Patterns

#### Pattern 1: Simple Configuration Storage
```rust
fn store_device_config<D>(device: &mut D, config: &DeviceConfig) -> Result<(), D::Error>
where
    D: OtpMemory<u32> + OtpSession,
{
    let _session = device.begin_session()?;
    device.write(CONFIG_ADDRESS, config.to_u32())?;
    device.end_session()
}
```

#### Pattern 2: Secure Key Storage
```rust
fn store_security_keys<D>(
    device: &mut D, 
    keys: &SecurityKeys
) -> Result<(), D::Error>
where
    D: OtpMemory<u32> + OtpSession + OtpRegions<u32> + OtpProtection + OtpVerification<u32>,
{
    let _session = device.begin_session()?;
    
    // Store keys in security region
    device.write_region(Region::Security, 0, &keys.data)?;
    
    // Verify the keys were written correctly
    device.verify(0, &keys.data)?;
    
    // Protect the security region
    device.enable_region_protection(Region::Security)?;
    
    device.end_session()
}
```

#### Pattern 3: Manufacturing Data Programming
```rust
fn program_manufacturing_data<D>(
    device: &mut D,
    calibration: &CalibrationData,
    serial: &SerialNumber,
) -> Result<(), D::Error>
where
    D: OtpMemory<u32> + OtpSession + OtpRegions<u32> + OtpSoakProgramming<u32>,
{
    let _session = device.begin_session()?;
    
    // Use soak programming for critical calibration data
    let soak_config = device.default_soak_config();
    device.soak_program(calibration.address(), &calibration.data, soak_config)?;
    
    // Regular programming for serial number
    device.write_region(Region::Data, serial.offset(), &serial.data)?;
    
    device.end_session()
}
```

### High-Level Application Abstractions

Applications often create higher-level abstractions that combine multiple traits:

```rust
/// High-level OTP service for applications
pub trait OtpApplicationService {
    type Error;
    
    /// Store device configuration with verification
    fn store_config(&mut self, config: &DeviceConfig) -> Result<(), Self::Error>;
    
    /// Load device configuration
    fn load_config(&self) -> Result<DeviceConfig, Self::Error>;
    
    /// Store cryptographic keys securely
    fn store_keys(&mut self, keys: &CryptoKeys) -> Result<(), Self::Error>;
    
    /// Program manufacturing data with high reliability
    fn program_manufacturing(&mut self, data: &ManufacturingData) -> Result<(), Self::Error>;
    
    /// Lock device for production use
    fn finalize_device(&mut self) -> Result<(), Self::Error>;
}
```

This can be implemented for any device that supports the required trait combination:

```rust
impl<D> OtpApplicationService for D
where
    D: OtpMemory<u32> 
      + OtpSession 
      + OtpRegions<u32> 
      + OtpProtection 
      + OtpSoakProgramming<u32> 
      + OtpVerification<u32>,
{
    type Error = D::Error;
    
    fn store_config(&mut self, config: &DeviceConfig) -> Result<(), Self::Error> {
        let _session = self.begin_session()?;
        self.write_region(Region::Config, 0, &config.data)?;
        self.verify(0, &config.data)?;
        self.end_session()
    }
    
    fn store_keys(&mut self, keys: &CryptoKeys) -> Result<(), Self::Error> {
        let _session = self.begin_session()?;
        self.write_region(Region::Security, 0, &keys.data)?;
        self.verify(keys.address(), &keys.data)?;
        self.enable_region_protection(Region::Security)?;
        self.end_session()
    }
    
    fn program_manufacturing(&mut self, data: &ManufacturingData) -> Result<(), Self::Error> {
        let _session = self.begin_session()?;
        let soak_config = self.default_soak_config();
        self.soak_program(data.address(), &data.calibration, soak_config)?;
        self.write_region(Region::Data, data.serial_offset(), &data.serial)?;
        self.end_session()
    }
    
    fn finalize_device(&mut self) -> Result<(), Self::Error> {
        self.enable_global_lock()
    }
    
    fn load_config(&self) -> Result<DeviceConfig, Self::Error> {
        let raw_data = self.read(CONFIG_ADDRESS)?;
        Ok(DeviceConfig::from_u32(raw_data))
    }
}
```

### Framework Integration

The traits also work well with application frameworks and services:

```rust
// Web service endpoint example
async fn program_device_endpoint(
    device: &mut dyn OtpApplicationService,
    request: ProgrammingRequest,
) -> Result<ProgrammingResponse, ServiceError> {
    device.store_config(&request.config).await?;
    device.store_keys(&request.keys).await?;
    device.program_manufacturing(&request.manufacturing).await?;
    
    Ok(ProgrammingResponse::success())
}

// Command-line tool example
fn cli_program_device<D>(
    device: &mut D,
    config_file: &str,
    keys_file: &str,
) -> Result<(), Box<dyn std::error::Error>>
where
    D: OtpApplicationService,
{
    let config = DeviceConfig::load_from_file(config_file)?;
    let keys = CryptoKeys::load_from_file(keys_file)?;
    
    device.store_config(&config)?;
    device.store_keys(&keys)?;
    device.finalize_device()?;
    
    println!("Device programmed successfully");
    Ok(())
}
```

### Trait Selection Guidelines for Applications

| Application Type | Required Traits | Optional Traits | Use Case |
|-----------------|----------------|-----------------|----------|
| **Simple Config Storage** | `OtpMemory<T>` | `OtpSession` | Basic device configuration |
| **Secure Applications** | `OtpMemory<T>`, `OtpProtection` | `OtpSession`, `OtpRegions<T>` | Security keys, certificates |
| **Manufacturing** | `OtpMemory<T>`, `OtpSession` | `OtpSoakProgramming<T>`, `OtpWriteTracking<T>` | Production programming |
| **High-Reliability** | `OtpMemory<T>`, `OtpVerification<T>` | `OtpSoakProgramming<T>`, `OtpWriteTracking<T>` | Critical data storage |
| **Complex Systems** | `OtpMemory<T>`, `OtpRegions<T>` | All optional traits | Full-featured applications |

The composable design allows applications to use exactly the traits they need, keeping interfaces clean and focused while supporting everything from simple configuration storage to complex, high-security programming workflows.
