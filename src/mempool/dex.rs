//! DEX interaction detection for Raydium, Orca, and Jupiter

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// DEX program IDs on Solana
pub mod program_ids {
    use super::*;
    
    /// Raydium AMM V4
    pub const RAYDIUM_AMM_V4: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
    
    /// Raydium Concentrated Liquidity
    pub const RAYDIUM_CLMM: &str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";
    
    /// Orca Whirlpool
    pub const ORCA_WHIRLPOOL: &str = "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc";
    
    /// Orca Aquafarm
    pub const ORCA_AQUAFARM: &str = "82yxjeMsvaURa4MbZZ7WZZHfobirZYkH1zF8fmeGtyaQ";
    
    /// Jupiter V6
    pub const JUPITER_V6: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
    
    /// Jupiter Limit Order
    pub const JUPITER_LIMIT_ORDER: &str = "j1o2qRpjcyUwEvwtcfhEQefh773ZgjxcVRry7LDqg5X";
    
    /// Jupiter DCA
    pub const JUPITER_DCA: &str = "DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M";
}

/// DEX program identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DexProgram {
    RaydiumAmm,
    RaydiumClmm,
    OrcaWhirlpool,
    OrcaAquafarm,
    JupiterV6,
    JupiterLimitOrder,
    JupiterDca,
    Unknown,
}

impl DexProgram {
    pub fn from_pubkey(pubkey: &Pubkey) -> Self {
        let pubkey_str = pubkey.to_string();
        
        match pubkey_str.as_str() {
            program_ids::RAYDIUM_AMM_V4 => DexProgram::RaydiumAmm,
            program_ids::RAYDIUM_CLMM => DexProgram::RaydiumClmm,
            program_ids::ORCA_WHIRLPOOL => DexProgram::OrcaWhirlpool,
            program_ids::ORCA_AQUAFARM => DexProgram::OrcaAquafarm,
            program_ids::JUPITER_V6 => DexProgram::JupiterV6,
            program_ids::JUPITER_LIMIT_ORDER => DexProgram::JupiterLimitOrder,
            program_ids::JUPITER_DCA => DexProgram::JupiterDca,
            _ => DexProgram::Unknown,
        }
    }

    pub fn is_known_dex(&self) -> bool {
        !matches!(self, DexProgram::Unknown)
    }

    pub fn name(&self) -> &'static str {
        match self {
            DexProgram::RaydiumAmm => "Raydium AMM V4",
            DexProgram::RaydiumClmm => "Raydium CLMM",
            DexProgram::OrcaWhirlpool => "Orca Whirlpool",
            DexProgram::OrcaAquafarm => "Orca Aquafarm",
            DexProgram::JupiterV6 => "Jupiter V6",
            DexProgram::JupiterLimitOrder => "Jupiter Limit Order",
            DexProgram::JupiterDca => "Jupiter DCA",
            DexProgram::Unknown => "Unknown",
        }
    }
}

/// Liquidity zone information detected from transaction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LiquidityZone {
    pub dex: DexProgram,
    pub pool_address: Pubkey,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub price: f64,
    pub timestamp: u64,
    pub slot: u64,
}

/// DEX interaction detected in transaction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DexInteraction {
    pub program: DexProgram,
    pub instruction_type: InstructionType,
    pub accounts: Vec<Pubkey>,
    pub data: Vec<u8>,
    pub liquidity_zone: Option<LiquidityZone>,
}

/// Types of DEX instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InstructionType {
    Swap,
    AddLiquidity,
    RemoveLiquidity,
    CreatePool,
    UpdatePool,
    Unknown,
}

/// Detect DEX interactions from transaction instructions
pub fn detect_dex_interactions(
    instructions: &[solana_sdk::instruction::CompiledInstruction],
    account_keys: &[solana_sdk::pubkey::Pubkey],
) -> Vec<DexInteraction> {
    let mut interactions = Vec::new();

    for instruction in instructions {
        if let Some(program_id) = account_keys.get(instruction.program_id_index as usize) {
            let dex_program = DexProgram::from_pubkey(program_id);
            
            if dex_program.is_known_dex() {
                let instruction_type = parse_instruction_type(&instruction.data, &dex_program);
                
                let accounts: Vec<Pubkey> = instruction
                    .accounts
                    .iter()
                    .filter_map(|&idx| account_keys.get(idx as usize).copied())
                    .collect();

                let interaction = DexInteraction {
                    program: dex_program,
                    instruction_type,
                    accounts,
                    data: instruction.data.clone(),
                    liquidity_zone: None, // Will be populated by parser
                };

                interactions.push(interaction);
            }
        }
    }

    interactions
}

