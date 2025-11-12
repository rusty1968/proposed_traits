use core::fmt::Debug;

/// Represents a valid block address type.
pub trait BlockAddress: PartialEq + Debug + Copy + Clone {}

/// Represents a range of blocks starting at `start` and spanning `count` blocks.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BlockRange<A> {
    pub start: A,
    pub count: usize,
}

/// Common error kinds for block device operations.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    ReadError,
    ProgramError,
    EraseError,
    OutOfBounds,
}

/// Trait for converting implementation-specific errors into a generic [`ErrorKind`].
pub trait Error: Debug {
    /// Returns a generic error kind corresponding to the specific error.
    fn kind(&self) -> ErrorKind;
}

impl Error for core::convert::Infallible {
    fn kind(&self) -> ErrorKind {
        match *self {}
    }
}

/// A trait that BlockDevice implementations can use to share an error type.
pub trait ErrorType {
    /// Errors returned by this block device.
    type Error: Error;
}

/// Trait representing a block device abstraction that operates in units of blocks.
pub trait BlockDevice: ErrorType {
    /// The type used to represent block addresses.
    type Address: BlockAddress;

    /// Returns the size of a readable block in bytes.
    fn read_size(&self) -> usize;

    /// Reads data starting at the given block address.
    ///
    /// # Parameters
    /// - `address`: The block address to start reading from.
    /// - `data`: The buffer to store the read data.
    ///
    /// # Returns
    /// A result indicating success or failure.
    fn read(&mut self, address: Self::Address, data: &mut [u8]) -> Result<(), Self::Error>;

    /// Returns the size of an erasable block in bytes.
    fn erase_size(&self) -> usize;

    /// Erases a range of blocks on the device.
    ///
    /// # Parameters
    /// - `range`: The range of blocks to erase.
    ///
    /// # Returns
    /// A result indicating success or failure.
    fn erase(&mut self, range: BlockRange<Self::Address>) -> Result<(), Self::Error>;

    /// Returns the size of a programmable block in bytes.
    fn program_size(&self) -> usize;

    /// Programs data starting at the given block address.
    ///
    /// # Parameters
    /// - `address`: The block address to start programming at.
    /// - `data`: The data to program.
    ///
    /// # Returns
    /// A result indicating success or failure.
    fn program(&mut self, address: Self::Address, data: &[u8]) -> Result<(), Self::Error>;

    /// Returns the total capacity of the device in bytes.
    fn capacity(&self) -> usize;
}

/// Optional trait for block devices that support trimming.
pub trait TrimDevice: ErrorType {
    /// The type used to represent block addresses.
    type Address: BlockAddress;

    /// Trims a range of blocks on the device.
    ///
    /// # Parameters
    /// - `range`: The range of blocks to trim.
    ///
    /// # Returns
    /// A result indicating success or failure.
    fn trim(&mut self, range: BlockRange<Self::Address>) -> Result<(), Self::Error>;
}

/// Optional trait for block devices that support locking.
pub trait LockableDevice: ErrorType {
    /// The type used to represent block addresses.
    type Address: BlockAddress;

    /// Locks a range of blocks on the device.
    ///
    /// # Parameters
    /// - `range`: The range of blocks to lock.
    ///
    /// # Returns
    /// A result indicating success or failure.
    fn lock(&mut self, range: BlockRange<Self::Address>) -> Result<(), Self::Error>;

    /// Unlocks a range of blocks on the device.
    ///
    /// # Parameters
    /// - `range`: The range of blocks to unlock.
    ///
    /// # Returns
    /// A result indicating success or failure.
    fn unlock(&mut self, range: BlockRange<Self::Address>) -> Result<(), Self::Error>;
}

/// Optional trait for block devices that support wear leveling.
pub trait WearLevelDevice: ErrorType {
    /// The type used to represent block addresses.
    type Address: BlockAddress;

    /// Performs wear leveling on a range of blocks on the device.
    ///
    /// # Parameters
    /// - `range`: The range of blocks to wear level.
    ///
    /// # Returns
    /// A result indicating success or failure.
    fn wear_level(&mut self, range: BlockRange<Self::Address>) -> Result<(), Self::Error>;
}

// Implement BlockAddress for common address types
impl BlockAddress for u32 {}
impl BlockAddress for u64 {}
impl BlockAddress for usize {}
