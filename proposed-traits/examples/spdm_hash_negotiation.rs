/// Example implementation demonstrating protocol-driven hash negotiation
/// This shows how the enhanced trait design supports SPDM hash algorithm negotiation
use core::fmt::Debug;
use proposed_traits::digest::{DigestRegistry, ErrorType, Error, ErrorKind, DynamicDigestOp};

// Maximum number of supported algorithms for platform systems
const MAX_ALGORITHMS: usize = 8;
const MAX_HASH_OUTPUT: usize = 64; // SHA-512 output size
const MAX_MESSAGE_BUFFER: usize = 1024;

// SPDM Hash Algorithm Identifiers (from SPDM specification)
pub const SPDM_SHA256: u32 = 0x01;
pub const SPDM_SHA384: u32 = 0x02; 
pub const SPDM_SHA512: u32 = 0x03;

/// Error type for SPDM digest operations
#[derive(Debug)]
pub enum SpdmDigestError {
    UnsupportedAlgorithm,
    NegotiationFailed,
    HardwareError,
    InvalidState,
    BufferTooSmall,
}

impl Error for SpdmDigestError {
    fn kind(&self) -> ErrorKind {
        match self {
            SpdmDigestError::UnsupportedAlgorithm => ErrorKind::UnsupportedAlgorithm,
            SpdmDigestError::NegotiationFailed => ErrorKind::InitializationError,
            SpdmDigestError::HardwareError => ErrorKind::HardwareFailure,
            SpdmDigestError::InvalidState => ErrorKind::NotInitialized,
            SpdmDigestError::BufferTooSmall => ErrorKind::InvalidOutputSize,
        }
    }
}

/// Dynamic digest operation trait for runtime algorithm selection
/// SPDM-specific extension of the generic DynamicDigestOp trait
pub trait DigestOpDyn: DynamicDigestOp<Error = SpdmDigestError, AlgorithmId = u32> {
    /// SPDM-specific finalize that returns output in a fixed-size buffer
    fn finalize(self: Box<Self>) -> Result<(usize, [u8; MAX_HASH_OUTPUT]), SpdmDigestError>;
}

/// SPDM-specific digest registry trait implementing the generic DigestRegistry
/// for SPDM protocol hash algorithm negotiation
pub trait SpdmDigestRegistry: DigestRegistry<AlgorithmId = u32, DigestOp = Box<dyn DigestOpDyn>, Error = SpdmDigestError> {}

/// SPDM-specific hash algorithm negotiation
pub struct SpdmHashNegotiator<D: SpdmDigestRegistry> {
    digest_provider: D,
    negotiated_algorithm: Option<u32>,
    peer_algorithms: [u32; MAX_ALGORITHMS],
    peer_count: usize,
}

impl<D: SpdmDigestRegistry> SpdmHashNegotiator<D> {
    pub fn new(digest_provider: D) -> Self {
        Self {
            digest_provider,
            negotiated_algorithm: None,
            peer_algorithms: [0; MAX_ALGORITHMS],
            peer_count: 0,
        }
    }
    
    /// Set peer's supported algorithms from SPDM capabilities exchange
    pub fn set_peer_algorithms(&mut self, peer_algorithms: &[u32]) -> Result<(), SpdmDigestError> {
        if peer_algorithms.len() > MAX_ALGORITHMS {
            return Err(SpdmDigestError::BufferTooSmall);
        }
        
        self.peer_count = peer_algorithms.len();
        self.peer_algorithms[..self.peer_count].copy_from_slice(peer_algorithms);
        Ok(())
    }
    
    /// Negotiate hash algorithm based on capabilities
    /// Follows SPDM preference order: SHA-512 > SHA-384 > SHA-256
    pub fn negotiate_algorithm(&mut self) -> Result<u32, SpdmDigestError> {
        // SPDM algorithm preference order (strongest to weakest)
        let preference_order = [SPDM_SHA512, SPDM_SHA384, SPDM_SHA256];
        
        for &algo in &preference_order {
            if self.digest_provider.supports_algorithm(algo) && 
               self.peer_algorithms[..self.peer_count].contains(&algo) {
                self.negotiated_algorithm = Some(algo);
                return Ok(algo);
            }
        }
        
        Err(SpdmDigestError::NegotiationFailed)
    }
    
