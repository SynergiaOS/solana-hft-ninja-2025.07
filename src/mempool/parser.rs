//! Transaction parsing and analysis

use serde::{Deserialize, Serialize};
use solana_sdk::{message::VersionedMessage, pubkey::Pubkey, transaction::VersionedTransaction};

use crate::mempool::{dex::*, error::*, metrics::MempoolMetrics};

/// Parsed transaction with DEX interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    #[serde(with = "signature_serde")]
    pub signature: [u8; 64],
    pub account_keys: Vec<Pubkey>,
    pub instructions: Vec<ParsedInstruction>,
    pub dex_interactions: Vec<DexInteraction>,
    pub timestamp: u64,
    pub slot: u64,
}

/// Serde helper for signature serialization
mod signature_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(signature: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        signature.to_vec().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        vec.try_into()
            .map_err(|_| serde::de::Error::custom("Invalid signature length"))
    }
}

/// Parsed instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedInstruction {
    pub program_id_index: u8,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}

/// Transaction parser
#[derive(Clone)]
pub struct ZeroCopyParser {
    metrics: MempoolMetrics,
    max_memory_bytes: usize,
}

impl ZeroCopyParser {
    pub fn new(metrics: MempoolMetrics, max_memory_bytes: usize) -> Self {
        Self {
            metrics,
            max_memory_bytes,
        }
    }

    /// Parse transaction
    pub fn parse_transaction(
        &self,
        data: &[u8],
        timestamp: u64,
        slot: u64,
    ) -> Result<ParsedTransaction> {
        let _timer = self.metrics.processing_timer();

        // Check memory limit
        if data.len() > self.max_memory_bytes {
            return Err(MempoolError::MemoryLimitExceeded(
                self.max_memory_bytes / 1024 / 1024,
            ));
        }

        // Deserialize transaction
        let transaction = match self.deserialize_transaction(data) {
            Ok(tx) => tx,
            Err(e) => {
                self.metrics.increment_deserialization_errors();
                return Err(e);
            }
        };

        // Extract account keys and instructions
        let account_keys = self.extract_account_keys(&transaction)?;
        let instructions = self.extract_instructions(&transaction)?;

        // Convert to compiled instructions for DEX detection
        let compiled_instructions = self.get_compiled_instructions(&transaction);

        // Detect DEX interactions
        let dex_interactions = detect_dex_interactions(&compiled_instructions, &account_keys);

        // Update metrics
        self.metrics.increment_transactions_processed();
        self.metrics.add_bytes_received(data.len() as u64);

        if !dex_interactions.is_empty() {
            self.metrics.increment_dex_detections();
        }

        Ok(ParsedTransaction {
            signature: transaction.signatures[0]
                .as_ref()
                .try_into()
                .unwrap_or([0u8; 64]),
            account_keys,
            instructions,
            dex_interactions,
            timestamp,
            slot,
        })
    }

    /// Deserialize transaction
    fn deserialize_transaction(&self, data: &[u8]) -> Result<VersionedTransaction> {
        bincode::deserialize(data).map_err(MempoolError::from)
    }

    /// Extract account keys from transaction
    fn extract_account_keys(&self, transaction: &VersionedTransaction) -> Result<Vec<Pubkey>> {
        match &transaction.message {
            VersionedMessage::Legacy(message) => Ok(message.account_keys.clone()),
            VersionedMessage::V0(message) => {
                let keys = message.account_keys.clone();
                // For V0 messages, we'd need to resolve lookup tables
                // This is simplified for now - in production, resolve address lookup tables
                Ok(keys)
            }
        }
    }

    /// Extract instructions from transaction
    fn extract_instructions(
        &self,
        transaction: &VersionedTransaction,
    ) -> Result<Vec<ParsedInstruction>> {
        let instructions = match &transaction.message {
            VersionedMessage::Legacy(message) => &message.instructions,
            VersionedMessage::V0(message) => &message.instructions,
        };

        Ok(instructions
            .iter()
            .map(|ix| ParsedInstruction {
                program_id_index: ix.program_id_index,
                accounts: ix.accounts.clone(),
                data: ix.data.clone(),
            })
            .collect())
    }

    /// Get compiled instructions for DEX detection
    fn get_compiled_instructions(
        &self,
        transaction: &VersionedTransaction,
    ) -> Vec<solana_sdk::instruction::CompiledInstruction> {
        match &transaction.message {
            VersionedMessage::Legacy(message) => message.instructions.clone(),
            VersionedMessage::V0(message) => message.instructions.clone(),
        }
    }

    /// Check if transaction contains DEX interactions
    pub fn has_dex_interactions(&self, data: &[u8]) -> Result<bool> {
        let transaction = self.deserialize_transaction(data)?;
        let account_keys = self.extract_account_keys(&transaction)?;
        let compiled_instructions = self.get_compiled_instructions(&transaction);

        let has_dex = !detect_dex_interactions(&compiled_instructions, &account_keys).is_empty();
        Ok(has_dex)
    }

    /// Get transaction size without full deserialization
    pub fn get_transaction_size(&self, data: &[u8]) -> Result<usize> {
        Ok(data.len())
    }
}

/// Memory-efficient transaction buffer
pub struct TransactionBuffer {
    buffer: Vec<u8>,
    capacity: usize,
}

impl TransactionBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, data: &[u8]) -> Result<()> {
        if self.buffer.len() + data.len() > self.capacity {
            // Remove oldest transactions (simple FIFO)
            let remove_amount =
                (self.buffer.len() + data.len() - self.capacity).min(self.buffer.len());
            self.buffer.drain(0..remove_amount);
        }

        self.buffer.extend_from_slice(data);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::{
        signature::Keypair, signer::Signer, system_instruction, transaction::Transaction,
    };

    #[test]
    fn test_zero_copy_deserialization() {
        let metrics = MempoolMetrics::new();
        let parser = ZeroCopyParser::new(metrics, 16 * 1024 * 1024);

        // Create a simple transaction for testing with valid instruction
        let keypair = Keypair::new();
        // Create a transfer instruction (0 lamports to self) - minimal valid transaction
        let instruction = system_instruction::transfer(
            &keypair.pubkey(),
            &keypair.pubkey(),
            0, // 0 lamports transfer to self
        );
        let recent_blockhash = solana_sdk::hash::Hash::default();
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_blockhash,
        );
        let versioned = VersionedTransaction::from(transaction);

        let serialized = bincode::serialize(&versioned).unwrap();
        let parsed = parser.parse_transaction(&serialized, 0, 0).unwrap();

        assert!(!parsed.account_keys.is_empty());
        assert_eq!(parsed.instructions.len(), 1); // Now we have 1 transfer instruction
    }

    #[test]
    fn test_memory_limit() {
        let metrics = MempoolMetrics::new();
        let parser = ZeroCopyParser::new(metrics, 1024); // 1KB limit

        let large_data = vec![0u8; 2048];
        let result = parser.parse_transaction(&large_data, 0, 0);

        assert!(matches!(result, Err(MempoolError::MemoryLimitExceeded(0))));
    }

    #[test]
    fn test_transaction_buffer() {
        let mut buffer = TransactionBuffer::new(100);

        buffer.push(&[1, 2, 3]).unwrap();
        buffer.push(&[4, 5, 6]).unwrap();

        assert_eq!(buffer.len(), 6);

        // Test overflow handling
        buffer.push(&[7; 100]).unwrap();
        assert!(buffer.len() <= 100);
    }
}
