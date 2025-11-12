//! # Design Considerations
//!
//! This module defines traits and error types for abstracting system-level
//! clock and reset control operations. The design emphasizes **flexibility**,
//! **composability**, and **extensibility** to support a wide range of hardware
//! platforms and use cases.
//!
//! ## Flexibility
//!
//! - **Platform Independence**: The traits abstract hardware-specific operations,
//!   allowing implementations to target different platforms without modifying
//!   consumer code.
//! - **Custom Identifiers and Configurations**: Associated types like `ClockId`,
//!   `ClockConfig`, and `ResetId` allow implementers to define identifiers and
//!   configurations that match their hardware model.
//! - **Error Abstraction**: The `Error` trait and `ErrorKind` enum decouple
//!   error handling from specific implementations, enabling generic code to
//!   respond to common failure modes while supporting detailed diagnostics.
//!
//! ## Composability
//!
//! - **Unified Error Handling**: The `ErrorType` supertrait provides a consistent
//!   interface for accessing error types, enabling integration with other traits
//!   and systems.
//! - **Non-Exhaustive ErrorKind**: The `#[non_exhaustive]` attribute on `ErrorKind`
//!   allows future expansion without breaking existing code, supporting long-term
//!   composability.
//!
//! ## Extensibility
//!
//! - **Custom Error Types**: Implementers can define their own error types and
//!   map them to `ErrorKind`, enabling rich, context-specific error reporting
//!   while maintaining compatibility with generic consumers.
//! - **Vendor-Specific Configuration**: The `ClockConfig` type supports complex
//!   configuration structures such as PLL settings, dividers, or source selectors.
//! - **Partial Implementations**: While all trait methods are required, implementers
//!   can provide no-op or stub implementations for unsupported features, enabling
//!   partial functionality where appropriate.
//!
//! ## Summary
//!
//! This design provides a robust foundation for building portable, maintainable,
//! and scalable hardware abstraction layers. By leveraging Rust’s trait system,
//! it enables:
//!
//! - Reuse of generic drivers across platforms.
//! - Simplified testing and mocking.
//! - Clear contracts between hardware abstraction and higher-level logic.

use core::time::Duration;

///
/// This represents a common set of syste, control  operation errors. Implementations are
/// free to define more specific or additional error types. However, by providing
/// a mapping to these common errors, generic code can still react to them.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    ClockNotFound,
    ClockAlreadyEnabled,
    ClockAlreadyDisabled,
    InvalidClockFrequency,
    ClockConfigurationFailed,
    InvalidResetId,
    HardwareFailure,
    PermissionDenied,
    Timeout,
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

/// Trait for clock control operations.
/// Abstracts enabling, disabling, and configuring clocks for peripherals or system components.
pub trait ClockControl: ErrorType {
    /// Type for identifying a clock (e.g., peripheral ID, clock name, or register offset).
    type ClockId: Clone + PartialEq;
    /// Type for configuring a clock.
    type ClockConfig: PartialEq;

    /// Enables a clock for the specified clock ID.
    ///
    /// # Arguments
    ///
    /// * `clock_id` - A reference to the identifier of the clock to enable.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Ok if the operation is successful, or an error of type `Self::Error`.
    fn enable(&mut self, clock_id: &Self::ClockId) -> Result<(), Self::Error>;

    /// Disables a clock for the specified clock ID.
    ///
    /// # Arguments
    ///
    /// * `clock_id` - A reference to the identifier of the clock to disable.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Ok if the operation is successful, or an error of type `Self::Error`.
    fn disable(&mut self, clock_id: &Self::ClockId) -> Result<(), Self::Error>;

    /// Sets the frequency of a clock (in Hz).
    ///
    /// # Arguments
    ///
    /// * `clock_id` - A reference to the identifier of the clock to set the frequency for.
    /// * `frequency_hz` - The frequency to set, in Hertz.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Ok if the operation is successful, or an error of type `Self::Error`.
    fn set_frequency(
        &mut self,
        clock_id: &Self::ClockId,
        frequency_hz: u64,
    ) -> Result<(), Self::Error>;

    /// Gets the current frequency of a clock (in Hz).
    ///
    /// # Arguments
    ///
    /// * `clock_id` - A reference to the identifier of the clock to get the frequency for.
    ///
    /// # Returns
    ///
    /// * `Result<u64, Self::Error>` - Ok with the current frequency in Hertz, or an error of type `Self::Error`.
    fn get_frequency(&self, clock_id: &Self::ClockId) -> Result<u64, Self::Error>;

    /// Configures clock-specific parameters (e.g., divider, source).
    /// Vendor-specific parameters can be passed via `ClockConfig`.
    ///
    /// # Arguments
    ///
    /// * `clock_id` - A reference to the identifier of the clock to configure.
    /// * `config` - The configuration parameters for the clock.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Ok if the operation is successful, or an error of type `Self::Error`.
    fn configure(
        &mut self,
        clock_id: &Self::ClockId,
        config: Self::ClockConfig,
    ) -> Result<(), Self::Error>;

    /// Retrieves the current configuration of a clock.
    ///
    /// # Arguments
    ///
    /// * `clock_id` - A reference to the identifier of the clock to get the configuration for.
    ///
    /// # Returns
    ///
    /// * `Result<Self::ClockConfig, Self::Error>` - Ok with the current configuration, or an error of type `Self::Error`.
    fn get_config(&self, clock_id: &Self::ClockId) -> Result<Self::ClockConfig, Self::Error>;
}

/// Trait for reset control operations.
/// Abstracts asserting and deasserting reset signals for peripherals or system components.
pub trait ResetControl: ErrorType {
    /// Type for identifying a reset line (e.g., peripheral ID, reset name, or register offset).
    type ResetId: Clone + PartialEq;

    /// Asserts the reset signal for the specified reset ID (holds the component in reset).
    ///
    /// # Arguments
    ///
    /// * `reset_id` - A reference to the identifier of the reset line to assert.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Ok if the operation is successful, or an error of type `Self::Error`.
    fn reset_assert(&mut self, reset_id: &Self::ResetId) -> Result<(), Self::Error>;

    /// Deasserts the reset signal for the specified reset ID (releases the component from reset).
    ///
    /// # Arguments
    ///
    /// * `reset_id` - A reference to the identifier of the reset line to deassert.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Ok if the operation is successful, or an error of type `Self::Error`.
    fn reset_deassert(&mut self, reset_id: &Self::ResetId) -> Result<(), Self::Error>;

    /// Performs a pulse reset (asserts then deasserts) with a specified duration (in microseconds).
    ///
    /// # Arguments
    ///
    /// * `reset_id` - A reference to the identifier of the reset line to pulse.
    /// * `duration_us` - The duration of the pulse in microseconds.
    ///
    /// # Returns
    ///
    /// * `Result<(), Self::Error>` - Ok if the operation is successful, or an error of type `Self::Error`.
    fn reset_pulse(
        &mut self,
        reset_id: &Self::ResetId,
        duration: Duration,
    ) -> Result<(), Self::Error>;

    /// Checks if the reset signal is currently asserted for the specified reset ID.
    ///
    /// # Arguments
    ///
    /// * `reset_id` - A reference to the identifier of the reset line to check.
    ///
    /// # Returns
    ///
    /// * `Result<bool, Self::Error>` - Ok with a boolean indicating if the reset is asserted, or an error of type `Self::Error`.
    fn reset_is_asserted(&self, reset_id: &Self::ResetId) -> Result<bool, Self::Error>;
}
