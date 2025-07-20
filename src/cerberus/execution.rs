use anyhow::{Result, anyhow};
use solana_sdk::{
    transaction::Transaction,
    signature::{Keypair, Signer},
    pubkey::Pubkey,
    instruction::Instruction,
    system_instruction,
};
use std::sync::Arc;
use std::str::FromStr;
use tracing::{debug, info};
use crate::cerberus::{PositionState, RpcManager};
use base64::{Engine as _, engine::general_purpose};

/// Cerberus execution engine for Jito bundles
pub struct CerberusExecutor {
    rpc_manager: Arc<RpcManager>,
    jito_endpoint: String,
    wallet_keypair: Arc<Keypair>,
    tip_account: Pubkey,
}

impl CerberusExecutor {
    /// Create new executor
    pub async fn new(
        rpc_manager: Arc<RpcManager>,
        jito_endpoint: &str,
    ) -> Result<Self> {
        info!("⚡ Initializing Cerberus Executor");
        
        // Load wallet keypair from environment or file
        let wallet_keypair = Arc::new(load_wallet_keypair()?);
        
        // Jito tip account (mainnet)
        let tip_account = Pubkey::from_str("96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5")?;
        
        info!("✅ Executor initialized");
        info!("💼 Wallet: {}", wallet_keypair.pubkey());
        info!("🎯 Jito endpoint: {}", jito_endpoint);
        
        Ok(Self {
            rpc_manager,
            jito_endpoint: jito_endpoint.to_string(),
            wallet_keypair,
            tip_account,
        })
    }

    /// Execute sell order via Jito bundle
    pub async fn execute_sell(&self, position: &PositionState, reason: &str) -> Result<String> {
        info!("💰 Executing SELL for {} - Reason: {}", position.mint, reason);
        
        // Build sell transaction
        let sell_tx = self.build_sell_transaction(position).await?;
        
        // Create Jito bundle with tip
        let bundle = self.create_jito_bundle(vec![sell_tx], position.position_size_sol).await?;
        
        // Send bundle
        let bundle_id = self.send_jito_bundle(bundle).await?;
        
        info!("✅ Sell bundle sent: {}", bundle_id);
        Ok(bundle_id)
    }

    /// Execute buy more order via Jito bundle
    pub async fn execute_buy_more(&self, position: &PositionState, amount_sol: f64) -> Result<String> {
        info!("📈 Executing BUY MORE for {} - Amount: {} SOL", position.mint, amount_sol);
        
        // Build buy transaction
        let buy_tx = self.build_buy_transaction(position, amount_sol).await?;
        
        // Create Jito bundle with tip
        let bundle = self.create_jito_bundle(vec![buy_tx], amount_sol).await?;
        
        // Send bundle
        let bundle_id = self.send_jito_bundle(bundle).await?;
        
        info!("✅ Buy more bundle sent: {}", bundle_id);
        Ok(bundle_id)
    }

