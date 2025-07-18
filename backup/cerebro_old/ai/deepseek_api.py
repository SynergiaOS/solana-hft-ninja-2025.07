"""
🧮 DeepSeek-Math FastAPI Server
Cost-effective AI API for mathematical trading calculations.
Optimized for <$1 operational cost with smart caching and quantization.
"""

import asyncio
import logging
import time
import os
from typing import Dict, List, Optional, Any
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException, BackgroundTasks
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
import uvicorn

from .deepseek_math import DeepSeekMath, TradingCalculation, RiskAssessment
from ..config.ai_config import DeepSeekConfig

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Global model instance
deepseek_model: Optional[DeepSeekMath] = None

# Request/Response Models
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

class SandwichCalculationRequest(BaseModel):
    target_tx_size: float = Field(..., gt=0, description="Target transaction size")
    pool_liquidity: float = Field(..., gt=0, description="Pool liquidity")
    current_price: float = Field(..., gt=0, description="Current token price")
    slippage: float = Field(..., ge=0.1, le=10.0, description="Slippage tolerance %")

class RiskAssessmentRequest(BaseModel):
    strategy: str = Field(..., description="Trading strategy")
    token: str = Field(..., description="Token address")
    position_size: float = Field(..., gt=0, description="Position size in SOL")
    market_conditions: Dict[str, Any] = Field(..., description="Market conditions")
    volatility: float = Field(..., ge=0.0, le=1.0, description="Historical volatility")
    liquidity: float = Field(..., gt=0, description="Available liquidity")

class CalculationResponse(BaseModel):
    calculation_type: str
    result: Dict[str, Any]
    confidence: float
    reasoning: str
    execution_time_ms: int
    model_used: str
    timestamp: float

class RiskAssessmentResponse(BaseModel):
    risk_score: float
    risk_factors: List[str]
    recommended_position_size: float
    max_loss_estimate: float
    confidence: float
    reasoning: str
    timestamp: float

class HealthResponse(BaseModel):
    status: str
    model_loaded: bool
    memory_usage_mb: float
    cache_hit_ratio: float
    uptime_seconds: float

# Startup/Shutdown handlers
@asynccontextmanager
async def lifespan(app: FastAPI):
    """Manage application lifespan"""
    global deepseek_model
    
    # Startup
    logger.info("🧮 Starting DeepSeek-Math API server...")
    
    try:
        # Initialize configuration
        config = DeepSeekConfig(
            model_name=os.getenv("MODEL_NAME", "deepseek-ai/deepseek-math-7b-instruct"),
            use_quantization=os.getenv("USE_QUANTIZATION", "true").lower() == "true",
            use_lmcache=os.getenv("USE_LMCACHE", "true").lower() == "true",
            cache_size_mb=int(os.getenv("CACHE_SIZE_MB", "1024")),
            max_tokens=int(os.getenv("MAX_TOKENS", "512")),
            temperature=float(os.getenv("TEMPERATURE", "0.1")),
            lora_adapter_path=os.getenv("LORA_ADAPTER_PATH"),
            cache_ttl_seconds=3600
        )
        
        # Initialize model
        deepseek_model = DeepSeekMath(config)
        success = await deepseek_model.initialize()
        
        if not success:
            raise RuntimeError("Failed to initialize DeepSeek-Math model")
        
        logger.info("✅ DeepSeek-Math API server started successfully")
        
    except Exception as e:
        logger.error(f"❌ Failed to start DeepSeek-Math API: {e}")
        raise
    
    yield
    
    # Shutdown
    logger.info("🧹 Shutting down DeepSeek-Math API server...")
    if deepseek_model:
        await deepseek_model.cleanup()
    logger.info("✅ DeepSeek-Math API server shutdown complete")

# Create FastAPI app
app = FastAPI(
    title="DeepSeek-Math Trading API",
    description="Cost-effective AI for mathematical trading calculations",
    version="1.0.0",
    lifespan=lifespan
)

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Store startup time for uptime calculation
startup_time = time.time()

