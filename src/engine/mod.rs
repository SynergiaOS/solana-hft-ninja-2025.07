use crate::mempool::router::{EventPriority, MempoolEvent, OpportunityType};
use crate::{config::Config, market::MarketData, strategy::Strategy};
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

pub struct Engine {
    config: Config,
    market_data: MarketData,
    strategy: Box<dyn Strategy>,
    dry_run: bool,
    mev_processor: MevProcessor,
}

/// MEV opportunity processor
pub struct MevProcessor {
    dry_run: bool,
    processed_count: u64,
    successful_count: u64,
}

/// MEV opportunity ready for execution
#[derive(Debug, Clone)]
pub struct MevOpportunity {
    pub opportunity_type: OpportunityType,
    pub estimated_profit_sol: f64,
    pub estimated_gas_cost: u64,
    pub execution_deadline_ns: u64,
    pub priority: EventPriority,
}

impl Engine {
    pub async fn new(config: Config, dry_run: bool) -> Result<Self> {
        let market_data = MarketData::new(&config.solana).await?;
        let strategy = crate::strategy::create_strategy(&config.strategy)?;
        let mev_processor = MevProcessor::new(dry_run);

        Ok(Self {
            config,
            market_data,
            strategy,
            dry_run,
            mev_processor,
        })
    }

    /// Run engine with mempool integration for real-time MEV
    pub async fn run_with_mempool(
        &mut self,
        mut mempool_rx: broadcast::Receiver<Arc<MempoolEvent>>,
    ) -> Result<()> {
        info!("Starting HFT Engine with mempool integration...");

        let mut last_strategy_run = Instant::now();
        let strategy_interval = Duration::from_millis(self.config.strategy.update_interval_ms);

        loop {
            tokio::select! {
                // HIGH PRIORITY: Real-time mempool events
                Ok(event) = mempool_rx.recv() => {
                    let start_time = Instant::now();

                    if let Some(opportunity) = self.analyze_mempool_event(&event).await {
                        match self.mev_processor.execute_opportunity(opportunity).await {
                            Ok(result) => {
                                let latency = start_time.elapsed();
                                info!(
                                    "MEV executed: {:?} in {:?} - Profit: {:.4} SOL",
                                    event.opportunity_type,
                                    latency,
                                    result.profit_sol
                                );
                            }
                            Err(e) => {
                                warn!("MEV execution failed: {}", e);
                            }
                        }
                    }
                }

                // LOW PRIORITY: Regular strategy execution
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    if last_strategy_run.elapsed() >= strategy_interval {
                        if let Err(e) = self.run_regular_strategy().await {
                            error!("Strategy execution error: {}", e);
                        }
                        last_strategy_run = Instant::now();
                    }
                }
            }
        }
    }

    /// Legacy run method for backward compatibility
    pub async fn run(&self) -> Result<()> {
        warn!("Running engine without mempool integration - MEV opportunities will be missed!");

        loop {
            if let Err(e) = self.run_regular_strategy().await {
                error!("Strategy execution error: {}", e);
            }

            tokio::time::sleep(Duration::from_millis(
                self.config.strategy.update_interval_ms,
            ))
            .await;
        }
    }

    /// Run regular trading strategy (non-MEV)
    async fn run_regular_strategy(&self) -> Result<()> {
        let market_snapshot = self.market_data.get_snapshot().await?;
        let orders = self.strategy.generate_orders(&market_snapshot).await?;

        for order in orders {
            if !self.dry_run {
                debug!("Executing regular order: {:?}", order);
                // TODO: Implement order execution
            }
        }

        Ok(())
    }

    /// Analyze mempool event for MEV opportunities
    async fn analyze_mempool_event(&self, event: &MempoolEvent) -> Option<MevOpportunity> {
        let deadline_ns = event.timestamp_ns + self.get_execution_deadline(event.priority);

        match &event.opportunity_type {
            OpportunityType::Sandwich {
                swap_amount_in,
                slippage_bps,
                ..
            } => {
                let estimated_profit =
                    self.calculate_sandwich_profit(*swap_amount_in, *slippage_bps);
                if estimated_profit > 0.01 {
                    // Minimum 0.01 SOL profit
                    Some(MevOpportunity {
                        opportunity_type: event.opportunity_type.clone(),
                        estimated_profit_sol: estimated_profit,
                        estimated_gas_cost: 10_000, // 0.00001 SOL
                        execution_deadline_ns: deadline_ns,
                        priority: event.priority,
                    })
                } else {
                    None
                }
            }

            OpportunityType::Arbitrage {
                profit_bps,
                optimal_amount,
                ..
            } => {
                let estimated_profit =
                    (*optimal_amount as f64 * *profit_bps as f64) / (10_000.0 * 1e9);
                if estimated_profit > 0.005 {
                    // Minimum 0.005 SOL profit
                    Some(MevOpportunity {
                        opportunity_type: event.opportunity_type.clone(),
                        estimated_profit_sol: estimated_profit,
                        estimated_gas_cost: 15_000, // Higher gas for cross-DEX
                        execution_deadline_ns: deadline_ns,
                        priority: event.priority,
                    })
                } else {
                    None
                }
            }

            OpportunityType::NewToken {
                initial_liquidity_sol,
                ..
            } => {
                let estimated_profit = (*initial_liquidity_sol as f64 * 0.02) / 1e9; // 2% of liquidity
                Some(MevOpportunity {
                    opportunity_type: event.opportunity_type.clone(),
                    estimated_profit_sol: estimated_profit,
                    estimated_gas_cost: 20_000, // Higher gas for token sniping
                    execution_deadline_ns: deadline_ns,
                    priority: event.priority,
                })
            }

            _ => None,
        }
    }

    /// Get execution deadline based on priority
    fn get_execution_deadline(&self, priority: EventPriority) -> u64 {
        match priority {
            EventPriority::Critical => 50_000_000, // 50ms
            EventPriority::High => 100_000_000,    // 100ms
            EventPriority::Medium => 500_000_000,  // 500ms
            EventPriority::Low => 2_000_000_000,   // 2s
        }
    }

    /// Calculate potential sandwich profit
    fn calculate_sandwich_profit(&self, swap_amount: u64, slippage_bps: u64) -> f64 {
        let swap_amount_sol = swap_amount as f64 / 1e9;
        let slippage_factor = slippage_bps as f64 / 10_000.0;

        // Simplified sandwich profit calculation
        // Real implementation would need proper AMM math
        swap_amount_sol * slippage_factor * 0.5 // 50% of slippage captured
    }
}

