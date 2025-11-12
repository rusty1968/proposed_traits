use proposed_traits::{
    common::{FromBytes, ToBytes},
    digest::DigestAlgorithm,
    ecdsa::{Curve, EccPrivateKey, EcdsaKeyGen, EcdsaSign, EcdsaVerify, PrivateKeyForCurve, PubKeyForCurve, SignatureForCurve},
};

use proposed_traits::ecdsa::ErrorKind as EcdsaErrorKind;
use proposed_traits::ecdsa::ErrorType as EcdsaErrorType;
use proposed_traits::ecdsa::Error as EcdsaError;


use core::fmt::Debug;
use p384::{
    ecdsa::{Signature, SigningKey, VerifyingKey},
    EncodedPoint, Scalar, SecretKey,
};
use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha384};

// Endian enum
#[derive(Debug, Copy, Clone)]
pub enum Endian {
    BigEndian,
    LittleEndian,
}

// Scalar type (wraps p384::Scalar)
#[derive(Clone)]
pub struct Secp384r1Scalar(Scalar);

impl ToBytes for Secp384r1Scalar {
    fn to_bytes(&self) -> &[u8] {
        // p384::Scalar provides big-endian bytes
        self.0.to_bytes().as_slice()
    }
}

impl FromBytes for Secp384r1Scalar {
    type Error = Secp384r1Error;

    fn from_bytes(bytes: &[u8], endian: Endian) -> Result<Self, Self::Error> {
        if bytes.len() != 48 {
            return Err(Secp384r1Error::InvalidLength);
        }
        let mut array = [0u8; 48];
        array.copy_from_slice(bytes);
        if matches!(endian, Endian::LittleEndian) {
            array.reverse();
        }
        let scalar = Scalar::from_be_bytes(&array).map_err(|_| Secp384r1Error::InvalidValue)?;
        Ok(Secp384r1Scalar(scalar))
    }
}

// Private key (wraps p384::SecretKey)
pub struct Secp384r1PrivateKey(SecretKey);

impl<'a> EccPrivateKey<'a> for Secp384r1PrivateKey {
    fn zeroize(&mut self) {
        // p384::SecretKey implements zeroize
        self.0.zeroize();
    }
}

impl ToBytes for Secp384r1PrivateKey {
    fn to_bytes(&self) -> &[u8] {
        self.0.to_bytes().as_slice()
    }
}

impl FromBytes for Secp384r1PrivateKey {
    type Error = Secp384r1Error;

    fn from_bytes(bytes: &[u8], endian: Endian) -> Result<Self, Self::Error> {
        let scalar = Secp384r1Scalar::from_bytes(bytes, endian)?;
        let secret_key = SecretKey::from_bytes(&scalar.0.to_bytes())
            .map_err(|_| Secp384r1Error::InvalidValue)?;
        Ok(Secp384r1PrivateKey(secret_key))
    }
}

impl PrivateKeyForCurve<Secp384r1> for Secp384r1PrivateKey {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

// Public key (wraps p384::VerifyingKey)
pub struct Secp384r1PublicKey(VerifyingKey);

impl ToBytes for Secp384r1PublicKey {
    fn to_bytes(&self) -> &[u8] {
        // Uncompressed format (96 bytes: x || y)
        EncodedPoint::from(&self.0).as_bytes()
    }
}

impl FromBytes for Secp384r1PublicKey {
    type Error = Secp384r1Error;

    fn from_bytes(bytes: &[u8], _endian: Endian) -> Result<Self, Self::Error> {
        if bytes.len() != 97 && bytes.len() != 49 {
            // Uncompressed (97) or compressed (49)
            return Err(Secp384r1Error::InvalidLength);
        }
        let point = EncodedPoint::from_bytes(bytes).map_err(|_| Secp384r1Error::InvalidValue)?;
        let verifying_key =
            VerifyingKey::from_encoded_point(&point).map_err(|_| Secp384r1Error::InvalidValue)?;
        Ok(Secp384r1PublicKey(verifying_key))
    }
}

impl PubKeyForCurve<Secp384r1> for Secp384r1PublicKey {
    fn x(&self) -> &Secp384r1Scalar {
        // Extract x-coordinate (not directly exposed by p384; reconstruct from EncodedPoint)
        static mut X_BUFFER: Secp384r1Scalar = Secp384r1Scalar(Scalar::zero());
        let point = EncodedPoint::from(&self.0);
        let x_bytes = point.x().ok_or(()).unwrap(); // Safe for uncompressed points
        unsafe {
            X_BUFFER = Secp384r1Scalar(
                Scalar::from_be_bytes(x_bytes).unwrap(), // Validated during from_bytes
            );
            &X_BUFFER
        }
    }