@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint"""
    global deepseek_model
    
    try:
        if deepseek_model:
            metrics = await deepseek_model.get_metrics()
            return HealthResponse(
                status="healthy",
                model_loaded=True,
                memory_usage_mb=metrics.get("memory_usage_mb", 0),
                cache_hit_ratio=metrics.get("cache_hit_ratio", 0),
                uptime_seconds=time.time() - startup_time
            )
        else:
            return HealthResponse(
                status="unhealthy",
                model_loaded=False,
                memory_usage_mb=0,
                cache_hit_ratio=0,
                uptime_seconds=time.time() - startup_time
            )
    except Exception as e:
        logger.error(f"❌ Health check failed: {e}")
        raise HTTPException(status_code=500, detail="Health check failed")

@app.post("/calculate/position-size", response_model=CalculationResponse)
async def calculate_position_size(request: PositionSizeRequest):
    """Calculate optimal position size using Kelly Criterion"""
    global deepseek_model
    
    if not deepseek_model:
        raise HTTPException(status_code=503, detail="Model not initialized")
    
    try:
        calculation = await deepseek_model.calculate_position_size(
            capital=request.capital,
            risk_tolerance=request.risk_tolerance,
            expected_return=request.expected_return,
            volatility=request.volatility,
            strategy=request.strategy
        )
        
        return CalculationResponse(
            calculation_type=calculation.calculation_type,
            result=calculation.result,
            confidence=calculation.confidence,
            reasoning=calculation.reasoning,
            execution_time_ms=calculation.execution_time_ms,
            model_used=calculation.model_used,
            timestamp=time.time()
        )
        
    except Exception as e:
        logger.error(f"❌ Position size calculation failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/calculate/arbitrage-profit", response_model=CalculationResponse)
async def calculate_arbitrage_profit(request: ArbitrageProfitRequest):
    """Calculate arbitrage profit potential"""
    global deepseek_model
    
    if not deepseek_model:
        raise HTTPException(status_code=503, detail="Model not initialized")
    
    try:
        calculation = await deepseek_model.calculate_arbitrage_profit(
            token=request.token,
            price_a=request.price_a,
            price_b=request.price_b,
            liquidity_a=request.liquidity_a,
            liquidity_b=request.liquidity_b,
            gas_cost=request.gas_cost
        )
        
        return CalculationResponse(
            calculation_type=calculation.calculation_type,
            result=calculation.result,
            confidence=calculation.confidence,
            reasoning=calculation.reasoning,
            execution_time_ms=calculation.execution_time_ms,
            model_used=calculation.model_used,
            timestamp=time.time()
        )
        
    except Exception as e:
        logger.error(f"❌ Arbitrage calculation failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/calculate/sandwich", response_model=CalculationResponse)
async def calculate_sandwich_parameters(request: SandwichCalculationRequest):
    """Calculate sandwich attack parameters"""
    global deepseek_model
    
    if not deepseek_model:
        raise HTTPException(status_code=503, detail="Model not initialized")
    
    try:
        # Use the sandwich calculation prompt
        prompt = f"""
Calculate sandwich attack parameters:
- Target transaction: {request.target_tx_size} SOL
- Pool liquidity: {request.pool_liquidity} SOL
- Current price: {request.current_price}
- Slippage tolerance: {request.slippage}%

Calculate optimal front-run and back-run sizes. Return JSON:
{{"front_run_size": float, "back_run_size": float, "expected_profit": float, "risk_score": float}}
"""
        
        response = await deepseek_model._generate_response(prompt)
        result = deepseek_model._parse_json_response(response)
        
        return CalculationResponse(
            calculation_type="sandwich_calculation",
            result=result,
            confidence=0.8,
            reasoning="Sandwich attack parameter calculation",
            execution_time_ms=100,  # Placeholder
            model_used=deepseek_model.config.model_name,
            timestamp=time.time()
        )
        
    except Exception as e:
        logger.error(f"❌ Sandwich calculation failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/assess/risk", response_model=RiskAssessmentResponse)
async def assess_trading_risk(request: RiskAssessmentRequest):
    """Assess trading risk for a position"""
    global deepseek_model
    
    if not deepseek_model:
        raise HTTPException(status_code=503, detail="Model not initialized")
    
    try:
        risk_assessment = await deepseek_model.assess_trading_risk(
            strategy=request.strategy,
            token=request.token,
            position_size=request.position_size,
            market_conditions=request.market_conditions,
            volatility=request.volatility,
            liquidity=request.liquidity
        )
        
        return RiskAssessmentResponse(
            risk_score=risk_assessment.risk_score,
            risk_factors=risk_assessment.risk_factors,
            recommended_position_size=risk_assessment.recommended_position_size,
            max_loss_estimate=risk_assessment.max_loss_estimate,
            confidence=risk_assessment.confidence,
            reasoning=risk_assessment.reasoning,
            timestamp=time.time()
        )
        
    except Exception as e:
        logger.error(f"❌ Risk assessment failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/metrics")
async def get_metrics():
    """Get model performance metrics"""
    global deepseek_model
    
    if not deepseek_model:
        raise HTTPException(status_code=503, detail="Model not initialized")
    
    try:
        metrics = await deepseek_model.get_metrics()
        metrics["uptime_seconds"] = time.time() - startup_time
        return metrics
        
    except Exception as e:
        logger.error(f"❌ Metrics retrieval failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/cache/clear")
async def clear_cache():
    """Clear model cache"""
    global deepseek_model
    
    if not deepseek_model or not deepseek_model.lmcache:
        raise HTTPException(status_code=503, detail="Cache not available")
    
    try:
        await deepseek_model.lmcache.clear()
        return {"status": "success", "message": "Cache cleared"}
        
    except Exception as e:
        logger.error(f"❌ Cache clear failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    port = int(os.getenv("API_PORT", "8003"))
    uvicorn.run(
        "ai.deepseek_api:app",
        host="0.0.0.0",
        port=port,
        workers=1,
        log_level="info"
    )
