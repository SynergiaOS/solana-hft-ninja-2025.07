#!/usr/bin/env python3
"""
Simple BFF for Cerebro Dashboard Demo
FastAPI server with mock trading data
"""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
import uvicorn
import logging
from datetime import datetime

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# FastAPI app
app = FastAPI(
    title="Cerebro Simple BFF",
    description="Backend-for-Frontend with mock trading data",
    version="1.0.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3001", "http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "timestamp": datetime.now().isoformat(),
        "version": "1.0.0"
    }

@app.get("/api/trading/history")
async def get_trading_history():
    """Get successful trading history with real-looking data"""
    return {
        "trades": [
            {
                "id": "tx_sandwich_001",
                "type": "sandwich",
                "token_pair": "SOL/USDC",
                "profit_sol": 0.0847,
                "profit_usd": 12.34,
                "execution_time_ms": 87,
                "timestamp": "2025-07-18T23:15:42Z",
                "status": "completed",
                "strategy": "SandwichStrategy",
                "confidence": 0.94,
                "gas_fees": 0.0012,
                "slippage": 0.23,
                "dex": "Raydium"
            },
            {
                "id": "tx_arbitrage_002", 
                "type": "arbitrage",
                "token_pair": "BONK/SOL",
                "profit_sol": 0.156,
                "profit_usd": 22.67,
                "execution_time_ms": 134,
                "timestamp": "2025-07-18T23:12:18Z",
                "status": "completed",
                "strategy": "CrossDexArbitrage",
                "confidence": 0.89,
                "gas_fees": 0.0018,
                "price_diff": 2.34,
                "dex_from": "Orca",
                "dex_to": "Jupiter"
            },
            {
                "id": "tx_liquidation_003",
                "type": "liquidation", 
                "token_pair": "mSOL/USDC",
                "profit_sol": 0.234,
                "profit_usd": 34.12,
                "execution_time_ms": 76,
                "timestamp": "2025-07-18T23:08:55Z",
                "status": "completed",
                "strategy": "LiquidationBot",
                "confidence": 0.97,
                "gas_fees": 0.0015,
                "liquidation_bonus": 5.5,
                "protocol": "Solend"
            },
            {
                "id": "tx_snipe_004",
                "type": "token_snipe",
                "token_pair": "NEWTOKEN/SOL", 
                "profit_sol": 0.445,
                "profit_usd": 64.78,
                "execution_time_ms": 45,
                "timestamp": "2025-07-18T22:58:33Z",
                "status": "completed",
                "strategy": "TokenLaunchSniper",
                "confidence": 0.91,
                "gas_fees": 0.0025,
                "entry_price": 0.000123,
                "exit_price": 0.000189,
                "tokens_bought": 1000000
            },
            {
                "id": "tx_jupiter_005",
                "type": "jupiter_arbitrage",
                "token_pair": "RAY/USDC",
                "profit_sol": 0.089,
                "profit_usd": 12.95,
                "execution_time_ms": 112,
                "timestamp": "2025-07-18T22:45:21Z", 
                "status": "completed",
                "strategy": "JupiterArbStrategy",
                "confidence": 0.86,
                "gas_fees": 0.0014,
                "route_hops": 3,
                "impact": 0.12
            }
        ],
        "summary": {
            "total_trades": 5,
            "successful_trades": 5,
            "total_profit_sol": 1.0087,
            "total_profit_usd": 146.86,
            "success_rate": 100.0,
            "avg_execution_time_ms": 90.8,
            "total_gas_fees": 0.0084,
            "net_profit_sol": 1.0003,
            "roi_percentage": 12.5
        }
    }

@app.get("/api/strategies")
async def get_strategies():
    """Get strategy performance data"""
    return {
        "strategies": [
            {
                "name": "SandwichStrategy",
                "active": True,
                "trades_today": 12,
                "success_rate": 94.2,
                "profit_sol": 0.847,
                "avg_execution_ms": 89,
                "risk_level": "medium",
                "last_trade": "2025-07-18T23:15:42Z"
            },
            {
                "name": "CrossDexArbitrage", 
                "active": True,
                "trades_today": 8,
                "success_rate": 87.5,
                "profit_sol": 0.623,
                "avg_execution_ms": 145,
                "risk_level": "low",
                "last_trade": "2025-07-18T23:12:18Z"
            },
            {
                "name": "LiquidationBot",
                "active": True,
                "trades_today": 3,
                "success_rate": 100.0,
                "profit_sol": 0.456,
                "avg_execution_ms": 78,
                "risk_level": "low",
                "last_trade": "2025-07-18T23:08:55Z"
            },
            {
                "name": "TokenLaunchSniper",
                "active": True,
                "trades_today": 2,
                "success_rate": 50.0,
                "profit_sol": 0.445,
                "avg_execution_ms": 52,
                "risk_level": "high",
                "last_trade": "2025-07-18T22:58:33Z"
            },
            {
                "name": "JupiterArbStrategy",
                "active": True,
                "trades_today": 6,
                "success_rate": 83.3,
                "profit_sol": 0.234,
                "avg_execution_ms": 118,
                "risk_level": "medium",
                "last_trade": "2025-07-18T22:45:21Z"
            }
        ]
    }

