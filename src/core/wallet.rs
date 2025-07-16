use solana_sdk::{signature::Keypair, signer::Signer};
use std::sync::Arc;
use anyhow::Result;

#[derive(Clone)]
pub struct WalletManager {
    keypair: Arc<Keypair>,
}

impl WalletManager {
    pub fn new(keypair: Keypair) -> Self {
        Self {
            keypair: Arc::new(keypair),
        }
    }

    pub fn from_file(path: &str) -> Result<Self> {
        // Placeholder for keypair loading
        Ok(Self::new(Keypair::new()))
    }

    pub fn pubkey(&self) -> solana_sdk::pubkey::Pubkey {
        self.keypair.pubkey()
    }

    pub fn sign_transaction(&self, tx: &mut solana_sdk::transaction::Transaction) -> Result<()> {
        tx.sign(&[&*self.keypair], tx.message.recent_blockhash);
        Ok(())
    }
}