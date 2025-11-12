use embedded_hal::i2c::SevenBitAddress;

/// Represents errors that can occur during I3C communication.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// A Common Command Code (CCC) was malformed or not supported.
    InvalidCcc,

    /// A conflict or failure occurred during dynamic address assignment.
    DynamicAddressConflict,

    /// An in-band interrupt (IBI) was not acknowledged or was malformed.
    IbiError,

    /// A device attempted to join the bus dynamically but failed protocol checks.
    HotJoinError,

    /// A violation occurred in High Data Rate (HDR) mode, such as framing or timing issues.
    HdrModeViolation,

    /// A reserved or illegal broadcast address (e.g., 0x7E) was used incorrectly.
    InvalidBroadcastAddress,

    /// A catch-all for other or vendor-specific errors.
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

pub trait ErrorType {
    /// Error type.
    type Error: Error;
}

/// Represents the supported I3C bus speed modes.
#[derive(Debug, Clone, Copy)]
pub enum I3cSpeed {
    /// Standard Data Rate (SDR) mode.
    SDR,
    /// High Data Rate (HDR) mode.
    HDR,
    /// High Data Rate Double Data Rate (HDR-DDR) mode.
    HDRDdr,
}

/// A trait for I3C master devices that isintended to be used alongside  the embedded-hal I2C trait.
///
/// This trait adds I3C-specific capabilities such as dynamic address assignment,
/// in-band interrupt handling, hot-join support, and high-speed mode configuration.
pub trait I3c: ErrorType {
    /// Assigns a dynamic address to a device with a known static address.
    ///
    /// This method initiates the Dynamic Address Assignment (DAA) process as defined by the I3C specification.
    /// Devices initially join the I3C bus with a static address (or no address), and the master assigns them
    /// a unique dynamic address for subsequent communication.
    ///
    /// # Parameters
    ///
    /// * `static_address` - The 7-bit static address of the device requesting a dynamic address.
    ///   This must be a valid `SevenBitAddress`, as I3C only supports 7-bit addressing.
    ///
    /// # Returns
    ///
    /// * `Ok(dynamic_address)` - The newly assigned dynamic address, also a `SevenBitAddress`.
    /// * `Err(Self::Error)` - An error occurred during the assignment process, such as:
    ///   - Address conflict
    ///   - Protocol violation
    ///   - Bus arbitration failure
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let static_addr = SevenBitAddress::new(0x52).unwrap();
    /// match controller.assign_dynamic_address(static_addr) {
    ///     Ok(dynamic_addr) => println!("Assigned dynamic address: {:?}", dynamic_addr),
    ///     Err(e) => eprintln!("Failed to assign dynamic address: {:?}", e),
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - This method is specific to I3C and has no equivalent in I2C.
    /// - It is typically used during bus initialization or when handling hot-join events.
    fn assign_dynamic_address(
        &mut self,
        static_address: SevenBitAddress,
    ) -> Result<SevenBitAddress, Self::Error>;

    /// Acknowledges an in-band interrupt (IBI) from a device.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the device that issued the IBI.
    fn acknowledge_ibi(&mut self, address: SevenBitAddress) -> Result<(), Self::Error>;

    /// Handles a hot-join request from a device joining the bus dynamically.
    ///
    /// In an I3C bus system, devices can dynamically join the bus after it has already been initialized and is operational.
    /// This is known as a **hot-join**. Unlike I2C, which typically requires all devices to be present at initialization,
    /// I3C allows for more flexible and dynamic device management.
    ///
    /// This method is invoked by the I3C master when a new device signals its intent to join the bus. The master must then:
    /// - Detect the hot-join request.
    /// - Validate the new device’s presence and protocol compliance.
    /// - Assign a dynamic address (typically via `assign_dynamic_address`).
    /// - Update internal routing or device tables.
    ///
    /// # Typical flow
    /// 1. A device asserts a hot-join request on the bus.
    /// 2. The I3C master detects this and calls `handle_hot_join()`.
    /// 3. The method performs protocol-level checks and initiates dynamic address assignment.
    /// 4. If successful, the device becomes an active participant on the bus.
    ///
    /// # Errors
    /// Returns `Err(Self::Error)` if an error occurs during the hot-join process, such as:
    /// - Address conflict
    /// - Protocol violation
    /// - Bus arbitration failure
    ///
    fn handle_hot_join(&mut self) -> Result<(), Self::Error>;

    /// Sets the bus speed mode.
    ///
    /// # Arguments
    ///
    /// * `speed` - The desired I3C speed mode.
    fn set_bus_speed(&mut self, speed: I3cSpeed) -> Result<(), Self::Error>;

    /// Requests mastership of the bus in a multi-master environment.
    fn request_mastership(&mut self) -> Result<(), Self::Error>;
}
