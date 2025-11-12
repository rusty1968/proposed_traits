//! Software ECDSA implementation using RustCrypto crates
//!
//! This example demonstrates how to implement the ECDSA traits defined in
//! `proposed-traits/src/ecdsa.rs` using the RustCrypto ecosystem (`p256`, `ecdsa` crates).
//!
//! The implementation provides:
//! - P256 curve support with SHA-256 digest
//! - Key generation, signing, and verification
//! - Proper error handling and endianness support
//! - Both trait-based and direct API usage examples

use core::fmt::Debug;
use proposed_traits::{
    ecdsa::{
        Curve, EcdsaKeyGen, EcdsaSign, EcdsaVerify, ErrorType as EcdsaErrorType, Error, ErrorKind,
        PrivateKeyForCurve, PubKeyForCurve, SignatureForCurve
    },
    digest::{DigestAlgorithm},
    common::{FromBytes, ToBytes, Endian, ErrorType as CommonErrorType, SerdeError, ErrorKind as CommonErrorKind}
};

// Re-export crypto crates for convenience
pub use p256::{
    ecdsa::{Signature as P256Signature, SigningKey, VerifyingKey},
    elliptic_curve::{FieldBytes, ScalarPrimitive, sec1::ToEncodedPoint, PrimeField},
    Scalar, AffinePoint
};
pub use ecdsa::{signature::{Signer, Verifier, hazmat::PrehashVerifier}};
pub use rand_core::{RngCore, CryptoRng};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};

/// Software-specific ECDSA error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftwareEcdsaError {
    /// Invalid signature
    InvalidSignature,
    /// Key generation failed
    KeyGenError,
    /// Signing operation failed
    SigningError,
    /// Invalid key format
    InvalidKey,
    /// Invalid curve point
    InvalidPoint,
    /// RNG error
    RngError,
    /// Buffer too small
    BufferTooSmall,
    /// Generic computation error
    ComputationError,
}

impl Error for SoftwareEcdsaError {
    fn kind(&self) -> ErrorKind {
        match self {
            SoftwareEcdsaError::InvalidSignature => ErrorKind::InvalidSignature,
            SoftwareEcdsaError::KeyGenError => ErrorKind::KeyGenError,
            SoftwareEcdsaError::SigningError => ErrorKind::SigningError,
            SoftwareEcdsaError::InvalidKey => ErrorKind::Other,
            SoftwareEcdsaError::InvalidPoint => ErrorKind::Other,
            SoftwareEcdsaError::RngError => ErrorKind::Other,
            SoftwareEcdsaError::BufferTooSmall => ErrorKind::Other,
            SoftwareEcdsaError::ComputationError => ErrorKind::Other,
        }
    }
}

impl SerdeError for SoftwareEcdsaError {
    fn kind(&self) -> CommonErrorKind {
        match self {
            SoftwareEcdsaError::BufferTooSmall => CommonErrorKind::DestinationBufferTooSmall,
            _ => CommonErrorKind::Other,
        }
    }
}

/// P-256 curve definition
#[derive(Debug, Clone, Copy)]
pub struct P256Curve;

/// SHA-256 algorithm for P-256
#[derive(Debug, Clone, Copy)]
pub struct Sha256Algorithm;

impl DigestAlgorithm for Sha256Algorithm {
    const OUTPUT_BITS: usize = 256;
    type DigestOutput = Vec<u8>;
}

impl Curve for P256Curve {
    type DigestType = Sha256Algorithm;
    type Scalar = P256ScalarWrapper;
}

/// Wrapper for P-256 scalar
#[derive(Debug, Clone)]
pub struct P256ScalarWrapper {
    pub scalar: Scalar,
}

impl EcdsaErrorType for P256ScalarWrapper {
    type Error = SoftwareEcdsaError;
}

impl CommonErrorType for P256ScalarWrapper {
    type Error = SoftwareEcdsaError;
}

