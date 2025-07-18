"""
🧮 DeepSeek-Math Mock API for Testing
Simple mock version for testing without actual AI model
"""

import time
import random
import asyncio
from typing import Dict, Any
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field
import uvicorn

# Request Models
class PositionSizeRequest(BaseModel):
    capital: float = Field(..., description="Available capital in SOL")
    risk_tolerance: float = Field(..., ge=0.01, le=0.5, description="Risk tolerance (0.01-0.5)")
    expected_return: float = Field(..., description="Expected return percentage")
    volatility: float = Field(..., ge=0.0, le=1.0, description="Market volatility (0.0-1.0)")
    strategy: str = Field(..., description="Trading strategy name")

class ArbitrageProfitRequest(BaseModel):
    token: str = Field(..., description="Token address")
    price_a: float = Field(..., gt=0, description="Price on DEX A")
    price_b: float = Field(..., gt=0, description="Price on DEX B")
    liquidity_a: float = Field(..., gt=0, description="Liquidity on DEX A")
    liquidity_b: float = Field(..., gt=0, description="Liquidity on DEX B")
    gas_cost: float = Field(..., ge=0, description="Estimated gas cost")

# FastAPI app
app = FastAPI(
    title="DeepSeek-Math Mock API",
    description="Mock API for testing HFT Ninja AI calculations",
    version="1.0.0"
)

startup_time = time.time()

@app.get("/")
async def root():
    return {
        "service": "DeepSeek-Math Mock API",
        "status": "running",
        "version": "1.0.0",
        "uptime_seconds": time.time() - startup_time
    }

@app.get("/health")
async def health_check():
    return {
        "status": "healthy",
        "service": "deepseek-math-mock",
        "timestamp": time.time(),
        "uptime_seconds": time.time() - startup_time,
        "model_loaded": True,
        "cache_enabled": True
    }

@app.post("/calculate/position-size")
async def calculate_position_size(request: PositionSizeRequest):
    """Mock position size calculation using Kelly Criterion"""
    
    # Simulate processing time
    await asyncio.sleep(random.uniform(0.1, 0.3))
    
    # Mock Kelly Criterion calculation
    kelly_fraction = (request.expected_return - 0.02) / (request.volatility ** 2)
    kelly_fraction = max(0.0, min(kelly_fraction, 0.25))  # Cap at 25%
    
    position_size = request.capital * kelly_fraction * request.risk_tolerance
    position_size = min(position_size, request.capital * 0.1)  # Max 10% of capital
    
    risk_score = min(request.volatility * 2 + (1 - request.risk_tolerance), 1.0)
    
    return {
        "result": {
            "position_size": round(position_size, 4),
            "kelly_fraction": round(kelly_fraction, 4),
            "risk_score": round(risk_score, 3),
            "max_loss": round(position_size * request.risk_tolerance, 4),
            "confidence": 0.94
        },
        "metadata": {
            "strategy": request.strategy,
            "calculation_method": "kelly_criterion_mock",
            "latency_ms": random.randint(150, 250),
            "cost_usd": 0.000001,
            "timestamp": time.time()
        }
    }

@app.post("/calculate/arbitrage-profit")
async def calculate_arbitrage_profit(request: ArbitrageProfitRequest):
    """Mock arbitrage profit calculation"""
    
    # Simulate processing time
    await asyncio.sleep(random.uniform(0.1, 0.2))
    
    price_diff = abs(request.price_b - request.price_a)
    price_diff_pct = (price_diff / request.price_a) * 100
    
    # Mock profit calculation
    min_liquidity = min(request.liquidity_a, request.liquidity_b)
    max_trade_size = min_liquidity * 0.05  # 5% of liquidity
    
    gross_profit = max_trade_size * (price_diff / request.price_a)
    net_profit = gross_profit - request.gas_cost
    
    is_profitable = net_profit > 0.001  # Minimum 0.001 SOL profit
    
    return {
        "result": {
            "net_profit": round(net_profit, 6),
            "gross_profit": round(gross_profit, 6),
            "price_difference_pct": round(price_diff_pct, 3),
            "max_trade_size": round(max_trade_size, 4),
            "is_profitable": is_profitable,
            "confidence": 0.92
        },
        "metadata": {
            "token": request.token,
            "calculation_method": "arbitrage_mock",
            "latency_ms": random.randint(120, 200),
            "cost_usd": 0.000001,
            "timestamp": time.time()
        }
    }

@app.get("/metrics")
async def get_metrics():
    """Mock metrics endpoint"""
    return {
        "model_info": {
            "name": "deepseek-math-7b-mock",
            "version": "mock-1.0",
            "quantization": "4-bit",
            "memory_usage_gb": 0.1
        },
        "performance": {
            "avg_latency_ms": 200,
            "requests_processed": random.randint(100, 1000),
            "cache_hit_ratio": 0.75,
            "accuracy_score": 0.94
        },
        "cost_efficiency": {
            "daily_cost_usd": 0.001,
            "cost_per_request": 0.000001,
            "requests_per_dollar": 1000000
        },
        "uptime_seconds": time.time() - startup_time
    }

if __name__ == "__main__":
    port = 8003
    print(f"🧮 Starting DeepSeek-Math Mock API on port {port}")
    print("📊 Mock calculations available:")
    print("  • Position sizing (Kelly Criterion)")
    print("  • Arbitrage profit analysis") 
    print("  • Risk assessment")
    
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=port,
        log_level="info"
    )
