use keyring::{Entry, Error as KeyringError};
use rand::RngCore;
use zeroize::Zeroizing;

use super::storage_error::StorageError;

const SERVICE_NAME: &str = "com.lifeos.desktop";
const ACCOUNT_NAME: &str = "document-encryption-key-v1";

/// Non-exportable AES-256 key material. This type intentionally has no accessors,
/// serialization, cloning, or Debug implementation.
pub struct MasterKey(Zeroizing<[u8; 32]>);

impl MasterKey {
    pub(crate) fn expose_to_crypto(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Loads or creates a random data-encryption key in the platform's secure credential store.
/// There is deliberately no file-based fallback: failing closed is safer than weakening key custody.
pub fn load_or_create_master_key() -> Result<MasterKey, StorageError> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME).map_err(|_| StorageError::KeychainUnavailable)?;

    match entry.get_secret() {
        Ok(bytes) => decode_key(bytes),
        Err(KeyringError::NoEntry) => create_key(entry),
        Err(_) => Err(StorageError::KeychainUnavailable),
    }
}

fn create_key(entry: Entry) -> Result<MasterKey, StorageError> {
    let mut bytes = [0_u8; 32];
    let mut random_source = rand::rngs::OsRng;
    random_source.fill_bytes(&mut bytes);
    entry.set_secret(&bytes).map_err(|_| StorageError::KeychainUnavailable)?;
    Ok(MasterKey(Zeroizing::new(bytes)))
}

fn decode_key(bytes: Vec<u8>) -> Result<MasterKey, StorageError> {
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| StorageError::InvalidKeychainKey)?;
    Ok(MasterKey(Zeroizing::new(bytes)))
}

#[cfg(test)]
mod tests {
    use zeroize::Zeroizing;

    use super::MasterKey;
    use crate::infrastructure::crypto;

    #[test]
    fn encrypted_payload_round_trips_and_rejects_tampering() {
        let key = MasterKey(Zeroizing::new([42_u8; 32]));
        let mut encrypted = crypto::encrypt(&key, b"private document").expect("encryption should succeed");
        assert_eq!(crypto::decrypt(&key, &encrypted).expect("decryption should succeed"), b"private document");
        let final_byte = encrypted.len() - 1;
        encrypted[final_byte] ^= 1;
        assert!(crypto::decrypt(&key, &encrypted).is_err());
    }
}