/// MEV execution result
#[derive(Debug)]
pub struct MevExecutionResult {
    pub success: bool,
    pub profit_sol: f64,
    pub gas_used: u64,
    pub execution_time_ms: u64,
    pub transaction_hash: Option<String>,
}

impl MevProcessor {
    pub fn new(dry_run: bool) -> Self {
        Self {
            dry_run,
            processed_count: 0,
            successful_count: 0,
        }
    }

    /// Execute MEV opportunity
    pub async fn execute_opportunity(
        &mut self,
        opportunity: MevOpportunity,
    ) -> Result<MevExecutionResult> {
        let start_time = Instant::now();
        self.processed_count += 1;

        if self.dry_run {
            info!(
                "DRY RUN: Would execute MEV opportunity: {:?}",
                opportunity.opportunity_type
            );
            self.successful_count += 1;
            return Ok(MevExecutionResult {
                success: true,
                profit_sol: opportunity.estimated_profit_sol,
                gas_used: opportunity.estimated_gas_cost,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                transaction_hash: Some("dry_run_tx_hash".to_string()),
            });
        }

        // Real execution logic would go here
        match opportunity.opportunity_type {
            OpportunityType::Sandwich { .. } => self.execute_sandwich_attack(opportunity).await,
            OpportunityType::Arbitrage { .. } => self.execute_arbitrage(opportunity).await,
            OpportunityType::NewToken { .. } => self.execute_token_snipe(opportunity).await,
            _ => {
                warn!("Unknown opportunity type, skipping execution");
                Ok(MevExecutionResult {
                    success: false,
                    profit_sol: 0.0,
                    gas_used: 0,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    transaction_hash: None,
                })
            }
        }
    }

    /// Execute sandwich attack (placeholder)
    async fn execute_sandwich_attack(
        &mut self,
        opportunity: MevOpportunity,
    ) -> Result<MevExecutionResult> {
        let start_time = Instant::now();

        // TODO: Implement actual sandwich execution
        // 1. Build front-run transaction
        // 2. Build back-run transaction
        // 3. Submit as bundle to Jito

        warn!("Sandwich execution not implemented yet - skipping for safety");

        Ok(MevExecutionResult {
            success: false,
            profit_sol: 0.0,
            gas_used: 0,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            transaction_hash: None,
        })
    }

    /// Execute arbitrage opportunity
    async fn execute_arbitrage(
        &mut self,
        opportunity: MevOpportunity,
    ) -> Result<MevExecutionResult> {
        let start_time = Instant::now();

        // TODO: Implement arbitrage execution
        // 1. Calculate optimal swap amounts
        // 2. Build swap transactions for both DEXes
        // 3. Submit as atomic bundle

        info!(
            "Arbitrage execution placeholder - would execute: {:?}",
            opportunity.opportunity_type
        );
        self.successful_count += 1;

        Ok(MevExecutionResult {
            success: true,
            profit_sol: opportunity.estimated_profit_sol * 0.8, // 80% success rate simulation
            gas_used: opportunity.estimated_gas_cost,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            transaction_hash: Some("arb_tx_placeholder".to_string()),
        })
    }

    /// Execute token sniping
    async fn execute_token_snipe(
        &mut self,
        opportunity: MevOpportunity,
    ) -> Result<MevExecutionResult> {
        let start_time = Instant::now();

        // TODO: Implement token sniping
        // 1. Detect new pool creation
        // 2. Build immediate buy transaction
        // 3. Submit with high priority fee

        info!(
            "Token snipe execution placeholder - would execute: {:?}",
            opportunity.opportunity_type
        );
        self.successful_count += 1;

        Ok(MevExecutionResult {
            success: true,
            profit_sol: opportunity.estimated_profit_sol * 0.6, // 60% success rate for sniping
            gas_used: opportunity.estimated_gas_cost,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            transaction_hash: Some("snipe_tx_placeholder".to_string()),
        })
    }

    /// Get processor statistics
    pub fn get_stats(&self) -> (u64, u64, f64) {
        let success_rate = if self.processed_count > 0 {
            (self.successful_count as f64 / self.processed_count as f64) * 100.0
        } else {
            0.0
        };

        (self.processed_count, self.successful_count, success_rate)
    }
}