    /// Get the currently negotiated algorithm
    pub fn get_negotiated_algorithm(&self) -> Option<u32> {
        self.negotiated_algorithm
    }
    
    /// Create hash operation using negotiated algorithm
    pub fn create_hash(&mut self) -> Result<Box<dyn DigestOpDyn>, SpdmDigestError> {
        let algo = self.negotiated_algorithm
            .ok_or(SpdmDigestError::InvalidState)?;
        self.digest_provider.create_digest(algo)
    }
    
    /// Get output size for negotiated algorithm
    pub fn get_negotiated_output_size(&self) -> Option<usize> {
        self.negotiated_algorithm
            .and_then(|algo| self.digest_provider.get_output_size(algo))
    }
}

/// Example digest registry for platform systems
pub struct PlatformDigestRegistry {
    supported_algorithms: &'static [u32],
}

impl ErrorType for PlatformDigestRegistry {
    type Error = SpdmDigestError;
}

impl PlatformDigestRegistry {
    pub fn new() -> Self {
        Self {
            // Platform systems typically support SHA-256, SHA-384, SHA-512
            supported_algorithms: &[SPDM_SHA256, SPDM_SHA384, SPDM_SHA512],
        }
    }
}

impl DigestRegistry for PlatformDigestRegistry {
    type AlgorithmId = u32;
    type DigestOp = Box<dyn DigestOpDyn>;

    fn supports_algorithm(&self, algorithm_id: u32) -> bool {
        self.supported_algorithms.contains(&algorithm_id)
    }
    
    fn get_output_size(&self, algorithm_id: u32) -> Option<usize> {
        match algorithm_id {
            SPDM_SHA256 => Some(32),  // 256 bits = 32 bytes
            SPDM_SHA384 => Some(48),  // 384 bits = 48 bytes  
            SPDM_SHA512 => Some(64),  // 512 bits = 64 bytes
            _ => None,
        }
    }
    
    fn create_digest(&mut self, algorithm_id: u32) -> Result<Box<dyn DigestOpDyn>, SpdmDigestError> {
        match algorithm_id {
            SPDM_SHA256 => Ok(Box::new(PlatformSha256Op::new())),
            SPDM_SHA384 => Ok(Box::new(PlatformSha384Op::new())),
            SPDM_SHA512 => Ok(Box::new(PlatformSha512Op::new())),
            _ => Err(SpdmDigestError::UnsupportedAlgorithm),
        }
    }
    
    fn supported_algorithms(&self) -> &[u32] {
        self.supported_algorithms
    }
}

impl SpdmDigestRegistry for PlatformDigestRegistry {}

/// Example platform SHA-256 operation
struct PlatformSha256Op {
    buffer: [u8; MAX_MESSAGE_BUFFER],
    buffer_len: usize,
}

impl PlatformSha256Op {
    fn new() -> Self {
        Self {
            buffer: [0; MAX_MESSAGE_BUFFER],
            buffer_len: 0,
        }
    }
}

impl DynamicDigestOp for PlatformSha256Op {
    type Error = SpdmDigestError;
    type AlgorithmId = u32;

    fn update(&mut self, input: &[u8]) -> Result<(), SpdmDigestError> {
        if self.buffer_len + input.len() > MAX_MESSAGE_BUFFER {
            return Err(SpdmDigestError::BufferTooSmall);
        }
        
        self.buffer[self.buffer_len..self.buffer_len + input.len()].copy_from_slice(input);
        self.buffer_len += input.len();
        // In real implementation, this would process data using available crypto engine
        Ok(())
    }

    fn finalize(&mut self) -> Result<(), SpdmDigestError> {
        // In real implementation, this would finalize the digest computation
        Ok(())
    }