/// Parse instruction type from instruction data
fn parse_instruction_type(data: &[u8], dex: &DexProgram) -> InstructionType {
    if data.is_empty() {
        return InstructionType::Unknown;
    }

    match dex {
        DexProgram::RaydiumAmm => parse_raydium_instruction(data),
        DexProgram::RaydiumClmm => parse_raydium_clmm_instruction(data),
        DexProgram::OrcaWhirlpool => parse_orca_instruction(data),
        DexProgram::OrcaAquafarm => parse_orca_aquafarm_instruction(data),
        DexProgram::JupiterV6 => parse_jupiter_instruction(data),
        DexProgram::JupiterLimitOrder => parse_jupiter_limit_order_instruction(data),
        DexProgram::JupiterDca => parse_jupiter_dca_instruction(data),
        DexProgram::Unknown => InstructionType::Unknown,
    }
}

fn parse_raydium_instruction(data: &[u8]) -> InstructionType {
    match data.get(0) {
        Some(9) => InstructionType::Swap,
        Some(3) => InstructionType::AddLiquidity,
        Some(4) => InstructionType::RemoveLiquidity,
        _ => InstructionType::Unknown,
    }
}

fn parse_raydium_clmm_instruction(data: &[u8]) -> InstructionType {
    match data.get(0) {
        Some(43) => InstructionType::Swap,
        Some(5) => InstructionType::AddLiquidity,
        Some(9) => InstructionType::RemoveLiquidity,
        _ => InstructionType::Unknown,
    }
}

fn parse_orca_instruction(data: &[u8]) -> InstructionType {
    match data.get(0) {
        Some(248) => InstructionType::Swap,
        Some(242) => InstructionType::AddLiquidity,
        Some(243) => InstructionType::RemoveLiquidity,
        _ => InstructionType::Unknown,
    }
}

fn parse_orca_aquafarm_instruction(data: &[u8]) -> InstructionType {
    match data.get(0) {
        Some(1) => InstructionType::AddLiquidity,
        Some(2) => InstructionType::RemoveLiquidity,
        _ => InstructionType::Unknown,
    }
}

fn parse_jupiter_instruction(data: &[u8]) -> InstructionType {
    match data.get(0) {
        Some(1) => InstructionType::Swap,
        _ => InstructionType::Unknown,
    }
}

fn parse_jupiter_limit_order_instruction(data: &[u8]) -> InstructionType {
    match data.get(0) {
        Some(0) => InstructionType::AddLiquidity,
        Some(1) => InstructionType::RemoveLiquidity,
        _ => InstructionType::Unknown,
    }
}

fn parse_jupiter_dca_instruction(data: &[u8]) -> InstructionType {
    match data.get(0) {
        Some(0) => InstructionType::AddLiquidity,
        Some(1) => InstructionType::RemoveLiquidity,
        _ => InstructionType::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_dex_program_detection() {
        let raydium_pubkey = Pubkey::from_str(program_ids::RAYDIUM_AMM_V4).unwrap();
        assert_eq!(DexProgram::from_pubkey(&raydium_pubkey), DexProgram::RaydiumAmm);

        let orca_pubkey = Pubkey::from_str(program_ids::ORCA_WHIRLPOOL).unwrap();
        assert_eq!(DexProgram::from_pubkey(&orca_pubkey), DexProgram::OrcaWhirlpool);

        let jupiter_pubkey = Pubkey::from_str(program_ids::JUPITER_V6).unwrap();
        assert_eq!(DexProgram::from_pubkey(&jupiter_pubkey), DexProgram::JupiterV6);

        let unknown_pubkey = Pubkey::new_unique();
        assert_eq!(DexProgram::from_pubkey(&unknown_pubkey), DexProgram::Unknown);
    }

    #[test]
    fn test_instruction_type_parsing() {
        // Test Raydium swap instruction
        let raydium_swap_data = vec![9, 0, 0, 0];
        assert_eq!(
            parse_instruction_type(&raydium_swap_data, &DexProgram::RaydiumAmm),
            InstructionType::Swap
        );

        // Test Orca add liquidity
        let orca_add_data = vec![242, 0, 0, 0];
        assert_eq!(
            parse_instruction_type(&orca_add_data, &DexProgram::OrcaWhirlpool),
            InstructionType::AddLiquidity
        );
    }
}