use crate::i2c_target::I2CCoreTarget;

/// A convenience trait alias for I3C targets that support all advanced features,
/// including I2C fallback, dynamic addressing, and In-Band Interrupts (IBI).
pub trait I3CTarget: DynamicAddressable + IBICapable {}

impl<T> I3CTarget for T where T: DynamicAddressable + IBICapable {}

/// Core trait for I3C targets that builds on I2C target behavior.
///
/// This trait serves as the foundation for I3C-specific extensions and assumes
/// the device supports basic I2C target functionality.
pub trait I3CCoreTarget: I2CCoreTarget {}

/// Trait for I3C targets that support dynamic address assignment (DAA).
///
/// This method is invoked during the Dynamic Address Assignment (DAA) process,
/// where the I3C controller assigns a unique dynamic address to the target device.
/// The target should store this address and use it for all subsequent communications
/// on the I3C bus.
///
/// # Typical Responsibilities
///
/// - **Update Internal State**: Store the new dynamic address internally for future use.
/// - **Transition to I3C Mode**: If the device was in I2C fallback mode, it may now
///   enable I3C-specific features such as In-Band Interrupts (IBIs) or HDR modes.
/// - **Enable I3C Features**: Prepare to respond to I3C broadcast commands, group addressing,
///   and other advanced features.
/// - **Notify Subsystems**: Optionally inform internal components that the device is now
///   addressable and fully operational.
/// - **Reject Further DAA**: After assignment, the target typically ignores or rejects
///   additional DAA requests unless reset.
///
/// # Example
///
/// ```rust,ignore
/// fn on_dynamic_address_assigned(&mut self, new_address: u8) {
///     self.dynamic_address = Some(new_address);
///     self.enable_i3c_features();
/// }
/// ```
///
/// # Note
///
/// This method is only relevant for I3C targets that support dynamic addressing.
/// Static-only targets may not implement this behavior.
pub trait DynamicAddressable: I3CCoreTarget {
    fn on_dynamic_address_assigned(&mut self, new_address: u8);
}

/// Trait for I3C targets that support In-Band Interrupts (IBI).
///
/// This trait defines the methods required for a target to participate in IBI communication.
pub trait IBICapable: I3CCoreTarget {
    /// Checks whether the target wants to initiate an In-Band Interrupt (IBI).
    ///
    /// This method is used to determine if the target device is requesting to send an
    /// In-Band Interrupt (IBI) to the controller. It is typically polled by the controller
    /// or evaluated by the bus logic to decide whether to initiate an IBI sequence.
    ///
    /// # Returns
    ///
    /// * `true` — if the target has an IBI pending and wishes to notify the controller.
    /// * `false` — if no IBI is currently requested.
    ///
    /// # Typical Use Cases
    ///
    /// - The target has new data available (e.g., a sensor reading).
    /// - An internal event or threshold condition has been triggered.
    /// - The target needs to asynchronously alert the controller.
    ///
    /// # Note
    ///
    /// This method should be lightweight and non-blocking. It is only relevant for
    /// targets that support IBI functionality. If `true` is returned, the controller
    /// may proceed to acknowledge the IBI and request the payload using `get_ibi_payload`.
    fn wants_ibi(&self) -> bool;

    /// Retrieves the payload data to be sent with an In-Band Interrupt (IBI).
    ///
    /// This method is called by the I3C controller after it acknowledges an IBI request
    /// from the target. The target should fill the provided buffer with the IBI payload
    /// data and return the number of bytes written.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A mutable slice where the IBI payload should be written. The controller
    ///   will use this buffer to read the data that the target wants to send as part of the IBI.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - The number of bytes written to the buffer, indicating the size of the IBI payload.
    /// * `Err(Self::Error)` - If the payload is not ready or an error occurs during preparation.
    ///
    /// # Typical Use Cases
    ///
    /// - Sending sensor readings or alert codes.
    /// - Providing event metadata or diagnostic information.
    /// - Transmitting compact status updates or flags.
    ///
    /// # Notes
    ///
    /// - The maximum payload size is typically limited (e.g., 1–3 bytes), depending on the controller's
    ///   configuration and the I3C specification.
    /// - This method is only relevant for targets that support IBI with data payloads.
    /// - It should be safe to call this method only after `wants_ibi()` has returned `true`.
    fn get_ibi_payload(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;

    /// Called after the In-Band Interrupt (IBI) has been acknowledged by the controller.
    ///
    /// This method is invoked once the I3C controller has accepted and processed the IBI
    /// request initiated by the target. It allows the target to perform any necessary
    /// post-IBI actions, such as clearing internal flags, resetting interrupt state,
    /// or preparing for the next IBI event.
    ///
    /// # Typical Use Cases
    ///
    /// - Clear internal IBI request flags.
    /// - Reset or update internal state machines.
    /// - Log or track that the IBI was successfully delivered.
    /// - Prepare for future IBI events.
    ///
    /// # Note
    ///
    /// This method is only relevant for targets that support IBI functionality.
    /// It is typically called after the controller has completed the IBI data phase
    /// and acknowledged the interrupt.
    fn on_ibi_acknowledged(&mut self);
}