    fn output_size(&self) -> usize {
        32
    }

    fn algorithm_id(&self) -> u32 {
        SPDM_SHA256
    }

    fn copy_output(&self, output: &mut [u8]) -> Result<usize, SpdmDigestError> {
        if output.len() < 32 {
            return Err(SpdmDigestError::BufferTooSmall);
        }
        
        // Simulate SHA-256 output (32 bytes)
        for i in 0..32 {
            output[i] = (i as u8).wrapping_add(0xAB); // Mock hash data
        }
        Ok(32)
    }
}

impl DigestOpDyn for PlatformSha256Op {
    fn finalize(self: Box<Self>) -> Result<(usize, [u8; MAX_HASH_OUTPUT]), SpdmDigestError> {
        // In real implementation, this would compute the final hash
        // For demo, return mock 32-byte hash
        let mut output = [0u8; MAX_HASH_OUTPUT];
        // Simulate SHA-256 output (32 bytes)
        for i in 0..32 {
            output[i] = (i as u8).wrapping_add(0xAB); // Mock hash data
        }
        Ok((32, output))
    }
}

/// Example platform SHA-384 operation
struct PlatformSha384Op {
    buffer: [u8; MAX_MESSAGE_BUFFER],
    buffer_len: usize,
}

impl PlatformSha384Op {
    fn new() -> Self {
        Self {
            buffer: [0; MAX_MESSAGE_BUFFER],
            buffer_len: 0,
        }
    }
}

impl DynamicDigestOp for PlatformSha384Op {
    type Error = SpdmDigestError;
    type AlgorithmId = u32;

    fn update(&mut self, input: &[u8]) -> Result<(), SpdmDigestError> {
        if self.buffer_len + input.len() > MAX_MESSAGE_BUFFER {
            return Err(SpdmDigestError::BufferTooSmall);
        }
        
        self.buffer[self.buffer_len..self.buffer_len + input.len()].copy_from_slice(input);
        self.buffer_len += input.len();
        Ok(())
    }

    fn finalize(&mut self) -> Result<(), SpdmDigestError> {
        Ok(())
    }

    fn output_size(&self) -> usize {
        48
    }

    fn algorithm_id(&self) -> u32 {
        SPDM_SHA384
    }

    fn copy_output(&self, output: &mut [u8]) -> Result<usize, SpdmDigestError> {
        if output.len() < 48 {
            return Err(SpdmDigestError::BufferTooSmall);
        }
        
        // Simulate SHA-384 output (48 bytes)
        for i in 0..48 {
            output[i] = (i as u8).wrapping_add(0xCD); // Mock hash data
        }
        Ok(48)
    }
}

impl DigestOpDyn for PlatformSha384Op {
    fn finalize(self: Box<Self>) -> Result<(usize, [u8; MAX_HASH_OUTPUT]), SpdmDigestError> {
        // Return mock 48-byte hash
        let mut output = [0u8; MAX_HASH_OUTPUT];
        // Simulate SHA-384 output (48 bytes)
        for i in 0..48 {
            output[i] = (i as u8).wrapping_add(0xCD); // Mock hash data
        }
        Ok((48, output))
    }
}

/// Example platform SHA-512 operation
struct PlatformSha512Op {
    buffer: [u8; MAX_MESSAGE_BUFFER],
    buffer_len: usize,
}

impl PlatformSha512Op {
    fn new() -> Self {
        Self {
            buffer: [0; MAX_MESSAGE_BUFFER],
            buffer_len: 0,
        }
    }
}

impl DynamicDigestOp for PlatformSha512Op {
    type Error = SpdmDigestError;
    type AlgorithmId = u32;

    fn update(&mut self, input: &[u8]) -> Result<(), SpdmDigestError> {
        if self.buffer_len + input.len() > MAX_MESSAGE_BUFFER {
            return Err(SpdmDigestError::BufferTooSmall);
        }
        
        self.buffer[self.buffer_len..self.buffer_len + input.len()].copy_from_slice(input);
        self.buffer_len += input.len();
        Ok(())
    }