impl ToBytes for P256ScalarWrapper {
    fn to_bytes(&self, dest: &mut [u8], endian: Endian) -> Result<(), Self::Error> {
        if dest.len() < 32 {
            return Err(SoftwareEcdsaError::BufferTooSmall);
        }
        
        let bytes = self.scalar.to_bytes();
        match endian {
            Endian::Big => dest[..32].copy_from_slice(&bytes),
            Endian::Little => {
                for (i, &byte) in bytes.iter().enumerate() {
                    dest[31 - i] = byte;
                }
            }
        }
        Ok(())
    }
}

impl FromBytes for P256ScalarWrapper {
    fn from_bytes(bytes: &[u8], endian: Endian) -> Result<Self, Self::Error> {
        if bytes.len() != 32 {
            return Err(SoftwareEcdsaError::InvalidKey);
        }
        
        let mut scalar_bytes = [0u8; 32];
        match endian {
            Endian::Big => scalar_bytes.copy_from_slice(bytes),
            Endian::Little => {
                for (i, &byte) in bytes.iter().enumerate() {
                    scalar_bytes[31 - i] = byte;
                }
            }
        }
        
        let scalar = Scalar::from_repr(scalar_bytes.into())
            .into_option()
            .ok_or(SoftwareEcdsaError::InvalidKey)?;
            
        Ok(P256ScalarWrapper { scalar })
    }
}

/// P-256 private key wrapper
pub struct SoftwareP256PrivateKey {
    pub signing_key: SigningKey,
}

impl EcdsaErrorType for SoftwareP256PrivateKey {
    type Error = SoftwareEcdsaError;
}

impl CommonErrorType for SoftwareP256PrivateKey {
    type Error = SoftwareEcdsaError;
}

impl ToBytes for SoftwareP256PrivateKey {
    fn to_bytes(&self, dest: &mut [u8], endian: Endian) -> Result<(), Self::Error> {
        if dest.len() < 32 {
            return Err(SoftwareEcdsaError::BufferTooSmall);
        }
        
        let bytes = self.signing_key.to_bytes();
        match endian {
            Endian::Big => dest[..32].copy_from_slice(&bytes),
            Endian::Little => {
                for (i, &byte) in bytes.iter().enumerate() {
                    dest[31 - i] = byte;
                }
            }
        }
        Ok(())
    }
}

impl FromBytes for SoftwareP256PrivateKey {
    fn from_bytes(bytes: &[u8], endian: Endian) -> Result<Self, Self::Error> {
        if bytes.len() != 32 {
            return Err(SoftwareEcdsaError::InvalidKey);
        }
        
        let mut key_bytes = [0u8; 32];
        match endian {
            Endian::Big => key_bytes.copy_from_slice(bytes),
            Endian::Little => {
                for (i, &byte) in bytes.iter().enumerate() {
                    key_bytes[31 - i] = byte;
                }
            }
        }
        
        let signing_key = SigningKey::from_bytes(&key_bytes.into())
            .map_err(|_| SoftwareEcdsaError::InvalidKey)?;
            
        Ok(SoftwareP256PrivateKey { signing_key })
    }
}

impl PrivateKeyForCurve<P256Curve> for SoftwareP256PrivateKey {
    fn zeroize(&mut self) {
        // SigningKey from p256 implements Drop with zeroization
    }
}

/// P-256 public key wrapper
#[derive(Debug, Clone)]
pub struct SoftwareP256PublicKey {
    pub verifying_key: VerifyingKey,
}

impl EcdsaErrorType for SoftwareP256PublicKey {
    type Error = SoftwareEcdsaError;
}

impl CommonErrorType for SoftwareP256PublicKey {
    type Error = SoftwareEcdsaError;
}

