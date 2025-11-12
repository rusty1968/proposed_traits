//! Example: SPI NOR Flash Block Device Adapter
//!
//! This example demonstrates how to create an adapter that implements the 
//! `BlockDevice` trait for SPI NOR flash devices. The adapter bridges between
//! the generic block device interface and SPI NOR specific operations.

use core::fmt::Debug;
use proposed_traits::block_device::{
    BlockDevice, BlockRange, ErrorType, Error, ErrorKind,
};

/// SPI NOR device configuration data
#[derive(Debug, Clone, Copy)]
pub struct SpiNorData {
    pub sector_size: u32,
    pub page_size: u32,
    pub capacity: u32,
    pub jedec_id: [u8; 3],
}

/// SPI NOR specific device interface
///
/// This trait represents the low-level SPI NOR flash operations that would
/// typically be implemented by a hardware driver or HAL.
pub trait SpiNorDevice {
    type Error: Debug;

    fn nor_read_init(&mut self, data: &SpiNorData) -> Result<(), Self::Error>;
    fn nor_write_init(&mut self, data: &SpiNorData) -> Result<(), Self::Error>;
    fn nor_write_enable(&mut self) -> Result<(), Self::Error>;
    fn nor_write_disable(&mut self) -> Result<(), Self::Error>;
    fn nor_read_jedec_id(&mut self) -> Result<[u8; 3], Self::Error>;
    fn nor_sector_erase(&mut self, address: u32) -> Result<(), Self::Error>;
    fn nor_page_program(&mut self, address: u32, data: &[u8]) -> Result<(), Self::Error>;
    fn nor_page_program_4b(&mut self, address: u32, data: &[u8]) -> Result<(), Self::Error>;
    fn nor_read_data(&mut self, address: u32, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn nor_read_fast_4b_data(&mut self, address: u32, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn nor_sector_aligned(&mut self, address: u32) -> bool;
    fn nor_wait_until_ready(&mut self);
    fn nor_reset(&mut self) -> Result<(), Self::Error>;
    fn nor_reset_enable(&mut self) -> Result<(), Self::Error>;
}

/// Error type for SPI NOR adapter operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiNorAdapterError<E> {
    /// Underlying SPI NOR device error
    DeviceError(E),
    /// Invalid block address
    InvalidAddress,
    /// Invalid block size
    InvalidSize,
    /// Device not initialized
    NotInitialized,
    /// Alignment error
    AlignmentError,
    /// JEDEC ID mismatch
    JedecIdMismatch,
}

impl<E: Debug> Error for SpiNorAdapterError<E> {
    fn kind(&self) -> ErrorKind {
        match self {
            SpiNorAdapterError::DeviceError(_) => ErrorKind::ReadError,
            SpiNorAdapterError::InvalidAddress => ErrorKind::OutOfBounds,
            SpiNorAdapterError::InvalidSize => ErrorKind::OutOfBounds,
            SpiNorAdapterError::NotInitialized => ErrorKind::ReadError,
            SpiNorAdapterError::AlignmentError => ErrorKind::OutOfBounds,
            SpiNorAdapterError::JedecIdMismatch => ErrorKind::ReadError,
        }
    }
}

/// Adapter that implements BlockDevice for SPI NOR devices
///
/// This adapter translates between block-based operations (used by filesystems
/// and higher-level abstractions) and the SPI NOR specific operations.
pub struct SpiNorBlockAdapter<D> {
    device: D,
    config: Option<SpiNorData>,
    initialized: bool,
}

impl<D> SpiNorBlockAdapter<D>
where
    D: SpiNorDevice,
{
    /// Create a new SPI NOR block adapter
    pub fn new(device: D) -> Self {
        Self {
            device,
            config: None,
            initialized: false,
        }
    }

    /// Initialize the adapter with SPI NOR configuration
    ///
    /// This performs the complete initialization sequence:
    /// 1. Reset the device
    /// 2. Initialize read and write operations
    /// 3. Verify the JEDEC ID matches the expected configuration
    pub fn initialize(&mut self, config: SpiNorData) -> Result<(), SpiNorAdapterError<D::Error>> {
        // Reset the device
        self.device.nor_reset_enable().map_err(SpiNorAdapterError::DeviceError)?;
        self.device.nor_reset().map_err(SpiNorAdapterError::DeviceError)?;
        self.device.nor_wait_until_ready();

        // Initialize for reading and writing
        self.device.nor_read_init(&config).map_err(SpiNorAdapterError::DeviceError)?;
        self.device.nor_write_init(&config).map_err(SpiNorAdapterError::DeviceError)?;

        // Verify JEDEC ID matches expected configuration
        let jedec_id = self.device.nor_read_jedec_id().map_err(SpiNorAdapterError::DeviceError)?;
        if jedec_id != config.jedec_id {
            return Err(SpiNorAdapterError::JedecIdMismatch);
        }

        self.config = Some(config);
        self.initialized = true;
        Ok(())
    }

    /// Get the configuration data
    pub fn config(&self) -> Option<&SpiNorData> {
        self.config.as_ref()
    }

    /// Convert block address to byte address
    ///
    /// Block addresses are sector-aligned, so we multiply by sector size
    fn block_to_byte_address(&self, block_addr: u32) -> u32 {
        if let Some(config) = &self.config {
            block_addr * config.sector_size
        } else {
            block_addr * 4096 // Default sector size
        }
    }

    /// Convert byte address to block address
    ///
    /// This rounds down to the nearest sector boundary
    fn byte_to_block_address(&self, byte_addr: u32) -> u32 {
        if let Some(config) = &self.config {
            byte_addr / config.sector_size
        } else {
            byte_addr / 4096 // Default sector size
        }
    }

    /// Validate that an address range is within device bounds
    fn validate_address_range(&self, address: u32, length: usize) -> Result<(), SpiNorAdapterError<D::Error>> {
        if let Some(config) = &self.config {
            let byte_address = self.block_to_byte_address(address);
            if byte_address + length as u32 > config.capacity {
                return Err(SpiNorAdapterError::InvalidAddress);
            }
        }
        Ok(())
    }
}

impl<D> ErrorType for SpiNorBlockAdapter<D>
where
    D: SpiNorDevice,
{
    type Error = SpiNorAdapterError<D::Error>;
}

impl<D> BlockDevice for SpiNorBlockAdapter<D>
where
    D: SpiNorDevice,
{
    type Address = u32;

    fn read_size(&self) -> usize {
        // SPI NOR can typically read any amount, but we return page size
        // as the optimal read unit
        if let Some(config) = &self.config {
            config.page_size as usize
        } else {
            256 // Default page size
        }
    }

    fn read(&mut self, address: Self::Address, data: &mut [u8]) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(SpiNorAdapterError::NotInitialized);
        }

