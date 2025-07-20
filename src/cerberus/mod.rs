use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn, error, debug};

pub mod position;
pub mod decision_tree;
pub mod execution;
pub mod rpc_manager;
pub mod store;

pub use position::*;
pub use decision_tree::*;
pub use execution::*;
pub use rpc_manager::*;
pub use store::*;

/// Cerberus Trade Execution Brain
/// 
/// Autonomous decision-making system for position management
/// - Analyzes every open position every 200ms
/// - Executes hard rules: timeout, stop-loss, take-profit
/// - Responds to AI signals from Cerebro
/// - Uses dual RPC (QuickNode + Helius) for reliability
#[derive(Clone)]
pub struct CerberusBrain {
    pub store: Arc<CerberusStore>,
    pub rpc_manager: Arc<RpcManager>,
    pub executor: Arc<CerberusExecutor>,
    pub config: CerberusConfig,
}

#[derive(Clone, Debug)]
pub struct CerberusConfig {
    pub loop_interval_ms: u64,
    pub quicknode_endpoint: String,
    pub helius_endpoint: String,
    pub redis_url: String,
    pub jito_endpoint: String,
    pub max_concurrent_positions: usize,
    pub default_timeout_seconds: u64,
    pub emergency_stop_enabled: bool,
}

impl Default for CerberusConfig {
    fn default() -> Self {
        Self {
            loop_interval_ms: 200,
            quicknode_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            helius_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            jito_endpoint: "https://mainnet.block-engine.jito.wtf".to_string(),
            max_concurrent_positions: 50,
            default_timeout_seconds: 600, // 10 minutes
            emergency_stop_enabled: true,
        }
    }
}

impl CerberusBrain {
    /// Initialize Cerberus with premium endpoints
    pub async fn new(config: CerberusConfig) -> Result<Self> {
        info!("🧠 Initializing Cerberus Trade Execution Brain");
        
        let store = Arc::new(CerberusStore::new(&config.redis_url).await?);
        let rpc_manager = Arc::new(RpcManager::new(
            &config.quicknode_endpoint,
            &config.helius_endpoint,
        ).await?);
        let executor = Arc::new(CerberusExecutor::new(
            Arc::clone(&rpc_manager),
            &config.jito_endpoint,
        ).await?);

        info!("✅ Cerberus initialized with dual RPC endpoints");
        info!("📊 QuickNode: {}", config.quicknode_endpoint);
        info!("📊 Helius: {}", config.helius_endpoint);
        
        Ok(Self {
            store,
            rpc_manager,
            executor,
            config,
        })
    }

    /// Start the main decision loop (200ms intervals)
    pub async fn start_decision_loop(&self) -> Result<()> {
        info!("🔄 Starting Cerberus decision loop ({}ms intervals)", self.config.loop_interval_ms);
        
        let brain = self.clone();
        
        // Background task for external command listening
        let _command_listener = {
            let brain = brain.clone();
            tokio::spawn(async move {
                if let Err(e) = brain.listen_external_commands().await {
                    error!("External command listener failed: {}", e);
                }
            })
        };

        // Main decision loop
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_millis(self.config.loop_interval_ms)
        );

        loop {
            interval.tick().await;
            
            if let Err(e) = self.process_all_positions().await {
                error!("Error processing positions: {}", e);
                continue;
            }
        }
    }

    /// Process all open positions through decision tree
    async fn process_all_positions(&self) -> Result<()> {
        let positions = self.store.get_all_open_positions().await?;
        
        if positions.is_empty() {
            return Ok(());
        }

        debug!("🔍 Processing {} open positions", positions.len());

        for position in positions {
            if let Err(e) = self.process_single_position(&position).await {
                error!("Error processing position {}: {}", position.mint, e);
                continue;
            }
        }

        Ok(())
    }

    /// Process single position through decision tree
    async fn process_single_position(&self, position: &PositionState) -> Result<()> {
        // Fetch live market data
        let market_data = self.rpc_manager.get_market_data(&position.mint).await?;
        
        // Calculate current PnL
        let current_price = market_data.price;
        let pnl_percent = ((current_price - position.entry_price) / position.entry_price) * 100.0;
        
        // Update position with latest data
        let mut updated_position = position.clone();
        updated_position.current_price = Some(current_price);
        updated_position.pnl_unrealized_percent = Some(pnl_percent);
        updated_position.last_analysis_timestamp = chrono::Utc::now().timestamp() as u64;
        
        // Run decision tree
        let decision = run_decision_tree(&updated_position, &market_data).await?;
        
        // Execute decision
        match decision {
            Decision::Sell(reason) => {
                info!("💰 SELL {} - Reason: {}", position.mint, reason);
                self.executor.execute_sell(&updated_position, &reason).await?;
                self.store.close_position(&position.mint, &reason).await?;
            },
            Decision::BuyMore(amount_sol) => {
                info!("📈 BUY MORE {} - Amount: {} SOL", position.mint, amount_sol);
                self.executor.execute_buy_more(&updated_position, amount_sol).await?;
                // Update position size
                updated_position.position_size_sol += amount_sol;
                self.store.update_position(&updated_position).await?;
            },
            Decision::Hold => {
                // Just update the position with latest data
                self.store.update_position(&updated_position).await?;
            },
        }

        Ok(())
    }

    /// Listen for external commands (Guardian alerts, Cerebro signals)
    async fn listen_external_commands(&self) -> Result<()> {
        info!("👂 Starting external command listener");
        self.store.listen_commands(Arc::clone(&self.executor)).await
    }

    /// Emergency stop - close all positions immediately
    pub async fn emergency_stop(&self, reason: &str) -> Result<()> {
        warn!("🚨 EMERGENCY STOP TRIGGERED: {}", reason);
        
        let positions = self.store.get_all_open_positions().await?;
        
        let position_count = positions.len();
        for position in &positions {
            if let Err(e) = self.executor.execute_sell(&position, &format!("EMERGENCY: {}", reason)).await {
                error!("Failed to emergency sell {}: {}", position.mint, e);
            } else {
                self.store.close_position(&position.mint, &format!("EMERGENCY: {}", reason)).await?;
            }
        }

        info!("✅ Emergency stop completed - {} positions closed", position_count);
        Ok(())
    }
}