@app.get("/api/live/events")
async def get_live_events():
    """Get recent live events for real-time dashboard"""
    import random
    from datetime import datetime, timedelta

    events = []
    now = datetime.now()

    # Generate some recent events
    for i in range(10):
        event_time = now - timedelta(minutes=random.randint(0, 30))
        event_types = [
            {
                "type": "new_trade",
                "data": {
                    "id": f"tx_{random.randint(1000, 9999)}",
                    "type": random.choice(["sandwich", "arbitrage", "liquidation"]),
                    "token_pair": random.choice(["SOL/USDC", "BONK/SOL", "RAY/USDC"]),
                    "profit_sol": round(random.uniform(0.01, 0.5), 4),
                    "profit_usd": round(random.uniform(1, 50), 2),
                    "execution_time_ms": random.randint(45, 200),
                    "confidence": round(random.uniform(0.8, 0.99), 2)
                }
            },
            {
                "type": "opportunity_detected",
                "data": {
                    "type": random.choice(["arbitrage", "sandwich"]),
                    "token_pair": random.choice(["BONK/SOL", "RAY/USDC", "mSOL/SOL"]),
                    "potential_profit": round(random.uniform(0.005, 0.1), 4),
                    "confidence": round(random.uniform(0.7, 0.95), 2),
                    "dex_from": random.choice(["Orca", "Raydium"]),
                    "dex_to": random.choice(["Jupiter", "Serum"])
                }
            },
            {
                "type": "system_metrics",
                "data": {
                    "transactions_processed": random.randint(1200, 1300),
                    "opportunities_detected": random.randint(150, 170),
                    "avg_latency_ms": random.randint(80, 120),
                    "memory_usage_mb": random.randint(200, 300)
                }
            }
        ]

        event = random.choice(event_types)
        event["timestamp"] = event_time.isoformat()
        events.append(event)

    return {"events": sorted(events, key=lambda x: x["timestamp"], reverse=True)}

@app.get("/api/system/metrics")
async def get_system_metrics():
    """Get real-time system metrics from HFT Ninja"""
    import httpx
    try:
        # Try to get real metrics from HFT Ninja
        async with httpx.AsyncClient() as client:
            response = await client.get("http://localhost:9464/metrics", timeout=5.0)
            if response.status_code == 200:
                # Parse Prometheus metrics
                metrics_text = response.text

                # Extract key metrics
                transactions_processed = 0
                opportunities_detected = 0

                for line in metrics_text.split('\n'):
                    if line.startswith('hft_transactions_processed_total'):
                        transactions_processed = int(float(line.split()[-1]))
                    elif line.startswith('hft_arbitrage_opportunities_total'):
                        opportunities_detected = int(float(line.split()[-1]))

                return {
                    "performance": {
                        "uptime_hours": 47.3,
                        "transactions_processed": transactions_processed,
                        "opportunities_detected": opportunities_detected,
                        "successful_executions": 31,
                        "avg_latency_ms": 92.4,
                        "memory_usage_mb": 234.7,
                        "cpu_usage_percent": 23.8
                    },
                    "trading": {
                        "daily_pnl_sol": 2.145,
                        "daily_pnl_usd": 312.45,
                        "total_volume_sol": 45.67,
                        "active_strategies": 5,
                        "pending_orders": 0,
                        "risk_exposure": 15.2
                    },
                    "network": {
                        "helius_connected": True,
                        "websocket_status": "connected",
                        "last_block": 285647392,
                        "tps": 2847,
                        "slot_height": 285647392
                    }
                }
    except Exception as e:
        logger.warning(f"Could not fetch real metrics: {e}")

    # Fallback to mock data
    return {
        "performance": {
            "uptime_hours": 47.3,
            "transactions_processed": 1247,
            "opportunities_detected": 156,
            "successful_executions": 31,
            "avg_latency_ms": 92.4,
            "memory_usage_mb": 234.7,
            "cpu_usage_percent": 23.8
        },
        "trading": {
            "daily_pnl_sol": 2.145,
            "daily_pnl_usd": 312.45,
            "total_volume_sol": 45.67,
            "active_strategies": 5,
            "pending_orders": 0,
            "risk_exposure": 15.2
        },
        "network": {
            "helius_connected": True,
            "websocket_status": "connected",
            "last_block": 285647392,
            "tps": 2847,
            "slot_height": 285647392
        }
    }

if __name__ == "__main__":
    logger.info("🚀 Starting Cerebro Simple BFF...")
    uvicorn.run(
        "simple_bff:app",
        host="0.0.0.0",
        port=8000,
        reload=True,
        log_level="info"
    )
