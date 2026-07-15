use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use rand::RngCore;

use super::{keychain::MasterKey, storage_error::StorageError};

const ENVELOPE_VERSION: u8 = 1;
const NONCE_LENGTH: usize = 12;

/// AES-256-GCM envelope: version byte, 96-bit nonce, then authenticated ciphertext.
/// The nonce is fresh for every encryption operation and is not secret.
pub fn encrypt(key: &MasterKey, plaintext: &[u8]) -> Result<Vec<u8>, StorageError> {
    let mut nonce_bytes = [0_u8; NONCE_LENGTH];
    let mut random_source = rand::rngs::OsRng;
    random_source.fill_bytes(&mut nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(key.expose_to_crypto()).map_err(|_| StorageError::AuthenticationFailed)?;
    let ciphertext = cipher.encrypt(Nonce::from_slice(&nonce_bytes), plaintext).map_err(|_| StorageError::AuthenticationFailed)?;
    let mut envelope = Vec::with_capacity(1 + NONCE_LENGTH + ciphertext.len());
    envelope.push(ENVELOPE_VERSION);
    envelope.extend_from_slice(&nonce_bytes);
    envelope.extend_from_slice(&ciphertext);
    Ok(envelope)
}

pub fn decrypt(key: &MasterKey, envelope: &[u8]) -> Result<Vec<u8>, StorageError> {
    if envelope.len() <= 1 + NONCE_LENGTH || envelope[0] != ENVELOPE_VERSION {
        return Err(StorageError::UnsupportedEncryptedFormat);
    }
    let cipher = Aes256Gcm::new_from_slice(key.expose_to_crypto()).map_err(|_| StorageError::AuthenticationFailed)?;
    cipher.decrypt(Nonce::from_slice(&envelope[1..=NONCE_LENGTH]), &envelope[1 + NONCE_LENGTH..]).map_err(|_| StorageError::AuthenticationFailed)
}
