use embedded_hal::i2c::ErrorType as I2CErrorType;

/// Direction of an I2C transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionDirection {
    /// Write transaction (controller writing to target).
    Write,
    /// Read transaction (controller reading from target).
    Read,
}

/// A convenience trait alias that represents a fully-featured I2C target device.
///
/// This trait combines all the core and extended I2C target traits into a single interface:
///
/// - [`I2CCoreTarget`]: Handles transaction lifecycle and address matching.
/// - [`ReadTarget`]: Supports reading data from the target.
/// - [`WriteTarget`]: Supports writing data to the target.
/// - [`WriteReadTarget`]: Supports combined write-read transactions.
/// - [`RegisterAccess`]: Supports register-level read/write operations.
///
/// Implementing this trait means the device is capable of handling all standard I2C target behaviors,
/// making it suitable for use in generic drivers or frameworks that require full I2C functionality.
///
/// # Example
/// ```rust,ignore
/// struct MyDevice { /* ... */ }
///
/// impl I2CCoreTarget for MyDevice { /* ... */ }
/// impl ReadTarget for MyDevice { /* ... */ }
/// impl WriteTarget for MyDevice { /* ... */ }
/// impl WriteReadTarget for MyDevice { /* ... */ }
/// impl RegisterAccess for MyDevice { /* ... */ }
///
/// // Now MyDevice automatically implements FullI2CTarget
/// fn use_device<T: I2CTarget>(dev: &mut T) {
///     // Use all I2C capabilities
/// }
/// ```
pub trait I2CTarget:
    I2CCoreTarget + ReadTarget + WriteTarget + WriteReadTarget + RegisterAccess
{
}

impl<T> I2CTarget for T where
    T: I2CCoreTarget + ReadTarget + WriteTarget + WriteReadTarget + RegisterAccess
{
}

/// Trait representing a target (slave) I2C device behavior.
///
/// This trait defines the core methods that an I2C target device must implement to handle
/// transactions initiated by an I2C master. It includes methods for handling stop conditions,
/// transaction starts, and address match events.
pub trait I2CCoreTarget: I2CErrorType {
    /// Initialize the target with a specific address.
    fn init(&mut self, address: u8) -> Result<(), Self::Error>;

    /// Called when a new I2C transaction begins.
    ///
    /// This method is invoked at the start of a transaction initiated by the controller (master).
    /// It provides a flag indicating whether the transaction is a repeated start, which allows
    /// the target (slave) device to preserve or reset internal state accordingly.
    ///
    /// # Arguments
    ///
    /// * `direction` - Indicates whether the upcoming transaction is a Read or Write.
    /// * `repeated` - A boolean flag indicating whether this transaction is a repeated start (`true`)
    ///                or a fresh start (`false`). A repeated start means the controller has not
    ///                released the bus between transactions.
    ///
    /// # Return
    ///
    /// * `Ok(Some(data))` - If `direction` is `Read`, return a byte to immediately send to the controller.
    /// * `Ok(None)` - No immediate data to send, or this is a write transaction.
    /// * `Err(e)` - An error occurred during setup.
    /// # Usage Model
    ///
    /// This method is distinct from `on_address_match(address: u8) -> bool`:
    ///
    /// - `on_address_match` is called during the address phase to decide whether the target should respond.
    /// - `on_transaction_start` is called after the address is matched and before the data phase begins.
    ///
    /// ## Typical Use Cases:
    /// - Reset internal state if `repeated == false`.
    /// - Preserve context (e.g., register pointer) if `repeated == true`.
    /// - Prepare buffers or state machines for read/write.
    ///
    /// ## When it's called:
    /// - After `on_address_match` returns `true`.
    /// - Before `on_read` or `on_write` is invoked.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// fn on_transaction_start(&mut self, direction: TransactionDirection, repeated: bool) {
    ///     if repeated {
    ///         // Continue using previous state
    ///     } else {
    ///         // Reset internal state
    ///     }
    /// }
    /// ```
    fn on_transaction_start(
        &mut self,
        direction: TransactionDirection,
        repeated: bool,
    ) -> Result<Option<u8>, Self::Error>;

    /// Optional: handle stop condition or reset.
    ///
    /// This method is called when the master sends a stop condition, indicating the end of a transaction.
    fn on_stop(&mut self);

    /// Optional: handle address match event.
    ///
    /// # Arguments
    ///
    /// * `address` - The address sent by the master.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the address matches the target's address, `false` otherwise.
    fn on_address_match(&mut self, address: u8) -> bool;
}

/// Trait for I2C targets that support write operations.
pub trait WriteTarget: I2CCoreTarget {
    /// Called when the master initiates a write to this target.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice containing the data to be written to the target.
    ///
    /// # Returns
    ///
    /// * `Result<(), I2CError>` - Returns `Ok(())` if the write is successful, or an `I2CError` if an error occurs.
    fn on_write(&mut self, data: &[u8]) -> Result<(), Self::Error>;
}

/// Trait for I2C targets that support read operations.
pub trait ReadTarget: I2CCoreTarget {
    /// Called when the master initiates a read from this target.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A mutable slice where the read data will be stored.
    ///
    /// # Returns
    ///
    /// * `Result<usize, I2CError>` - Returns the number of bytes read if successful, or an `I2CError` if an error occurs.
    fn on_read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
}

/// Trait for I2C targets that support combined write-read transactions.
pub trait WriteReadTarget: WriteTarget + ReadTarget {
    /// Performs a combined write-read transaction on the device.
    ///
    /// This method writes data from `write_buffer` and then reads data into `read_buffer`
    /// in a single, atomic operation (if supported by the underlying hardware).
    ///
    /// # Parameters
    /// - `write_buffer`: The buffer containing data to write.
    /// - `read_buffer`: The buffer to store the data read from the device.
    ///
    /// # Returns
    /// - `Ok(usize)`: The number of bytes read into `read_buffer`.
    /// - `Err(Self::Error)`: If the transaction fails.
    ///
    /// # Errors
    /// This function returns an error if the write or read operation fails.
    ///
    /// # Example
    /// ```ignore
    /// device.on_write_read(&mut [0x01, 0x02], &mut [0; 4])?;
    /// ```
    fn on_write_read(
        &mut self,
        write_buffer: &mut [u8],
        read_buffer: &mut [u8],
    ) -> Result<usize, Self::Error> {
        self.on_write(write_buffer)?;
        self.on_read(read_buffer)
    }
}

/// Trait for I2C targets that support register-based access.
pub trait RegisterAccess: WriteTarget + ReadTarget {
    fn write_register(&mut self, address: u8, data: u8) -> Result<(), Self::Error>;
    fn read_register(&mut self, address: u8, buffer: &mut [u8]) -> Result<usize, Self::Error>;
}