        let byte_address = self.block_to_byte_address(address);
        self.validate_address_range(address, data.len())?;

        // Use 4-byte addressing for better compatibility with larger devices
        self.device
            .nor_read_fast_4b_data(byte_address, data)
            .map_err(SpiNorAdapterError::DeviceError)
    }

    fn erase_size(&self) -> usize {
        // SPI NOR erases in sectors
        if let Some(config) = &self.config {
            config.sector_size as usize
        } else {
            4096 // Default sector size (4KB)
        }
    }

    fn erase(&mut self, range: BlockRange<Self::Address>) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(SpiNorAdapterError::NotInitialized);
        }

        // Validate the range
        self.validate_address_range(range.start, range.count * self.erase_size())?;

        // Enable writing for erase operations
        self.device.nor_write_enable().map_err(SpiNorAdapterError::DeviceError)?;

        // Erase each sector in the range
        for i in 0..range.count {
            let block_addr = range.start + i as u32;
            let byte_addr = self.block_to_byte_address(block_addr);

            // Verify sector alignment
            if !self.device.nor_sector_aligned(byte_addr) {
                return Err(SpiNorAdapterError::AlignmentError);
            }

            // Erase the sector
            self.device
                .nor_sector_erase(byte_addr)
                .map_err(SpiNorAdapterError::DeviceError)?;

            // Wait for erase to complete
            self.device.nor_wait_until_ready();
        }

        // Disable writing after erase completion
        self.device.nor_write_disable().map_err(SpiNorAdapterError::DeviceError)?;

        Ok(())
    }

    fn program_size(&self) -> usize {
        // SPI NOR programs in pages
        if let Some(config) = &self.config {
            config.page_size as usize
        } else {
            256 // Default page size
        }
    }

    fn program(&mut self, address: Self::Address, data: &[u8]) -> Result<(), Self::Error> {
        if !self.initialized {
            return Err(SpiNorAdapterError::NotInitialized);
        }

        let config = self.config.as_ref().ok_or(SpiNorAdapterError::NotInitialized)?;
        let byte_address = self.block_to_byte_address(address);
        
        self.validate_address_range(address, data.len())?;

        // Enable writing for program operations
        self.device.nor_write_enable().map_err(SpiNorAdapterError::DeviceError)?;

        // Program data in page-sized chunks
        let page_size = config.page_size as usize;
        for (chunk_idx, chunk) in data.chunks(page_size).enumerate() {
            let chunk_addr = byte_address + (chunk_idx * page_size) as u32;

            // Use 4-byte addressing for better compatibility
            self.device
                .nor_page_program_4b(chunk_addr, chunk)
                .map_err(SpiNorAdapterError::DeviceError)?;

            // Wait for programming to complete
            self.device.nor_wait_until_ready();
        }

        // Disable writing after programming completion
        self.device.nor_write_disable().map_err(SpiNorAdapterError::DeviceError)?;

        Ok(())
    }

    fn capacity(&self) -> usize {
        if let Some(config) = &self.config {
            config.capacity as usize
        } else {
            0
        }
    }
}