    fn y(&self) -> &Secp384r1Scalar {
        static mut Y_BUFFER: Secp384r1Scalar = Secp384r1Scalar(Scalar::zero());
        let point = EncodedPoint::from(&self.0);
        let y_bytes = point.y().ok_or(()).unwrap(); // Safe for uncompressed points
        unsafe {
            Y_BUFFER = Secp384r1Scalar(Scalar::from_be_bytes(y_bytes).unwrap());
            &Y_BUFFER
        }
    }

    fn new(x: Secp384r1Scalar, y: Secp384r1Scalar) -> Self {
        let point = EncodedPoint::from_affine_coordinates(
            &x.0.to_bytes(),
            &y.0.to_bytes(),
            false, // Uncompressed
        );
        let verifying_key = VerifyingKey::from_encoded_point(&point).expect("Valid coordinates"); // In practice, validate
        Secp384r1PublicKey(verifying_key)
    }
}

// Signature (wraps p384::ecdsa::Signature)
pub struct Secp384r1Signature(Signature);

impl ToBytes for Secp384r1Signature {
    fn to_bytes(&self) -> &[u8] {
        // DER encoding or raw r || s (96 bytes)
        self.0.to_bytes().as_slice()
    }
}

impl FromBytes for Secp384r1Signature {
    type Error = Secp384r1Error;

    fn from_bytes(bytes: &[u8], _endian: Endian) -> Result<Self, Self::Error> {
        let signature =
            Signature::from_bytes(bytes).map_err(|_| Secp384r1Error::InvalidSignature)?;
        Ok(Secp384r1Signature(signature))
    }
}

impl SignatureForCurve<Secp384r1> for Secp384r1Signature {
    fn r(&self) -> &Secp384r1Scalar {
        static mut R_BUFFER: Secp384r1Scalar = Secp384r1Scalar(Scalar::zero());
        unsafe {
            R_BUFFER = Secp384r1Scalar(*self.0.r());
            &R_BUFFER
        }
    }

    fn s(&self) -> &Secp384r1Scalar {
        static mut S_BUFFER: Secp384r1Scalar = Secp384r1Scalar(Scalar::zero());
        unsafe {
            S_BUFFER = Secp384r1Scalar(*self.0.s());
            &S_BUFFER
        }
    }

    fn new(r: Secp384r1Scalar, s: Secp384r1Scalar) -> Self {
        let signature = Signature::from_scalars(r.0, s.0).expect("Valid scalars"); // In practice, validate
        Secp384r1Signature(signature)
    }
}

// Curve definition
pub struct Secp384r1;

impl Curve for Secp384r1 {
    type DigestType = Sha384Digest;
    type Scalar = Secp384r1Scalar;
    const DIGEST_LENGTH: usize = 48; // SHA-384 output
}

// Digest type for SHA-384
pub struct Sha384Digest([u8; 48]);

impl DigestAlgorithm for Sha384Digest {
    type DigestOutput = [u8; 48];

    fn digest(data: &[u8]) -> Self::DigestOutput {
        let mut hasher = Sha384::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

impl ToBytes for Sha384Digest {
    fn to_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl FromBytes for Sha384Digest {
    type Error = Secp384r1Error;

    fn from_bytes(bytes: &[u8], _endian: Endian) -> Result<Self, Self::Error> {
        if bytes.len() != Secp384r1::DIGEST_LENGTH {
            return Err(Secp384r1Error::InvalidLength);
        }
        let array: [u8; 48] = bytes
            .try_into()
            .map_err(|_| Secp384r1Error::InvalidLength)?;
        Ok(Sha384Digest(array))
    }
}
