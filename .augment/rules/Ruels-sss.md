---
type: "agent_requested"
description: "rules "
---
# 🥷 SOLANA HFT NINJA - USER GUIDELINES FOR AUGMENT

## 🎯 **CORE INTERACTION RULES**

### **Rule 1: Context-First Communication**
- Always reference existing codebase architecture when asking questions
- Mention specific files, functions, or modules using backticks (e.g., `src/strategies/protocol_specific.rs`)
- Build on previous conversations - avoid starting from scratch
- Example: "Based on the `RaydiumConfig` in `src/strategies/protocol_specific.rs`, optimize the liquidity sniping parameters"

### **Rule 2: MEV-Focused Development**
- Prioritize MEV strategies: sandwich attacks, arbitrage, liquidations, token sniping
- Always consider Solana-specific optimizations (compute units, Jito bundles, priority fees)
- Reference performance targets: <100ms latency, >85% success rate, 5% daily ROI
- Example: "Enhance the sandwich strategy to achieve <50ms execution time using Jito bundles"

### **Rule 3: Risk-First Approach**
- Every trading suggestion must include risk management
- Reference existing risk limits: 20% max daily loss, 45s position timeout, -10% circuit breaker
- Always validate against `RiskConfig` in `src/config/mod.rs`
- Example: "Add position sizing logic that respects the 1.6 SOL daily loss limit"