    fn finalize(&mut self) -> Result<(), SpdmDigestError> {
        Ok(())
    }

    fn output_size(&self) -> usize {
        64
    }

    fn algorithm_id(&self) -> u32 {
        SPDM_SHA512
    }

    fn copy_output(&self, output: &mut [u8]) -> Result<usize, SpdmDigestError> {
        if output.len() < 64 {
            return Err(SpdmDigestError::BufferTooSmall);
        }
        
        // Simulate SHA-512 output (64 bytes)
        for i in 0..64 {
            output[i] = (i as u8).wrapping_add(0xEF); // Mock hash data
        }
        Ok(64)
    }
}

impl DigestOpDyn for PlatformSha512Op {
    fn finalize(self: Box<Self>) -> Result<(usize, [u8; MAX_HASH_OUTPUT]), SpdmDigestError> {
        // Return mock 64-byte hash
        let mut output = [0u8; MAX_HASH_OUTPUT];
        // Simulate SHA-512 output (64 bytes)
        for i in 0..64 {
            output[i] = (i as u8).wrapping_add(0xEF); // Mock hash data
        }
        Ok((64, output))
    }
}

/// SPDM session demonstrating protocol-driven hash negotiation
pub struct SpdmSession<D: SpdmDigestRegistry> {
    hash_negotiator: SpdmHashNegotiator<D>,
    session_state: SpdmSessionState,
}

#[derive(Debug, PartialEq)]
pub enum SpdmSessionState {
    Initial,
    CapabilitiesExchanged,
    AlgorithmsNegotiated,
    Established,
}

impl<D: SpdmDigestRegistry> SpdmSession<D> {
    pub fn new(digest_provider: D) -> Self {
        Self {
            hash_negotiator: SpdmHashNegotiator::new(digest_provider),
            session_state: SpdmSessionState::Initial,
        }
    }
    
    /// Handle SPDM GET_CAPABILITIES message
    pub fn handle_get_capabilities(&mut self, peer_algorithms: &[u32]) -> Result<&[u32], SpdmDigestError> {
        self.hash_negotiator.set_peer_algorithms(peer_algorithms)?;
        self.session_state = SpdmSessionState::CapabilitiesExchanged;
        
        // Return our supported algorithms
        Ok(self.hash_negotiator.digest_provider.supported_algorithms())
    }
    
    /// Handle SPDM NEGOTIATE_ALGORITHMS message
    pub fn handle_negotiate_algorithms(&mut self) -> Result<u32, SpdmDigestError> {
        if self.session_state != SpdmSessionState::CapabilitiesExchanged {
            return Err(SpdmDigestError::InvalidState);
        }
        
        let negotiated_algo = self.hash_negotiator.negotiate_algorithm()?;
        self.session_state = SpdmSessionState::AlgorithmsNegotiated;
        
        Ok(negotiated_algo)
    }
    
    /// Process SPDM message with negotiated hash algorithm
    pub fn process_message(&mut self, message: &[u8]) -> Result<(usize, [u8; MAX_HASH_OUTPUT]), SpdmDigestError> {
        if self.session_state != SpdmSessionState::AlgorithmsNegotiated {
            return Err(SpdmDigestError::InvalidState);
        }
        
        // Create hash operation using negotiated algorithm
        let mut hash_op = self.hash_negotiator.create_hash()?;
        hash_op.update(message)?;
        let hash_result = hash_op.finalize()?;
        
        Ok(hash_result)
    }
    