    /// Build sell transaction (Jupiter swap)
    async fn build_sell_transaction(&self, position: &PositionState) -> Result<Transaction> {
        debug!("🔨 Building sell transaction for {}", position.mint);
        
        // Get recent blockhash
        let recent_blockhash = self.rpc_manager.get_recent_blockhash().await?;
        
        // For now, create a placeholder transaction
        // In production, this would integrate with Jupiter API to build the actual swap
        let instruction = self.build_jupiter_sell_instruction(position).await?;
        
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.wallet_keypair.pubkey()),
            &[&*self.wallet_keypair],
            recent_blockhash,
        );
        
        Ok(transaction)
    }

    /// Build buy transaction (Jupiter swap)
    async fn build_buy_transaction(&self, position: &PositionState, amount_sol: f64) -> Result<Transaction> {
        debug!("🔨 Building buy transaction for {} - {} SOL", position.mint, amount_sol);
        
        // Get recent blockhash
        let recent_blockhash = self.rpc_manager.get_recent_blockhash().await?;
        
        // For now, create a placeholder transaction
        // In production, this would integrate with Jupiter API to build the actual swap
        let instruction = self.build_jupiter_buy_instruction(position, amount_sol).await?;
        
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.wallet_keypair.pubkey()),
            &[&*self.wallet_keypair],
            recent_blockhash,
        );
        
        Ok(transaction)
    }

    /// Build Jupiter sell instruction (placeholder)
    async fn build_jupiter_sell_instruction(&self, position: &PositionState) -> Result<Instruction> {
        // TODO: Integrate with Jupiter API
        // This is a placeholder - in production you would:
        // 1. Call Jupiter quote API
        // 2. Get swap instruction
        // 3. Return the instruction
        
        debug!("🪐 Building Jupiter sell instruction for {}", position.mint);
        
        // Placeholder: simple transfer instruction
        Ok(system_instruction::transfer(
            &self.wallet_keypair.pubkey(),
            &self.wallet_keypair.pubkey(),
            1000, // 0.000001 SOL placeholder
        ))
    }

    /// Build Jupiter buy instruction (placeholder)
    async fn build_jupiter_buy_instruction(&self, position: &PositionState, amount_sol: f64) -> Result<Instruction> {
        // TODO: Integrate with Jupiter API
        debug!("🪐 Building Jupiter buy instruction for {} - {} SOL", position.mint, amount_sol);
        
        // Placeholder: simple transfer instruction
        Ok(system_instruction::transfer(
            &self.wallet_keypair.pubkey(),
            &self.wallet_keypair.pubkey(),
            (amount_sol * 1_000_000_000.0) as u64, // Convert SOL to lamports
        ))
    }

    /// Create Jito bundle with tip transaction
    async fn create_jito_bundle(&self, mut transactions: Vec<Transaction>, trade_amount_sol: f64) -> Result<Vec<Transaction>> {
        debug!("📦 Creating Jito bundle with {} transactions", transactions.len());
        
        // Calculate dynamic tip based on trade size
        let tip_lamports = self.calculate_dynamic_tip(trade_amount_sol);
        
        // Create tip transaction
        let tip_tx = self.create_tip_transaction(tip_lamports).await?;
        
        // Add tip transaction to the beginning of the bundle
        transactions.insert(0, tip_tx);
        
        debug!("💰 Bundle created with {} lamports tip", tip_lamports);
        Ok(transactions)
    }

    /// Calculate dynamic tip based on trade size and market conditions
    fn calculate_dynamic_tip(&self, trade_amount_sol: f64) -> u64 {
        // Base tip: 0.001 SOL (1,000,000 lamports)
        let base_tip = 1_000_000u64;
        
        // Scale tip based on trade size (0.01% of trade)
        let trade_based_tip = (trade_amount_sol * 1_000_000_000.0 * 0.0001) as u64;
        
        // Use the higher of base tip or trade-based tip
        std::cmp::max(base_tip, trade_based_tip)
    }

    /// Create tip transaction for Jito
    async fn create_tip_transaction(&self, tip_lamports: u64) -> Result<Transaction> {
        let recent_blockhash = self.rpc_manager.get_recent_blockhash().await?;
        
        let tip_instruction = system_instruction::transfer(
            &self.wallet_keypair.pubkey(),
            &self.tip_account,
            tip_lamports,
        );
        
        let tip_transaction = Transaction::new_signed_with_payer(
            &[tip_instruction],
            Some(&self.wallet_keypair.pubkey()),
            &[&*self.wallet_keypair],
            recent_blockhash,
        );
        
        Ok(tip_transaction)
    }

    /// Send Jito bundle
    async fn send_jito_bundle(&self, bundle: Vec<Transaction>) -> Result<String> {
        debug!("🚀 Sending Jito bundle with {} transactions", bundle.len());
        
        // Convert transactions to base64
        let bundle_data: Vec<String> = bundle
            .iter()
            .map(|tx| general_purpose::STANDARD.encode(&bincode::serialize(tx).unwrap()))
            .collect();
        
        // Create Jito bundle request
        let bundle_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [bundle_data]
        });
        
        // Send to Jito
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/api/v1/bundles", self.jito_endpoint))
            .header("Content-Type", "application/json")
            .json(&bundle_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            if let Some(bundle_id) = result["result"].as_str() {
                Ok(bundle_id.to_string())
            } else {
                Err(anyhow!("No bundle ID in response: {:?}", result))
            }
        } else {
            let error_text = response.text().await?;
            Err(anyhow!("Jito bundle failed: {}", error_text))
        }
    }

    /// Get wallet balance
    pub async fn get_wallet_balance(&self) -> Result<f64> {
        let balance_lamports = self.rpc_manager.get_account_balance(&self.wallet_keypair.pubkey()).await?;
        Ok(balance_lamports as f64 / 1_000_000_000.0) // Convert to SOL
    }

    /// Simulate transaction before sending
    pub async fn simulate_transaction(&self, transaction: &Transaction) -> Result<bool> {
        // TODO: Implement transaction simulation
        // This would use RPC simulate_transaction to check if the transaction would succeed
        debug!("🧪 Simulating transaction");
        Ok(true) // Placeholder
    }
}

/// Load wallet keypair from environment or file
fn load_wallet_keypair() -> Result<Keypair> {
    // Try to load from environment variable first
    if let Ok(private_key_json) = std::env::var("SOLANA_PRIVATE_KEY") {
        let private_key_bytes: Vec<u8> = serde_json::from_str(&private_key_json)?;
        return Ok(Keypair::from_bytes(&private_key_bytes)?);
    }
    
    // Try to load from default Solana CLI location
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let keypair_path = format!("{}/.config/solana/id.json", home_dir);
    
    if std::path::Path::new(&keypair_path).exists() {
        let keypair_data = std::fs::read_to_string(&keypair_path)?;
        let private_key_bytes: Vec<u8> = serde_json::from_str(&keypair_data)?;
        return Ok(Keypair::from_bytes(&private_key_bytes)?);
    }
    
    Err(anyhow!("No wallet keypair found. Set SOLANA_PRIVATE_KEY environment variable or ensure ~/.config/solana/id.json exists"))
}