impl ToBytes for SoftwareP256PublicKey {
    fn to_bytes(&self, dest: &mut [u8], endian: Endian) -> Result<(), Self::Error> {
        let point = self.verifying_key.to_encoded_point(false); // Uncompressed
        
        // Extract coordinates from the encoded point bytes
        let point_bytes = point.as_bytes();
        if point_bytes.len() < 65 || point_bytes[0] != 0x04 {
            return Err(SoftwareEcdsaError::InvalidPoint);
        }
        
        if dest.len() < 64 {
            return Err(SoftwareEcdsaError::BufferTooSmall);
        }
        
        match endian {
            Endian::Big => {
                dest[..32].copy_from_slice(&point_bytes[1..33]); // x coordinate
                dest[32..64].copy_from_slice(&point_bytes[33..65]); // y coordinate
            }
            Endian::Little => {
                for (i, &byte) in point_bytes[1..33].iter().enumerate() {
                    dest[31 - i] = byte;
                }
                for (i, &byte) in point_bytes[33..65].iter().enumerate() {
                    dest[63 - i] = byte;
                }
            }
        }
        Ok(())
    }
}

impl FromBytes for SoftwareP256PublicKey {
    fn from_bytes(bytes: &[u8], endian: Endian) -> Result<Self, Self::Error> {
        if bytes.len() != 64 {
            return Err(SoftwareEcdsaError::InvalidKey);
        }
        
        let mut x_bytes = [0u8; 32];
        let mut y_bytes = [0u8; 32];
        
        match endian {
            Endian::Big => {
                x_bytes.copy_from_slice(&bytes[..32]);
                y_bytes.copy_from_slice(&bytes[32..64]);
            }
            Endian::Little => {
                for (i, &byte) in bytes[..32].iter().enumerate() {
                    x_bytes[31 - i] = byte;
                }
                for (i, &byte) in bytes[32..64].iter().enumerate() {
                    y_bytes[31 - i] = byte;
                }
            }
        }
        
        let _x_scalar = Scalar::from_repr(x_bytes.into())
            .into_option()
            .ok_or(SoftwareEcdsaError::InvalidKey)?;
        let _y_scalar = Scalar::from_repr(y_bytes.into())
            .into_option()
            .ok_or(SoftwareEcdsaError::InvalidKey)?;
        
        // Create affine point from x,y coordinates
        let encoded_point = {
            let mut point_bytes = [0u8; 65];
            point_bytes[0] = 0x04; // Uncompressed point prefix
            point_bytes[1..33].copy_from_slice(&x_bytes);
            point_bytes[33..65].copy_from_slice(&y_bytes);
            point_bytes
        };
        
        let verifying_key = VerifyingKey::from_sec1_bytes(&encoded_point)
            .map_err(|_| SoftwareEcdsaError::InvalidKey)?;
            
        Ok(SoftwareP256PublicKey { verifying_key })
    }
}

impl PubKeyForCurve<P256Curve> for SoftwareP256PublicKey {
    fn x(&self) -> &P256ScalarWrapper {
        // This is problematic with the current trait design since we need to 
        // return a reference to a scalar that we need to extract from the key
        // For now, we'll use unimplemented as this requires a trait redesign
        unimplemented!("Coordinate extraction requires trait redesign")
    }
    
    fn y(&self) -> &P256ScalarWrapper {
        unimplemented!("Coordinate extraction requires trait redesign")
    }
    
    fn new(x: P256ScalarWrapper, y: P256ScalarWrapper) -> Self {
        // Create point from scalar coordinates by encoding as SEC1 uncompressed
        let mut point_bytes = [0u8; 65];
        point_bytes[0] = 0x04; // Uncompressed point prefix
        point_bytes[1..33].copy_from_slice(&x.scalar.to_bytes());
        point_bytes[33..65].copy_from_slice(&y.scalar.to_bytes());
        
        let verifying_key = VerifyingKey::from_sec1_bytes(&point_bytes)
            .expect("Invalid public key coordinates");
        SoftwareP256PublicKey { verifying_key }
    }
}

/// P-256 ECDSA signature wrapper
#[derive(Debug, Clone)]
pub struct SoftwareP256Signature {
    pub signature: P256Signature,
}

