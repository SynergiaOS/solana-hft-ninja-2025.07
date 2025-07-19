//! DEX Detection Engine
//!
//! Advanced detection and parsing of DEX transactions across multiple protocols

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::debug;

/// Supported DEX protocols
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DexProtocol {
    Raydium,
    Orca,
    Jupiter,
    PumpFun,
    Serum,
    Meteora,
    Lifinity,
    Aldrin,
    Saber,
    Unknown(String),
}

impl FromStr for DexProtocol {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "raydium" => Ok(DexProtocol::Raydium),
            "orca" => Ok(DexProtocol::Orca),
            "jupiter" => Ok(DexProtocol::Jupiter),
            "pump.fun" | "pumpfun" => Ok(DexProtocol::PumpFun),
            "serum" => Ok(DexProtocol::Serum),
            "meteora" => Ok(DexProtocol::Meteora),
            "lifinity" => Ok(DexProtocol::Lifinity),
            "aldrin" => Ok(DexProtocol::Aldrin),
            "saber" => Ok(DexProtocol::Saber),
            _ => Ok(DexProtocol::Unknown(s.to_string())),
        }
    }
}

/// DEX transaction type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DexTransactionType {
    Swap {
        amount_in: u64,
        amount_out: u64,
        token_in: String,
        token_out: String,
        slippage_bps: Option<u64>,
    },
    AddLiquidity {
        token_a: String,
        token_b: String,
        amount_a: u64,
        amount_b: u64,
    },
    RemoveLiquidity {
        token_a: String,
        token_b: String,
        liquidity_amount: u64,
    },
    CreatePool {
        token_a: String,
        token_b: String,
        initial_price: Option<f64>,
    },
}

/// Detected DEX transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexTransaction {
    pub signature: String,
    pub protocol: DexProtocol,
    pub transaction_type: DexTransactionType,
    pub user: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub priority_fee: Option<u64>,
    pub compute_units: Option<u64>,
}

/// DEX program IDs for detection
pub struct DexProgramIds {
    programs: HashMap<String, DexProtocol>,
}

impl Default for DexProgramIds {
    fn default() -> Self {
        let mut programs = HashMap::new();

        // Raydium
        programs.insert(
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
            DexProtocol::Raydium,
        );
        programs.insert(
            "5quBtoiQqxF9Jv6KYKctB59NT3gtJD2Y65kdnB1Uev3h".to_string(),
            DexProtocol::Raydium,
        );

        // Orca
        programs.insert(
            "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP".to_string(),
            DexProtocol::Orca,
        );
        programs.insert(
            "DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1".to_string(),
            DexProtocol::Orca,
        );

        // Jupiter
        programs.insert(
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string(),
            DexProtocol::Jupiter,
        );
        programs.insert(
            "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB".to_string(),
            DexProtocol::Jupiter,
        );

        // Pump.fun
        programs.insert(
            "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_string(),
            DexProtocol::PumpFun,
        );

        // Serum
        programs.insert(
            "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string(),
            DexProtocol::Serum,
        );

        // Meteora
        programs.insert(
            "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB".to_string(),
            DexProtocol::Meteora,
        );

        // Lifinity
        programs.insert(
            "EewxydAPCCVuNEyrVN68PuSYdQ7wKn27V9Gjeoi8dy3S".to_string(),
            DexProtocol::Lifinity,
        );

        Self { programs }
    }
}

impl DexProgramIds {
    /// Get DEX protocol by program ID
    pub fn get_protocol(&self, program_id: &str) -> Option<&DexProtocol> {
        self.programs.get(program_id)
    }

    /// Check if program ID is a known DEX
    pub fn is_dex_program(&self, program_id: &str) -> bool {
        self.programs.contains_key(program_id)
    }
}

/// DEX detection engine
pub struct DexDetector {
    program_ids: DexProgramIds,
}

impl Default for DexDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DexDetector {
    /// Create new DEX detector
    pub fn new() -> Self {
        Self {
            program_ids: DexProgramIds::default(),
        }
    }

