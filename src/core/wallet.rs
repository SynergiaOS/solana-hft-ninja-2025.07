use anyhow::{Context, Result};
use solana_sdk::{signature::Keypair, signer::Signer};
use std::fs;
use std::sync::Arc;

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
        let keypair =
            Keypair::from_bytes(&bytes).with_context(|| "Failed to create keypair from bytes")?;

        Ok(Self::new(keypair))
    }

    pub fn pubkey(&self) -> solana_sdk::pubkey::Pubkey {
        self.keypair.pubkey()
    }

    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    pub fn sign_transaction(&self, tx: &mut solana_sdk::transaction::Transaction) -> Result<()> {
        tx.sign(&[&*self.keypair], tx.message.recent_blockhash);
        Ok(())
    }
}

/// New unified wallet structure for refactored engine
pub struct Wallet {
    keypair: Arc<Keypair>,
}

impl Wallet {
    pub fn load(path: &str) -> Result<Self> {
        let keypair_data = fs::read_to_string(path)
            .with_context(|| format!("Failed to read wallet file: {}", path))?;

        let bytes: Vec<u8> = serde_json::from_str(&keypair_data)
            .with_context(|| format!("Failed to parse wallet file as JSON: {}", path))?;

        let keypair =
            Keypair::from_bytes(&bytes).with_context(|| "Failed to create keypair from bytes")?;

        Ok(Self {
            keypair: Arc::new(keypair),
        })
    }

    pub fn pubkey(&self) -> solana_sdk::pubkey::Pubkey {
        self.keypair.pubkey()
    }

    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }
}
