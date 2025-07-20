use crate::core::wallet::WalletManager;
use anyhow::{anyhow, Context, Result};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcSendTransactionConfig, RpcTransactionConfig},
};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    hash::Hash,
    message::Message,
    pubkey::Pubkey,
    signature::Signature,
    system_instruction,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct SolanaClient {
    rpc_client: Arc<RpcClient>,
    commitment: CommitmentConfig,
    timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub signature: Signature,
    pub slot: u64,
    pub confirmation_status: String,
    pub execution_time_ms: u64,
    pub fee_lamports: u64,
}

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub address: Pubkey,
    pub balance_lamports: u64,
    pub balance_sol: f64,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: u64,
}

impl SolanaClient {
    pub fn new(rpc_url: &str, commitment: CommitmentLevel, timeout_ms: u64) -> Result<Self> {
        let commitment_config = CommitmentConfig { commitment };
        let timeout = Duration::from_millis(timeout_ms);

        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            rpc_url.to_string(),
            commitment_config,
        ));

        info!("🌐 Solana client initialized: {}", rpc_url);
        info!("⚙️ Commitment level: {:?}", commitment);
        info!("⏱️ Timeout: {}ms", timeout_ms);

        Ok(Self {
            rpc_client,
            commitment: commitment_config,
            timeout,
        })
    }

    pub fn devnet() -> Result<Self> {
        Self::new(
            "https://api.devnet.solana.com",
            CommitmentLevel::Confirmed,
            5000,
        )
    }

    pub fn mainnet() -> Result<Self> {
        Self::new(
            "https://api.mainnet-beta.solana.com",
            CommitmentLevel::Confirmed,
            10000,
        )
    }

    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        let balance = self
            .rpc_client
            .get_balance_with_commitment(pubkey, self.commitment)
            .context("Failed to get balance")?
            .value;

        debug!(
            "💰 Balance for {}: {} lamports ({:.6} SOL)",
            pubkey,
            balance,
            balance as f64 / 1_000_000_000.0
        );

        Ok(balance)
    }

    pub async fn get_account_info(&self, pubkey: &Pubkey) -> Result<AccountInfo> {
        let account = self
            .rpc_client
            .get_account_with_commitment(pubkey, self.commitment)
            .context("Failed to get account info")?
            .value
            .ok_or_else(|| anyhow!("Account not found: {}", pubkey))?;

        let balance_lamports = account.lamports;
        let balance_sol = balance_lamports as f64 / 1_000_000_000.0;

        Ok(AccountInfo {
            address: *pubkey,
            balance_lamports,
            balance_sol,
            owner: account.owner,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        })
    }

    pub async fn get_recent_blockhash(&self) -> Result<Hash> {
        let (blockhash, _) = self
            .rpc_client
            .get_latest_blockhash_with_commitment(self.commitment)
            .context("Failed to get recent blockhash")?;

        debug!("🔗 Recent blockhash: {}", blockhash);
        Ok(blockhash)
    }

    pub async fn send_transaction(
        &self,
        wallet: &WalletManager,
        instructions: Vec<solana_sdk::instruction::Instruction>,
    ) -> Result<TransactionResult> {
        let start_time = Instant::now();

        // Get recent blockhash
        let recent_blockhash = self.get_recent_blockhash().await?;

        // Create message
        let message = Message::new(&instructions, Some(&wallet.pubkey()));

        // Create transaction
        let mut transaction = Transaction::new_unsigned(message);
        transaction.sign(&[wallet.keypair()], recent_blockhash);

        info!("📤 Sending transaction: {}", transaction.signatures[0]);

        // Send transaction
        let config = RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(self.commitment.commitment),
            encoding: Some(UiTransactionEncoding::Base64),
            max_retries: Some(3),
            min_context_slot: None,
        };

        let signature = self
            .rpc_client
            .send_transaction_with_config(&transaction, config)
            .context("Failed to send transaction")?;

        // Wait for confirmation
        let confirmation_result = self.wait_for_confirmation(&signature).await?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        info!(
            "✅ Transaction confirmed: {} ({}ms)",
            signature, execution_time
        );

        Ok(TransactionResult {
            signature,
            slot: confirmation_result.slot,
            confirmation_status: confirmation_result.status,
            execution_time_ms: execution_time,
            fee_lamports: confirmation_result.fee_lamports,
        })
    }

    pub async fn send_sol(
        &self,
        wallet: &WalletManager,
        to: &Pubkey,
        amount_lamports: u64,
    ) -> Result<TransactionResult> {
        let instruction = system_instruction::transfer(&wallet.pubkey(), to, amount_lamports);

        info!(
            "💸 Sending {} lamports ({:.6} SOL) from {} to {}",
            amount_lamports,
            amount_lamports as f64 / 1_000_000_000.0,
            wallet.pubkey(),
            to
        );

        self.send_transaction(wallet, vec![instruction]).await
    }

    pub async fn send_sol_with_wallet(
        &self,
        wallet: &crate::core::wallet::Wallet,
        to: &Pubkey,
        amount_lamports: u64,
    ) -> Result<TransactionResult> {
        use solana_sdk::{system_instruction, message::Message, transaction::Transaction};

        let instruction = system_instruction::transfer(&wallet.pubkey(), to, amount_lamports);

        info!("💸 Sending {} lamports ({:.6} SOL) from {} to {}",
              amount_lamports,
              amount_lamports as f64 / 1_000_000_000.0,
              wallet.pubkey(),
              to);

        // Get recent blockhash
        let recent_blockhash = self.get_recent_blockhash().await?;

        // Create message
        let message = Message::new(&[instruction], Some(&wallet.pubkey()));

        // Create transaction
        let mut transaction = Transaction::new_unsigned(message);
        transaction.sign(&[wallet.keypair()], recent_blockhash);

        info!("📤 Sending transaction: {}", transaction.signatures[0]);

        // Send transaction
        let config = solana_client::rpc_config::RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(self.commitment.commitment),
            encoding: Some(UiTransactionEncoding::Base64),
            max_retries: Some(3),
            min_context_slot: None,
        };

        let signature = self.rpc_client
            .send_transaction_with_config(&transaction, config)
            .context("Failed to send transaction")?;

        // Wait for confirmation
        let start_time = std::time::Instant::now();
        let confirmation_result = self.wait_for_confirmation(&signature).await?;
        let execution_time = start_time.elapsed().as_millis() as u64;

        info!(
            "✅ Transaction confirmed: {} ({}ms)",
            signature, execution_time
        );

        Ok(TransactionResult {
            signature,
            slot: confirmation_result.slot,
            confirmation_status: confirmation_result.status,
            execution_time_ms: execution_time,
            fee_lamports: confirmation_result.fee_lamports,
        })
    }

    async fn wait_for_confirmation(&self, signature: &Signature) -> Result<ConfirmationResult> {
        let start_time = Instant::now();
        let max_wait_time = Duration::from_secs(30);

        loop {
            if start_time.elapsed() > max_wait_time {
                return Err(anyhow!("Transaction confirmation timeout: {}", signature));
            }

            match self.get_transaction_status(signature).await {
                Ok(Some(status)) => {
                    if let Some(confirmation_status) = status.confirmation_status {
                        match confirmation_status.as_str() {
                            "finalized" | "confirmed" => {
                                return Ok(ConfirmationResult {
                                    slot: status.slot,
                                    status: confirmation_status,
                                    fee_lamports: status.fee_lamports.unwrap_or(5000),
                                });
                            }
                            "processed" => {
                                debug!(
                                    "⏳ Transaction processed, waiting for confirmation: {}",
                                    signature
                                );
                            }
                            _ => {
                                warn!("❓ Unknown confirmation status: {}", confirmation_status);
                            }
                        }
                    }
                }
                Ok(None) => {
                    debug!("⏳ Transaction not found yet: {}", signature);
                }
                Err(e) => {
                    warn!("⚠️ Error checking transaction status: {}", e);
                }
            }

            sleep(Duration::from_millis(500)).await;
        }
    }

    async fn get_transaction_status(
        &self,
        signature: &Signature,
    ) -> Result<Option<TransactionStatusInfo>> {
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: Some(self.commitment),
            max_supported_transaction_version: Some(0),
        };

        match self
            .rpc_client
            .get_transaction_with_config(signature, config)
        {
            Ok(transaction) => {
                let slot = transaction.slot;
                let meta = transaction.transaction.meta.as_ref();
                let fee_lamports = meta.map(|m| m.fee).unwrap_or(0);

                // Determine confirmation status based on slot and commitment
                let confirmation_status = match self.commitment.commitment {
                    CommitmentLevel::Finalized => "finalized",
                    CommitmentLevel::Confirmed => "confirmed",
                    CommitmentLevel::Processed => "processed",
                    _ => "confirmed",
                };

                Ok(Some(TransactionStatusInfo {
                    slot,
                    confirmation_status: Some(confirmation_status.to_string()),
                    fee_lamports: Some(fee_lamports),
                }))
            }
            Err(_) => Ok(None),
        }
    }

    pub async fn simulate_transaction(
        &self,
        wallet: &WalletManager,
        instructions: Vec<solana_sdk::instruction::Instruction>,
    ) -> Result<SimulationResult> {
        let recent_blockhash = self.get_recent_blockhash().await?;
        let message = Message::new(&instructions, Some(&wallet.pubkey()));
        let transaction = Transaction::new_unsigned(message);

        let simulation = self
            .rpc_client
            .simulate_transaction_with_config(
                &transaction,
                solana_client::rpc_config::RpcSimulateTransactionConfig {
                    sig_verify: false,
                    replace_recent_blockhash: true,
                    commitment: Some(self.commitment),
                    encoding: Some(UiTransactionEncoding::Base64),
                    accounts: None,
                    min_context_slot: None,
                    inner_instructions: false,
                },
            )
            .context("Failed to simulate transaction")?;

        let result = simulation.value;

        Ok(SimulationResult {
            success: result.err.is_none(),
            error: result.err.map(|e| format!("{:?}", e)),
            logs: result.logs.unwrap_or_default(),
            units_consumed: result.units_consumed.unwrap_or(0),
        })
    }
}

#[derive(Debug)]
struct ConfirmationResult {
    slot: u64,
    status: String,
    fee_lamports: u64,
}

#[derive(Debug)]
struct TransactionStatusInfo {
    slot: u64,
    confirmation_status: Option<String>,
    fee_lamports: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub success: bool,
    pub error: Option<String>,
    pub logs: Vec<String>,
    pub units_consumed: u64,
}