impl EcdsaErrorType for SoftwareP256Signature {
    type Error = SoftwareEcdsaError;
}

impl CommonErrorType for SoftwareP256Signature {
    type Error = SoftwareEcdsaError;
}

impl ToBytes for SoftwareP256Signature {
    fn to_bytes(&self, dest: &mut [u8], endian: Endian) -> Result<(), Self::Error> {
        if dest.len() < 64 {
            return Err(SoftwareEcdsaError::BufferTooSmall);
        }
        
        let r_bytes = self.signature.r().to_bytes();
        let s_bytes = self.signature.s().to_bytes();
        
        match endian {
            Endian::Big => {
                dest[..32].copy_from_slice(&r_bytes);
                dest[32..64].copy_from_slice(&s_bytes);
            }
            Endian::Little => {
                for (i, &byte) in r_bytes.iter().enumerate() {
                    dest[31 - i] = byte;
                }
                for (i, &byte) in s_bytes.iter().enumerate() {
                    dest[63 - i] = byte;
                }
            }
        }
        Ok(())
    }
}

impl FromBytes for SoftwareP256Signature {
    fn from_bytes(bytes: &[u8], endian: Endian) -> Result<Self, Self::Error> {
        if bytes.len() != 64 {
            return Err(SoftwareEcdsaError::InvalidSignature);
        }
        
        let mut r_bytes = [0u8; 32];
        let mut s_bytes = [0u8; 32];
        
        match endian {
            Endian::Big => {
                r_bytes.copy_from_slice(&bytes[..32]);
                s_bytes.copy_from_slice(&bytes[32..64]);
            }
            Endian::Little => {
                for (i, &byte) in bytes[..32].iter().enumerate() {
                    r_bytes[31 - i] = byte;
                }
                for (i, &byte) in bytes[32..64].iter().enumerate() {
                    s_bytes[31 - i] = byte;
                }
            }
        }
        
        let signature = P256Signature::from_scalars(r_bytes, s_bytes)
            .map_err(|_| SoftwareEcdsaError::InvalidSignature)?;
            
        Ok(SoftwareP256Signature { signature })
    }
}

impl SignatureForCurve<P256Curve> for SoftwareP256Signature {
    fn r(&self) -> &P256ScalarWrapper {
        // Same issue as with public key coordinates
        unimplemented!("Coordinate extraction requires trait redesign")
    }
    
    fn s(&self) -> &P256ScalarWrapper {
        unimplemented!("Coordinate extraction requires trait redesign")
    }
    
    fn new(r: P256ScalarWrapper, s: P256ScalarWrapper) -> Self {
        let signature = P256Signature::from_scalars(r.scalar.to_bytes(), s.scalar.to_bytes())
            .expect("Invalid signature scalars");
        SoftwareP256Signature { signature }
    }
}

/// Software ECDSA provider
pub struct SoftwareEcdsa;

impl EcdsaErrorType for SoftwareEcdsa {
    type Error = SoftwareEcdsaError;
}

impl EcdsaKeyGen<P256Curve> for SoftwareEcdsa {
    type PrivateKey = SoftwareP256PrivateKey;
    type PublicKey = SoftwareP256PublicKey;
    
    fn generate_key_pair<R>(
        &mut self,
        rng: &mut R,
    ) -> Result<(Self::PrivateKey, Self::PublicKey), Self::Error>
    where
        R: RngCore + CryptoRng,
    {
        let signing_key = SigningKey::random(rng);
        let verifying_key = VerifyingKey::from(&signing_key);
        
        let private_key = SoftwareP256PrivateKey { signing_key };
        let public_key = SoftwareP256PublicKey { verifying_key };
        
        Ok((private_key, public_key))
    }
}

impl EcdsaSign<P256Curve> for SoftwareEcdsa {
    type PrivateKey = SoftwareP256PrivateKey;
    type Signature = SoftwareP256Signature;
    
