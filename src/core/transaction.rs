use solana_sdk::{
    instruction::Instruction,
    message::Message,
    transaction::Transaction,
    pubkey::Pubkey,
};
use anyhow::Result;

pub struct TransactionBuilder {
    payer: Pubkey,
}

impl TransactionBuilder {
    pub fn new(payer: Pubkey) -> Self {
        Self { payer }
    }

    pub fn build_transaction(
        &self,
        instructions: Vec<Instruction>,
        recent_blockhash: solana_sdk::hash::Hash,
    ) -> Transaction {
        let message = Message::new(&instructions, Some(&self.payer));
        Transaction::new_unsigned(message)
    }
}