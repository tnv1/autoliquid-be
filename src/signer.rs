use std::collections::HashMap;

use sui_types::base_types::SuiAddress;
use sui_types::crypto::SuiKeyPair;

pub trait Storage: Sync + Send {
    fn get_signer_by_address(&self, address: &SuiAddress) -> anyhow::Result<SuiKeyPair>;
    fn store_signer(&mut self, signer: SuiKeyPair) -> anyhow::Result<()>;
}

pub struct SuiStorage {
    signers: HashMap<SuiAddress, SuiKeyPair>,
}

impl Storage for SuiStorage {
    fn get_signer_by_address(&self, address: &SuiAddress) -> anyhow::Result<SuiKeyPair> {
        match self.signers.get(address) {
            Some(signer) => Ok(signer.copy()),
            None => Err(anyhow::anyhow!("Signer not found")),
        }
    }

    fn store_signer(&mut self, signer: SuiKeyPair) -> anyhow::Result<()> {
        let address = SuiAddress::from(&signer.public());
        self.signers.insert(address, signer);
        Ok(())
    }
}