    fn sign<R>(
        &mut self,
        private_key: &Self::PrivateKey,
        digest: <<P256Curve as Curve>::DigestType as DigestAlgorithm>::DigestOutput,
        _rng: &mut R,
    ) -> Result<Self::Signature, Self::Error>
    where
        R: RngCore + CryptoRng,
    {
        if digest.len() != 32 {
            return Err(SoftwareEcdsaError::SigningError);
        }
        
        let signature: P256Signature = private_key.signing_key.sign_prehash_recoverable(&digest)
            .map_err(|_| SoftwareEcdsaError::SigningError)?
            .0;
            
        Ok(SoftwareP256Signature { signature })
    }
}

impl EcdsaVerify<P256Curve> for SoftwareEcdsa {
    type PublicKey = SoftwareP256PublicKey;
    type Signature = SoftwareP256Signature;
    
    fn verify(
        &mut self,
        public_key: &Self::PublicKey,
        digest: <<P256Curve as Curve>::DigestType as DigestAlgorithm>::DigestOutput,
        signature: &Self::Signature,
    ) -> Result<(), Self::Error> {
        if digest.len() != 32 {
            return Err(SoftwareEcdsaError::InvalidSignature);
        }
        
        public_key.verifying_key.verify_prehash(&digest, &signature.signature)
            .map_err(|_| SoftwareEcdsaError::InvalidSignature)
    }
}

/// Helper function to serialize keys/signatures for display
fn serialize_to_bytes<T: ToBytes>(item: &T, size: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    item.to_bytes(&mut bytes, Endian::Big).expect("Serialization failed");
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let mut ecdsa = SoftwareEcdsa;
        let mut rng = OsRng;
        
        let result = ecdsa.generate_key_pair(&mut rng);
        assert!(result.is_ok(), "Key generation should succeed");
        
        println!("✓ Key generation test passed");
    }

    #[test]
    fn test_sign_and_verify() {
        let mut ecdsa = SoftwareEcdsa;
        let mut rng = OsRng;
        
        // Generate key pair
        let (private_key, public_key) = ecdsa.generate_key_pair(&mut rng).unwrap();
        
        // Create test digest
        let message = b"Hello, ECDSA!";
        let mut hasher = Sha256::new();
        hasher.update(message);
        let digest = hasher.finalize().to_vec();
        
        // Sign
        let signature = ecdsa.sign(&private_key, digest.clone(), &mut rng).unwrap();
        
        // Verify
        let result = ecdsa.verify(&public_key, digest, &signature);
        assert!(result.is_ok(), "Signature verification should succeed");
        
        println!("✓ Sign and verify test passed");
    }

    #[test]
    fn test_serialization() {
        let mut ecdsa = SoftwareEcdsa;
        let mut rng = OsRng;
        
        // Generate key pair
        let (private_key, public_key) = ecdsa.generate_key_pair(&mut rng).unwrap();
        
        // Test private key serialization roundtrip
        let mut private_bytes = [0u8; 32];
        private_key.to_bytes(&mut private_bytes, Endian::Big).unwrap();
        let restored_private = SoftwareP256PrivateKey::from_bytes(&private_bytes, Endian::Big).unwrap();
        
        // Test public key serialization roundtrip
        let mut public_bytes = [0u8; 64];
        public_key.to_bytes(&mut public_bytes, Endian::Big).unwrap();
        let restored_public = SoftwareP256PublicKey::from_bytes(&public_bytes, Endian::Big).unwrap();
        
        // Test that deserialized keys work
        let message = b"Test serialization";
        let mut hasher = Sha256::new();
        hasher.update(message);
        let digest = hasher.finalize().to_vec();
        
        let signature = ecdsa.sign(&restored_private, digest.clone(), &mut rng).unwrap();
        let verification = ecdsa.verify(&restored_public, digest, &signature);
        assert!(verification.is_ok(), "Verification with restored keys should work");
        
        println!("✓ Serialization test passed");
    }

    #[test]
    fn test_endianness() {
        let mut ecdsa = SoftwareEcdsa;
        let mut rng = OsRng;
        
        let (private_key, _) = ecdsa.generate_key_pair(&mut rng).unwrap();
        
        // Test different endianness
        let mut big_endian = [0u8; 32];
        let mut little_endian = [0u8; 32];
        
        private_key.to_bytes(&mut big_endian, Endian::Big).unwrap();
        private_key.to_bytes(&mut little_endian, Endian::Little).unwrap();
        
        // Should produce different byte orders
        assert_ne!(big_endian, little_endian);
        
        // Both should deserialize to functional keys
        let restored_big = SoftwareP256PrivateKey::from_bytes(&big_endian, Endian::Big).unwrap();
        let restored_little = SoftwareP256PrivateKey::from_bytes(&little_endian, Endian::Little).unwrap();
        
        // Test that both work for signing
        let test_data = b"endianness test";
        let mut hasher = Sha256::new();
        hasher.update(test_data);
        let digest = hasher.finalize().to_vec();
        
        let _sig1 = ecdsa.sign(&restored_big, digest.clone(), &mut rng).unwrap();
        let _sig2 = ecdsa.sign(&restored_little, digest.clone(), &mut rng).unwrap();
        
        // Both should succeed (signatures will likely be different due to randomness)
        println!("✓ Endianness test passed");
    }
}