    /// Detect if transaction is a DEX transaction
    pub fn detect_dex_transaction(&self, transaction: &Value) -> Option<DexTransaction> {
        // Extract transaction data
        let tx_data = transaction.as_object()?;
        let message = tx_data.get("message")?.as_object()?;
        let instructions = message.get("instructions")?.as_array()?;
        let account_keys = message.get("accountKeys")?.as_array()?;

        // Check each instruction for DEX programs
        for instruction in instructions {
            let inst_obj = instruction.as_object()?;
            let program_id_index = inst_obj.get("programIdIndex")?.as_u64()? as usize;

            if program_id_index >= account_keys.len() {
                continue;
            }

            let program_id = account_keys[program_id_index].as_str()?;

            if let Some(protocol) = self.program_ids.get_protocol(program_id) {
                // Found DEX transaction, parse it
                if let Some(dex_tx) = self.parse_dex_transaction(protocol, instruction, tx_data) {
                    return Some(dex_tx);
                }
            }
        }

        None
    }

    /// Parse DEX transaction details
    fn parse_dex_transaction(
        &self,
        protocol: &DexProtocol,
        instruction: &Value,
        tx_data: &serde_json::Map<String, Value>,
    ) -> Option<DexTransaction> {
        let signature = tx_data
            .get("signatures")?
            .as_array()?
            .first()?
            .as_str()?
            .to_string();

        // Extract basic transaction info
        let slot = 0; // Would be extracted from block data
        let block_time = None; // Would be extracted from block data
        let user = "unknown".to_string(); // Would be extracted from accounts

        // Parse instruction data based on protocol
        let transaction_type = match protocol {
            DexProtocol::Raydium => self.parse_raydium_instruction(instruction)?,
            DexProtocol::Orca => self.parse_orca_instruction(instruction)?,
            DexProtocol::Jupiter => self.parse_jupiter_instruction(instruction)?,
            DexProtocol::PumpFun => self.parse_pumpfun_instruction(instruction)?,
            _ => {
                debug!("Unsupported DEX protocol for parsing: {:?}", protocol);
                return None;
            }
        };

        Some(DexTransaction {
            signature,
            protocol: protocol.clone(),
            transaction_type,
            user,
            slot,
            block_time,
            priority_fee: None,
            compute_units: None,
        })
    }

    /// Parse Raydium instruction
    fn parse_raydium_instruction(&self, instruction: &Value) -> Option<DexTransactionType> {
        // Simplified parsing - in reality would decode instruction data
        let data = instruction.get("data")?.as_str()?;

        // Basic swap detection (would need proper instruction decoding)
        if data.starts_with("swap") || data.len() > 32 {
            Some(DexTransactionType::Swap {
                amount_in: 1000000, // Would be decoded from instruction
                amount_out: 950000, // Would be decoded from instruction
                token_in: "So11111111111111111111111111111111111111112".to_string(), // SOL
                token_out: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                slippage_bps: Some(100),
            })
        } else {
            None
        }
    }

    /// Parse Orca instruction
    fn parse_orca_instruction(&self, instruction: &Value) -> Option<DexTransactionType> {
        // Similar to Raydium but with Orca-specific logic
        let data = instruction.get("data")?.as_str()?;

        if data.len() > 16 {
            Some(DexTransactionType::Swap {
                amount_in: 500000,
                amount_out: 475000,
                token_in: "So11111111111111111111111111111111111111112".to_string(),
                token_out: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                slippage_bps: Some(50),
            })
        } else {
            None
        }
    }

    /// Parse Jupiter instruction
    fn parse_jupiter_instruction(&self, instruction: &Value) -> Option<DexTransactionType> {
        // Jupiter aggregator parsing
        let data = instruction.get("data")?.as_str()?;

        if data.len() > 20 {
            Some(DexTransactionType::Swap {
                amount_in: 2000000,
                amount_out: 1950000,
                token_in: "So11111111111111111111111111111111111111112".to_string(),
                token_out: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                slippage_bps: Some(25),
            })
        } else {
            None
        }
    }

    /// Parse Pump.fun instruction
    fn parse_pumpfun_instruction(&self, instruction: &Value) -> Option<DexTransactionType> {
        // Pump.fun meme token trading
        let data = instruction.get("data")?.as_str()?;

        if data.len() > 8 {
            Some(DexTransactionType::Swap {
                amount_in: 100000,
                amount_out: 95000,
                token_in: "So11111111111111111111111111111111111111112".to_string(),
                token_out: "meme_token_mint".to_string(), // Would be actual token mint
                slippage_bps: Some(500),                  // Higher slippage for meme tokens
            })
        } else {
            None
        }
    }
}

/// Create new DEX detector instance
pub fn create_dex_detector() -> DexDetector {
    DexDetector::new()
}