    /// Get information about the negotiated hash algorithm
    pub fn get_hash_info(&self) -> Option<(u32, usize)> {
        self.hash_negotiator.get_negotiated_algorithm()
            .zip(self.hash_negotiator.get_negotiated_output_size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spdm_hash_negotiation() {
        let registry = PlatformDigestRegistry::new();
        let mut session = SpdmSession::new(registry);
        
        // Simulate peer supporting SHA-256 and SHA-384
        let peer_algorithms = [SPDM_SHA256, SPDM_SHA384];
        let our_algorithms = session.handle_get_capabilities(&peer_algorithms).unwrap();
        
        // Verify we support SHA-256, SHA-384, and SHA-512
        assert!(our_algorithms.contains(&SPDM_SHA256));
        assert!(our_algorithms.contains(&SPDM_SHA384));
        assert!(our_algorithms.contains(&SPDM_SHA512));
        
        // Negotiate algorithms - should choose SHA-384 (strongest common)
        let negotiated = session.handle_negotiate_algorithms().unwrap();
        assert_eq!(negotiated, SPDM_SHA384);
        
        // Verify hash info
        let (algo, size) = session.get_hash_info().unwrap();
        assert_eq!(algo, SPDM_SHA384);
        assert_eq!(size, 48); // 384 bits = 48 bytes
        
        // Process a message
        let message = b"SPDM test message";
        let (hash_len, hash_result) = session.process_message(message).unwrap();
        assert_eq!(hash_len, 48); // SHA-384 output size
        
        // Verify the mock hash content for SHA-384 (starts with 0xCD pattern)
        assert_eq!(hash_result[0], 0xCD);
        assert_eq!(hash_result[1], 0xCE);
    }
    
    #[test]
    fn test_negotiation_failure() {
        let registry = PlatformDigestRegistry::new();
        let mut session = SpdmSession::new(registry);
        
        // Simulate peer supporting only unsupported algorithm
        let peer_algorithms = [0xFF]; // Unsupported algorithm
        let _our_algorithms = session.handle_get_capabilities(&peer_algorithms).unwrap();
        
        // Negotiation should fail
        let result = session.handle_negotiate_algorithms();
        assert!(matches!(result, Err(SpdmDigestError::NegotiationFailed)));
    }
}

/// Example demonstrating SPDM hash negotiation without dynamic allocation
fn main() -> Result<(), SpdmDigestError> {
    println!("SPDM Hash Negotiation Example (No Vec usage)");
    
    // Create platform digest registry
    let registry = PlatformDigestRegistry::new();
    let mut spdm_session = SpdmSession::new(registry);
    
    println!("Created SPDM session with platform digest registry");
    
    // Simulate peer capabilities exchange
    let peer_algorithms = [SPDM_SHA256, SPDM_SHA384];
    println!("Peer supports algorithms: {:?}", peer_algorithms);
    
    let our_algorithms = spdm_session.handle_get_capabilities(&peer_algorithms)?;
    println!("Our supported algorithms: {:?}", our_algorithms);
    
    // Negotiate algorithms
    let negotiated_algorithm = spdm_session.handle_negotiate_algorithms()?;
    println!("Negotiated algorithm: 0x{:02X}", negotiated_algorithm);
    
    if let Some((algo_id, output_size)) = spdm_session.get_hash_info() {
        println!("Algorithm ID: 0x{:02X}, Output size: {} bytes", algo_id, output_size);
    }
    
    // Process a test message
    let test_message = b"Hello, SPDM! This is a test message for hash processing.";
    println!("Processing message: {:?}", core::str::from_utf8(test_message).unwrap());
    
    let (hash_length, hash_output) = spdm_session.process_message(test_message)?;
    println!("Hash output length: {} bytes", hash_length);
    println!("Hash output (first 16 bytes): {:02X?}", &hash_output[..16]);
    
    // Demonstrate buffer reuse - process another message
    let second_message = b"Another message to demonstrate embedded-friendly operation";
    let (hash_length2, hash_output2) = spdm_session.process_message(second_message)?;
    println!("Second hash length: {} bytes", hash_length2);
    println!("Second hash (first 16 bytes): {:02X?}", &hash_output2[..16]);
    
    println!("Successfully completed SPDM hash negotiation without dynamic allocation!");
    
    Ok(())
}