/// Common SPI NOR flash device information based on JEDEC ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceInfo {
    pub manufacturer: &'static str,
    pub device_name: &'static str,
    pub capacity: u32,
    pub sector_size: u32,
    pub page_size: u32,
    pub block_size: u32,
}

/// JEDEC ID database for common SPI NOR flash devices
pub struct JedecDatabase;

impl JedecDatabase {
    /// Look up device information from JEDEC ID
    ///
    /// JEDEC ID format: [manufacturer_id, memory_type, capacity_code]
    /// This function demonstrates how to derive device geometry from the standardized JEDEC ID.
    pub fn lookup_device(jedec_id: [u8; 3]) -> Option<DeviceInfo> {
        match jedec_id {
            // Winbond devices
            [0xEF, 0x40, 0x14] => Some(DeviceInfo {
                manufacturer: "Winbond",
                device_name: "W25Q80DV",
                capacity: 1 * 1024 * 1024, // 1MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0xEF, 0x40, 0x15] => Some(DeviceInfo {
                manufacturer: "Winbond",
                device_name: "W25Q16DV",
                capacity: 2 * 1024 * 1024, // 2MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0xEF, 0x40, 0x16] => Some(DeviceInfo {
                manufacturer: "Winbond",
                device_name: "W25Q32FV",
                capacity: 4 * 1024 * 1024, // 4MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0xEF, 0x40, 0x17] => Some(DeviceInfo {
                manufacturer: "Winbond",
                device_name: "W25Q64FV",
                capacity: 8 * 1024 * 1024, // 8MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0xEF, 0x40, 0x18] => Some(DeviceInfo {
                manufacturer: "Winbond",
                device_name: "W25Q128FV",
                capacity: 16 * 1024 * 1024, // 16MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0xEF, 0x40, 0x19] => Some(DeviceInfo {
                manufacturer: "Winbond",
                device_name: "W25Q256FV",
                capacity: 32 * 1024 * 1024, // 32MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),

            // Macronix devices
            [0xC2, 0x20, 0x14] => Some(DeviceInfo {
                manufacturer: "Macronix",
                device_name: "MX25L8005",
                capacity: 1 * 1024 * 1024, // 1MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0xC2, 0x20, 0x15] => Some(DeviceInfo {
                manufacturer: "Macronix",
                device_name: "MX25L1605D",
                capacity: 2 * 1024 * 1024, // 2MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0xC2, 0x20, 0x16] => Some(DeviceInfo {
                manufacturer: "Macronix",
                device_name: "MX25L3205D",
                capacity: 4 * 1024 * 1024, // 4MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0xC2, 0x20, 0x17] => Some(DeviceInfo {
                manufacturer: "Macronix",
                device_name: "MX25L6405D",
                capacity: 8 * 1024 * 1024, // 8MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0xC2, 0x20, 0x18] => Some(DeviceInfo {
                manufacturer: "Macronix",
                device_name: "MX25L12805D",
                capacity: 16 * 1024 * 1024, // 16MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),

            // Micron/ST devices
            [0x20, 0xBA, 0x16] => Some(DeviceInfo {
                manufacturer: "Micron",
                device_name: "N25Q032A",
                capacity: 4 * 1024 * 1024, // 4MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0x20, 0xBA, 0x17] => Some(DeviceInfo {
                manufacturer: "Micron",
                device_name: "N25Q064A",
                capacity: 8 * 1024 * 1024, // 8MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0x20, 0xBA, 0x18] => Some(DeviceInfo {
                manufacturer: "Micron",
                device_name: "N25Q128A",
                capacity: 16 * 1024 * 1024, // 16MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),

            // Spansion/Cypress devices
            [0x01, 0x02, 0x16] => Some(DeviceInfo {
                manufacturer: "Spansion",
                device_name: "S25FL032P",
                capacity: 4 * 1024 * 1024, // 4MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0x01, 0x02, 0x17] => Some(DeviceInfo {
                manufacturer: "Spansion",
                device_name: "S25FL064P",
                capacity: 8 * 1024 * 1024, // 8MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0x01, 0x02, 0x18] => Some(DeviceInfo {
                manufacturer: "Spansion",
                device_name: "S25FL128P",
                capacity: 16 * 1024 * 1024, // 16MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),

            // ISSI devices
            [0x9D, 0x60, 0x14] => Some(DeviceInfo {
                manufacturer: "ISSI",
                device_name: "IS25LQ080",
                capacity: 1 * 1024 * 1024, // 1MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0x9D, 0x60, 0x15] => Some(DeviceInfo {
                manufacturer: "ISSI",
                device_name: "IS25LQ016",
                capacity: 2 * 1024 * 1024, // 2MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),
            [0x9D, 0x60, 0x16] => Some(DeviceInfo {
                manufacturer: "ISSI",
                device_name: "IS25LQ032",
                capacity: 4 * 1024 * 1024, // 4MB
                sector_size: 4096,
                page_size: 256,
                block_size: 65536,
            }),

            _ => None, // Unknown device
        }
    }

    /// Decode capacity from JEDEC capacity code (third byte)
    ///
    /// This demonstrates the standard JEDEC encoding for capacity:
    /// - Code 0x14 = 2^20 bits = 1 Mbit = 128 KBytes
    /// - Code 0x15 = 2^21 bits = 2 Mbit = 256 KBytes  
    /// - Code 0x16 = 2^22 bits = 4 Mbit = 512 KBytes
    /// - etc.
    pub fn decode_capacity_from_jedec(capacity_code: u8) -> Option<u32> {
        if capacity_code < 0x10 || capacity_code > 0x25 {
            return None; // Invalid range
        }
        
        // JEDEC capacity code represents 2^n bits
        // Convert to bytes by dividing by 8 (2^3)
        let bits = 1u32 << capacity_code;
        Some(bits / 8)
    }

    /// Get manufacturer name from manufacturer ID (first byte)
    pub fn get_manufacturer_name(manufacturer_id: u8) -> &'static str {
        match manufacturer_id {
            0x01 => "Spansion/Cypress",
            0x20 => "Micron/ST",
            0x9D => "ISSI",
            0xC2 => "Macronix", 
            0xEF => "Winbond",
            0x1F => "Atmel",
            0x37 => "AMIC",
            0x5E => "Zbit",
            0x8C => "ESMT",
            0xA1 => "Fudan",
            0xBF => "SST",
            0xC8 => "GigaDevice",
            0xE0 => "Paragon",
            _ => "Unknown",
        }
    }

    /// Attempt to derive device geometry using standard patterns
    ///
    /// This function shows how to make educated guesses about device geometry
    /// when the exact device isn't in our database but follows standard patterns.
    pub fn derive_geometry_from_patterns(jedec_id: [u8; 3]) -> Option<DeviceInfo> {
        let [manufacturer_id, memory_type, capacity_code] = jedec_id;
        
        // Try to decode capacity using standard JEDEC encoding
        let capacity = Self::decode_capacity_from_jedec(capacity_code)?;
        
        // Most modern SPI NOR devices follow these patterns:
        let (sector_size, page_size, block_size) = match memory_type {
            // Standard SPI NOR (most common)
            0x20 | 0x40 | 0x60 | 0x70 => (4096, 256, 65536),
            // Some older or specialized devices might use different sizes
            0x30 => (4096, 512, 65536), // Larger page size
            0x80 => (4096, 256, 32768), // Smaller block size
            _ => (4096, 256, 65536), // Default assumption
        };

        Some(DeviceInfo {
            manufacturer: Self::get_manufacturer_name(manufacturer_id),
            device_name: "Unknown Device",
            capacity,
            sector_size,
            page_size,
            block_size,
        })
    }
}

/// Auto-detection helper that combines database lookup with pattern matching
pub fn detect_device_geometry(jedec_id: [u8; 3]) -> Option<DeviceInfo> {
    // First try exact database match
    if let Some(info) = JedecDatabase::lookup_device(jedec_id) {
        return Some(info);
    }
    
    // Fall back to pattern-based detection
    JedecDatabase::derive_geometry_from_patterns(jedec_id)
}

/// Additional convenience methods for SPI NOR adapter
impl<D> SpiNorBlockAdapter<D>
where
    D: SpiNorDevice,
{
    /// Read JEDEC ID from the device
    ///
    /// This can be used to identify the flash device before initialization
    pub fn read_jedec_id(&mut self) -> Result<[u8; 3], SpiNorAdapterError<D::Error>> {
        self.device.nor_read_jedec_id().map_err(SpiNorAdapterError::DeviceError)
    }

    /// Reset the SPI NOR device
    ///
    /// This performs a complete reset and clears the initialization state
    pub fn reset(&mut self) -> Result<(), SpiNorAdapterError<D::Error>> {
        self.device.nor_reset_enable().map_err(SpiNorAdapterError::DeviceError)?;
        self.device.nor_reset().map_err(SpiNorAdapterError::DeviceError)?;
        self.device.nor_wait_until_ready();
        
        // Clear initialization state
        self.initialized = false;
        self.config = None;
        Ok(())
    }

    /// Check if the adapter is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the number of blocks (sectors) on the device
    pub fn block_count(&self) -> Option<u32> {
        self.config.map(|config| config.capacity / config.sector_size)
    }

    /// Get the device sector size in bytes
    pub fn sector_size(&self) -> Option<u32> {
        self.config.map(|config| config.sector_size)
    }

    /// Get the device page size in bytes
    pub fn page_size(&self) -> Option<u32> {
        self.config.map(|config| config.page_size)
    }
}

// Example usage and testing
#[cfg(test)]
mod tests {
    use super::*;

    /// Mock SPI NOR device for testing
    struct MockSpiNor {
        data: Vec<u8>,
        jedec_id: [u8; 3],
        write_enabled: bool,
    }

    impl MockSpiNor {
        fn new(capacity: usize, jedec_id: [u8; 3]) -> Self {
            Self {
                data: vec![0xFF; capacity], // NOR flash default state
                jedec_id,
                write_enabled: false,
            }
        }
    }

    #[derive(Debug, PartialEq)]
    enum MockError {
        WriteNotEnabled,
        InvalidAddress,
    }

    impl SpiNorDevice for MockSpiNor {
        type Error = MockError;

        fn nor_read_init(&mut self, _data: &SpiNorData) -> Result<(), Self::Error> {
            Ok(())
        }

        fn nor_write_init(&mut self, _data: &SpiNorData) -> Result<(), Self::Error> {
            Ok(())
        }

        fn nor_write_enable(&mut self) -> Result<(), Self::Error> {
            self.write_enabled = true;
            Ok(())
        }

        fn nor_write_disable(&mut self) -> Result<(), Self::Error> {
            self.write_enabled = false;
            Ok(())
        }

        fn nor_read_jedec_id(&mut self) -> Result<[u8; 3], Self::Error> {
            Ok(self.jedec_id)
        }

        fn nor_sector_erase(&mut self, address: u32) -> Result<(), Self::Error> {
            if !self.write_enabled {
                return Err(MockError::WriteNotEnabled);
            }
            
            let start = address as usize;
            let end = start + 4096; // 4KB sector
            
            if end > self.data.len() {
                return Err(MockError::InvalidAddress);
            }
            
            // Erase sets all bits to 1 (0xFF)
            self.data[start..end].fill(0xFF);
            Ok(())
        }

        fn nor_page_program(&mut self, address: u32, data: &[u8]) -> Result<(), Self::Error> {
            if !self.write_enabled {
                return Err(MockError::WriteNotEnabled);
            }
            
            let start = address as usize;
            let end = start + data.len();
            
            if end > self.data.len() {
                return Err(MockError::InvalidAddress);
            }
            
            // Programming can only clear bits (1 -> 0)
            for (i, &byte) in data.iter().enumerate() {
                self.data[start + i] &= byte;
            }
            Ok(())
        }

        fn nor_page_program_4b(&mut self, address: u32, data: &[u8]) -> Result<(), Self::Error> {
            self.nor_page_program(address, data)
        }

        fn nor_read_data(&mut self, address: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
            let start = address as usize;
            let end = start + buf.len();
            
            if end > self.data.len() {
                return Err(MockError::InvalidAddress);
            }
            
            buf.copy_from_slice(&self.data[start..end]);
            Ok(())
        }

        fn nor_read_fast_4b_data(&mut self, address: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
            self.nor_read_data(address, buf)
        }

        fn nor_sector_aligned(&mut self, address: u32) -> bool {
            address % 4096 == 0
        }

        fn nor_wait_until_ready(&mut self) {
            // Mock device is always ready
        }

        fn nor_reset(&mut self) -> Result<(), Self::Error> {
            self.write_enabled = false;
            Ok(())
        }

        fn nor_reset_enable(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn test_spi_nor_adapter_initialization() {
        let mock_device = MockSpiNor::new(16 * 1024 * 1024, [0xEF, 0x40, 0x18]);
        let mut adapter = SpiNorBlockAdapter::new(mock_device);

        let config = SpiNorData {
            sector_size: 4096,
            page_size: 256,
            capacity: 16 * 1024 * 1024,
            jedec_id: [0xEF, 0x40, 0x18],
        };

        assert!(adapter.initialize(config).is_ok());
        assert!(adapter.is_initialized());
        assert_eq!(adapter.capacity(), 16 * 1024 * 1024);
    }

    #[test]
    fn test_block_device_operations() {
        let mock_device = MockSpiNor::new(16 * 1024 * 1024, [0xEF, 0x40, 0x18]);
        let mut adapter = SpiNorBlockAdapter::new(mock_device);

        let config = SpiNorData {
            sector_size: 4096,
            page_size: 256,
            capacity: 16 * 1024 * 1024,
            jedec_id: [0xEF, 0x40, 0x18],
        };

        adapter.initialize(config).unwrap();

        // Test erase
        let erase_range = BlockRange { start: 0, count: 1 };
        assert!(adapter.erase(erase_range).is_ok());

        // Test program
        let test_data = vec![0x55; 256];
        assert!(adapter.program(0, &test_data).is_ok());

        // Test read
        let mut read_buffer = vec![0; 256];
        assert!(adapter.read(0, &mut read_buffer).is_ok());
        assert_eq!(read_buffer, test_data);
    }
}

/// Example demonstrating typical usage patterns
fn main() {
    println!("SPI NOR Flash Block Device Adapter Example");
    println!();
    println!("This example shows how to use the SpiNorBlockAdapter to bridge");
    println!("between a SPI NOR flash device and the generic BlockDevice trait.");
    println!();
    println!("In a real application, you would:");
    println!("1. Implement the SpiNorDevice trait for your hardware driver");
    println!("2. Create a SpiNorBlockAdapter with your device");
    println!("3. Initialize it with the correct SPI NOR configuration");
    println!("4. Use the adapter as a standard BlockDevice");
    println!();
    println!("See the tests module for a complete working example with a mock device.");
}