/// Simple hex encoding for output
fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

fn main() -> Result<(), SoftwareEcdsaError> {
    println!("🔐 Software ECDSA Implementation Example");
    println!("========================================");
    
    let mut ecdsa = SoftwareEcdsa;
    let mut rng = OsRng;
    
    // Generate key pair
    println!("Generating ECDSA key pair...");
    let (private_key, public_key) = ecdsa.generate_key_pair(&mut rng)?;
    println!("✓ Key pair generated successfully");
    
    // Display key info
    let private_bytes = serialize_to_bytes(&private_key, 32);
    let public_bytes = serialize_to_bytes(&public_key, 64);
    
    println!("Private key (first 16 bytes): {}...", 
             hex_encode(&private_bytes[..16]));
    println!("Public key: {}", 
             hex_encode(&public_bytes));
    
    // Create message and digest
    let message = b"Hello from the software ECDSA implementation!";
    println!("\nMessage: {:?}", core::str::from_utf8(message).unwrap());
    
    let mut hasher = Sha256::new();
    hasher.update(message);
    let digest = hasher.finalize().to_vec();
    println!("Digest: {}", hex_encode(&digest));
    
    // Sign the digest
    println!("\nSigning the digest...");
    let signature = ecdsa.sign(&private_key, digest.clone(), &mut rng)?;
    println!("✓ Signature created");
    
    let signature_bytes = serialize_to_bytes(&signature, 64);
    println!("Signature: {}", hex_encode(&signature_bytes));
    
    // Verify the signature
    println!("\nVerifying the signature...");
    ecdsa.verify(&public_key, digest, &signature)?;
    println!("✓ Signature verified successfully");
    
    // Test serialization roundtrip
    println!("\nTesting serialization roundtrip...");
    let private_serialized = serialize_to_bytes(&private_key, 32);
    let public_serialized = serialize_to_bytes(&public_key, 64);
    
    let restored_private = SoftwareP256PrivateKey::from_bytes(&private_serialized, Endian::Big)?;
    let restored_public = SoftwareP256PublicKey::from_bytes(&public_serialized, Endian::Big)?;
    
    // Test restored keys
    let test_digest = vec![0u8; 32]; // Simple test digest
    let test_signature = ecdsa.sign(&restored_private, test_digest.clone(), &mut rng)?;
    ecdsa.verify(&restored_public, test_digest, &test_signature)?;
    println!("✓ Serialization roundtrip successful");
    
    println!("\n🎉 Software ECDSA implementation completed successfully!");
    println!("✅ Key generation, signing, verification, and serialization all working");
    
    Ok(())
}
