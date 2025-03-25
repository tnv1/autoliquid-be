use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use sui_types::{base_types::SuiAddress, crypto::SuiKeyPair};

// Signer storage trait. This trait defines the interface for storing and retrieving signers.
// Required thread safety access.
pub trait Storage: Sync + Send {
    fn get_signer_by_address(&self, address: &SuiAddress) -> anyhow::Result<SuiKeyPair>;
    fn store_signer(&mut self, signer: SuiKeyPair) -> anyhow::Result<()>;
    fn get_all_addresses(&self) -> anyhow::Result<Vec<SuiAddress>>;
}

pub struct InmemoryStorage {
    signers: Arc<Mutex<HashMap<SuiAddress, SuiKeyPair>>>,
}

impl InmemoryStorage {
    pub fn new() -> Self {
        InmemoryStorage { signers: Arc::new(Mutex::new(HashMap::new())) }
    }
}

impl Storage for InmemoryStorage {
    fn get_signer_by_address(&self, address: &SuiAddress) -> anyhow::Result<SuiKeyPair> {
        let signers =
            self.signers.lock().map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?;

        match signers.get(address) {
            Some(signer) => Ok(signer.copy()),
            None => Err(anyhow::anyhow!("Signer not found")),
        }
    }

    fn store_signer(&mut self, signer: SuiKeyPair) -> anyhow::Result<()> {
        let address = SuiAddress::from(&signer.public());

        let mut signers =
            self.signers.lock().map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?;

        signers.insert(address, signer);
        Ok(())
    }

    fn get_all_addresses(&self) -> anyhow::Result<Vec<SuiAddress>> {
        let signers =
            self.signers.lock().map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?;

        Ok(signers.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use sui_types::crypto::{SuiKeyPair, get_key_pair};

    use super::*;

    #[test]
    fn test_store_and_get_signer() {
        let signer = SuiKeyPair::Ed25519(get_key_pair().1);
        let address = SuiAddress::from(&signer.public());
        let mut storage = InmemoryStorage::new();

        storage.store_signer(signer.copy()).unwrap();
        let fetched = storage.get_signer_by_address(&address).unwrap();

        assert_eq!(fetched.public(), signer.public());
    }

    #[test]
    fn test_get_signer_not_found() {
        let address = SuiAddress::random_for_testing_only();
        let storage = InmemoryStorage::new();

        let result = storage.get_signer_by_address(&address);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Signer not found");
    }
}
