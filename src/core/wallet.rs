use solana_sdk::{signature::Keypair, signer::Signer};
use std::sync::Arc;
use anyhow::{Result, Context};
use std::fs;

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
        // Read keypair from file
        let keypair_data = fs::read_to_string(path)
            .with_context(|| format!("Failed to read wallet file: {}", path))?;

        // Parse JSON array of bytes
        let bytes: Vec<u8> = serde_json::from_str(&keypair_data)
            .with_context(|| format!("Failed to parse wallet file as JSON: {}", path))?;

        // Create keypair from bytes
        let keypair = Keypair::from_bytes(&bytes)
            .with_context(|| "Failed to create keypair from bytes")?;

        Ok(Self::new(keypair))
    }

    pub fn pubkey(&self) -> solana_sdk::pubkey::Pubkey {
        self.keypair.pubkey()
    }

    pub fn sign_transaction(&self, tx: &mut solana_sdk::transaction::Transaction) -> Result<()> {
        tx.sign(&[&*self.keypair], tx.message.recent_blockhash);
        Ok(())
    }
}