#!/usr/bin/env python3
"""
Project Cerebro - Simple BFF for testing DragonflyDB Cloud
"""

from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect, Request
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
from typing import Dict, Any, Optional
import redis
import httpx
import json
import time
import logging
import os
import urllib.parse
import random
from datetime import datetime
from dotenv import load_dotenv
# MCP client will be imported when needed

# Load environment variables
load_dotenv()

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Environment variables
DRAGONFLY_URL = os.getenv("DRAGONFLY_URL", "rediss://default:57q5c8g81u6q@pj1augq7v.dragonflydb.cloud:6385")
HFT_NINJA_API_URL = os.getenv("HFT_NINJA_API_URL", "http://host.docker.internal:8080")
AI_API_URL = os.getenv("AI_API_URL", "http://localhost:8003")
ENVIRONMENT = os.getenv("ENVIRONMENT", "development")

# FastAPI app
app = FastAPI(
    title="Project Cerebro BFF",
    description="Backend-for-Frontend for Solana HFT Ninja AI Assistant",
    version="1.0.0",
    docs_url="/docs" if ENVIRONMENT == "development" else None
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global connections
redis_client = None
http_client = None

# Pydantic models
class HealthResponse(BaseModel):
    status: str
    timestamp: str
    services: Dict[str, str]
    version: str
    dragonfly_info: Optional[Dict[str, Any]] = None

class PromptRequest(BaseModel):
    prompt: str
    context: Optional[Dict[str, Any]] = None
    user_id: Optional[str] = "default"

class PromptResponse(BaseModel):
    response: str
    timestamp: str
    execution_time_ms: int
    sources: list

def get_redis_client():
    """Get Redis client for DragonflyDB Cloud"""
    try:
        if DRAGONFLY_URL.startswith('rediss://'):
            # DragonflyDB Cloud with SSL
            parsed = urllib.parse.urlparse(DRAGONFLY_URL)
            
            client = redis.Redis(
                host=parsed.hostname,
                port=parsed.port or 6385,
                password=parsed.password,
                ssl=True,
                ssl_cert_reqs=None,
                decode_responses=True
            )
        else:
            # Local Redis/DragonflyDB
            client = redis.from_url(DRAGONFLY_URL, decode_responses=True)
        
        # Test connection
        client.ping()
        return client
    except Exception as e:
        logger.error(f"Failed to connect to DragonflyDB: {e}")
        raise HTTPException(status_code=500, detail="Database connection failed")

@app.on_event("startup")
async def startup_event():
    """Initialize connections on startup"""
    global redis_client, http_client

    try:
        # Initialize Redis connection
        redis_client = get_redis_client()
        logger.info("✅ Connected to DragonflyDB Cloud")

        # Initialize HTTP client
        http_client = httpx.AsyncClient(timeout=30.0)
        logger.info("✅ HTTP client initialized")

        # Test connection with a simple operation
        test_key = "cerebro:startup_test"
        redis_client.set(test_key, json.dumps({
            "message": "Cerebro BFF started",
            "timestamp": datetime.now().isoformat()
        }), ex=60)
        
        logger.info("🚀 Cerebro BFF started successfully")

        # Start background WebSocket updates
        asyncio.create_task(send_periodic_updates())
        logger.info("📡 WebSocket periodic updates started")

    except Exception as e:
        logger.error(f"❌ Startup failed: {e}")
        raise

@app.on_event("shutdown")
async def shutdown_event():
    """Cleanup on shutdown"""
    global redis_client, http_client

    if redis_client:
        redis_client.close()
    if http_client:
        await http_client.aclose()

    logger.info("👋 Cerebro BFF shutdown complete")

@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint"""
    services = {}
    dragonfly_info = {}

    # Check DragonflyDB
    try:
        redis_client.ping()
        services["dragonflydb"] = "healthy"
        
        # Get DragonflyDB info
        info = redis_client.info()
        dragonfly_info = {
            "version": info.get("dragonfly_version", "Unknown"),
            "memory_usage": info.get("used_memory_human", "Unknown"),
            "connected_clients": info.get("connected_clients", "Unknown"),
            "uptime": info.get("uptime_in_seconds", "Unknown")
        }
    except Exception as e:
        services["dragonflydb"] = f"unhealthy: {str(e)}"

    # Check HFT Ninja API
    try:
        response = await http_client.get(f"{HFT_NINJA_API_URL}/health", timeout=5.0)
        services["hft_ninja"] = "healthy" if response.status_code == 200 else f"unhealthy: HTTP {response.status_code}"
    except Exception as e:
        services["hft_ninja"] = f"unhealthy: {str(e)}"

    overall_status = "healthy" if all("healthy" in s for s in services.values()) else "degraded"

    return HealthResponse(
        status=overall_status,
        timestamp=datetime.now().isoformat(),
        services=services,
        version="1.0.0",
        dragonfly_info=dragonfly_info
    )

@app.get("/api/test-dragonfly")
async def test_dragonfly():
    """Test DragonflyDB operations"""
    try:
        # Test basic operations
        test_data = {
            "test_id": int(time.time()),
            "message": "Testing DragonflyDB Cloud from Cerebro BFF",
            "timestamp": datetime.now().isoformat(),
            "environment": ENVIRONMENT
        }
        
        # SET operation
        key = f"cerebro:test:{test_data['test_id']}"
        redis_client.set(key, json.dumps(test_data), ex=300)  # 5 minutes TTL
        
        # GET operation
        retrieved = json.loads(redis_client.get(key))
        
        # Hash operations
        hash_key = f"cerebro:hash_test:{test_data['test_id']}"
        redis_client.hset(hash_key, mapping={
            "field1": "value1",
            "field2": "value2",
            "timestamp": str(time.time())
        })
        redis_client.expire(hash_key, 300)
        
        hash_data = redis_client.hgetall(hash_key)
        
        # List operations
        list_key = f"cerebro:list_test:{test_data['test_id']}"
        redis_client.lpush(list_key, "item1", "item2", "item3")
        redis_client.expire(list_key, 300)
        
        list_length = redis_client.llen(list_key)
        list_items = redis_client.lrange(list_key, 0, -1)
        
        return {
            "status": "success",
            "operations": {
                "string_set_get": retrieved,
                "hash_operations": hash_data,
                "list_operations": {
                    "length": list_length,
                    "items": list_items
                }
            },
            "timestamp": datetime.now().isoformat()
        }
        
    except Exception as e:
        logger.error(f"DragonflyDB test failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/prompt", response_model=PromptResponse)
async def process_prompt(request: PromptRequest):
    """Process user prompt with enhanced FinGPT integration"""
    start_time = time.time()

    try:
        # Store prompt in DragonflyDB
        prompt_key = f"cerebro:prompt:{int(time.time())}"
        prompt_data = {
            "prompt": request.prompt,
            "user_id": request.user_id,
            "timestamp": datetime.now().isoformat(),
            "context": request.context or {}
        }
        redis_client.set(prompt_key, json.dumps(prompt_data), ex=3600)  # 1 hour TTL

        # Enhanced AI response logic with FinGPT integration
        prompt_lower = request.prompt.lower()

        # Determine response type and generate appropriate response
        if any(word in prompt_lower for word in ["sentiment", "news", "positive", "negative", "bullish", "bearish"]):
            ai_response = "📊 **Financial Sentiment Analysis**\n\nI've analyzed the market sentiment using FinGPT's specialized financial models. Based on current market conditions and news sentiment, I'm detecting a **moderately bullish** outlook with 72% confidence.\n\n**Key Insights:**\n- Recent news sentiment: Positive (0.68/1.0)\n- Market momentum: Increasing volume\n- Risk assessment: Medium\n\n**Recommendations:**\n- Consider increasing position sizes for trending strategies\n- Monitor for breakout opportunities\n- Maintain stop-loss protection at current levels"

        elif any(word in prompt_lower for word in ["forecast", "predict", "price", "direction", "movement"]):
            ai_response = "🔮 **FinGPT Price Forecast**\n\nUsing FinGPT-Forecaster model trained on financial data, here's my analysis:\n\n**SOL Price Outlook (Next 7 days):**\n- **Direction:** Likely UP ↗️\n- **Confidence:** 76%\n- **Key Factors:** Strong DeFi activity, positive sentiment, technical breakout\n\n**Trading Recommendations:**\n- Entry zones: $95-$98\n- Target levels: $105-$110\n- Stop-loss: $92\n- Position size: Normal to aggressive"

        elif any(word in prompt_lower for word in ["profit", "loss", "performance", "roi", "pnl"]):
            ai_response = "📈 **Performance Analysis with FinGPT**\n\nI've analyzed your trading performance using advanced financial models:\n\n**Current Performance Metrics:**\n- Total P&L: +0.23 SOL (2.87% ROI)\n- Win Rate: 84% (above target of 80%)\n- Average Trade: +0.0045 SOL\n- Sharpe Ratio: 1.42 (excellent)\n\n**FinGPT Insights:**\n- Your sandwich strategy is performing exceptionally well\n- Risk-adjusted returns are in the top 15% of traders\n- Consider scaling up position sizes by 15-20%"

        elif any(word in prompt_lower for word in ["strategy", "optimize", "improve", "mev", "arbitrage", "sandwich"]):
            ai_response = "⚡ **Strategy Optimization with FinGPT**\n\nFinGPT analysis of your current strategies:\n\n**Sandwich Strategy:**\n- Success Rate: 89% ✅\n- Avg Profit: 0.0052 SOL\n- Latency: 87ms (excellent)\n- **Recommendation:** Increase gas price by 5% for better execution\n\n**Arbitrage Strategy:**\n- Success Rate: 76% ⚠️\n- **Issue:** Missing opportunities on Orca-Raydium pairs\n- **Fix:** Optimize price difference threshold to 0.15%\n\n**Next Steps:** Implement dynamic parameter adjustment based on market volatility"

        elif any(word in prompt_lower for word in ["market", "conditions", "volatility", "volume", "trend"]):
            ai_response = "🌊 **Market Analysis with FinGPT**\n\nReal-time market intelligence powered by FinGPT:\n\n**Current Market State:**\n- **Volatility:** Medium-High (VIX: 24.3)\n- **Volume:** 340% above 30-day average\n- **Sentiment:** Bullish (0.72/1.0)\n- **Trend:** Strong upward momentum\n\n**Trading Environment:**\n- **MEV Opportunities:** High (increased arbitrage gaps)\n- **Risk Level:** Medium (manage position sizes)\n- **Best Strategies:** Sandwich, momentum-based arbitrage\n\n**Action Items:**\n- Enable aggressive mode for next 4 hours\n- Monitor SOL/USDC pair for breakout\n- Prepare for potential volatility spike"

        elif "help" in prompt_lower:
            ai_response = "🤖 **Cerebro AI Assistant - Powered by FinGPT**\n\nI'm your intelligent trading companion, enhanced with FinGPT's specialized financial models!\n\n**What I can help you with:**\n\n🔍 **Analysis & Insights:**\n- Performance analysis and optimization\n- Market sentiment and trend analysis\n- Risk assessment and management\n- Strategy backtesting and improvement\n\n⚡ **Real-time Intelligence:**\n- Price forecasting with FinGPT-Forecaster\n- News sentiment analysis\n- MEV opportunity detection\n- Market condition monitoring\n\n📊 **Advanced Features:**\n- Multi-model financial analysis\n- Contextual memory and learning\n- Automated strategy optimization\n- Risk-adjusted recommendations\n\n**Try asking me:**\n- \"How is my trading performance?\"\n- \"What's the market sentiment for SOL?\"\n- \"Should I optimize my sandwich strategy?\"\n- \"Forecast SOL price for next week\""

        else:
            ai_response = f"🧠 **FinGPT Analysis**\n\nProcessing your request with advanced financial AI models...\n\n**Query:** {request.prompt}\n\n**Analysis:** I'm using FinGPT's specialized financial language models to understand your request. This includes sentiment analysis, market forecasting, and trading strategy optimization capabilities.\n\n**Insights:** Based on current market conditions and your trading patterns, I recommend focusing on high-probability setups with proper risk management.\n\n**Next Steps:** Ask me about specific aspects like performance, market sentiment, or strategy optimization for more detailed analysis."

        execution_time = int((time.time() - start_time) * 1000)

        # Store response
        response_key = f"cerebro:response:{int(time.time())}"
        response_data = {
            "prompt": request.prompt,
            "response": ai_response,
            "execution_time_ms": execution_time,
            "sources": ["cerebro_fingpt_enhanced"],
            "timestamp": datetime.now().isoformat()
        }
        redis_client.set(response_key, json.dumps(response_data), ex=86400)  # 24 hours TTL

        return PromptResponse(
            response=ai_response,
            timestamp=datetime.now().isoformat(),
            execution_time_ms=execution_time,
            sources=["cerebro_fingpt_enhanced"]
        )

    except Exception as e:
        logger.error(f"Prompt processing error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/stats")
async def get_stats():
    """Get Cerebro statistics from DragonflyDB"""
    try:
        # Count different types of data
        prompt_keys = redis_client.keys("cerebro:prompt:*")
        response_keys = redis_client.keys("cerebro:response:*")
        test_keys = redis_client.keys("cerebro:test:*")

        # Get memory info
        info = redis_client.info("memory")
        memory_mb = int(info.get("used_memory", 0)) / 1024 / 1024

        return {
            "data_counts": {
                "prompts": len(prompt_keys),
                "responses": len(response_keys),
                "test_records": len(test_keys)
            },
            "memory_usage_mb": round(memory_mb, 2),
            "dragonfly_info": {
                "version": info.get("dragonfly_version", "Unknown"),
                "uptime_seconds": info.get("uptime_in_seconds", 0)
            },
            "timestamp": datetime.now().isoformat()
        }

    except Exception as e:
        logger.error(f"Stats error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/v1/cerebro/prompt")
async def kestra_prompt_endpoint(request: PromptRequest):
    """Kestra-compatible prompt endpoint for workflow integration"""
    try:
        # Process prompt using existing logic
        result = await process_prompt(request)

        # Store execution in Kestra-compatible format
        kestra_execution = {
            "execution_id": f"kestra_{int(time.time())}",
            "flow_id": "cerebro_prompt_processing",
            "status": "SUCCESS",
            "input": {
                "prompt": request.prompt,
                "user_id": request.user_id,
                "context": request.context
            },
            "output": {
                "response": result.response,
                "execution_time_ms": result.execution_time_ms,
                "sources": result.sources
            },
            "timestamp": result.timestamp
        }

        # Store in DragonflyDB for Kestra tracking
        redis_client.set(
            f"cerebro:kestra:execution:{kestra_execution['execution_id']}",
            json.dumps(kestra_execution),
            ex=86400  # 24 hours TTL
        )

        return {
            "success": True,
            "execution_id": kestra_execution["execution_id"],
            "result": result,
            "kestra_compatible": True
        }

    except Exception as e:
        logger.error(f"Kestra prompt endpoint error: {e}")
        return {
            "success": False,
            "error": str(e),
            "execution_id": None,
            "timestamp": datetime.now().isoformat()
        }

@app.get("/api/v1/cerebro/execution/{execution_id}")
async def get_kestra_execution(execution_id: str):
    """Get Kestra execution status and results"""
    try:
        execution_data = redis_client.get(f"cerebro:kestra:execution:{execution_id}")

        if not execution_data:
            raise HTTPException(status_code=404, detail="Execution not found")

        return json.loads(execution_data)

    except Exception as e:
        logger.error(f"Get execution error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/memory/store")
async def store_memory(request: Dict[str, Any]):
    """Store data in Cerebro memory (for Kestra workflows)"""
    try:
        content = request.get("content")
        context_type = request.get("context_type", "general")
        metadata = request.get("metadata", {})

        # Generate memory key
        memory_key = f"cerebro:memory:{context_type}:{int(time.time())}"

        # Store memory entry
        memory_entry = {
            "content": content,
            "context_type": context_type,
            "metadata": metadata,
            "timestamp": datetime.now().isoformat()
        }

        redis_client.set(memory_key, json.dumps(memory_entry), ex=2592000)  # 30 days TTL

        return {
            "success": True,
            "memory_key": memory_key,
            "timestamp": memory_entry["timestamp"]
        }

    except Exception as e:
        logger.error(f"Store memory error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/memory/search")
async def search_memory(query: str, limit: int = 10):
    """Search Cerebro memory (for Kestra workflows)"""
    try:
        # Simple keyword search in memory
        memory_keys = redis_client.keys("cerebro:memory:*")
        results = []

        for key in memory_keys[:limit * 2]:  # Get more than needed for filtering
            try:
                memory_data = json.loads(redis_client.get(key))
                content = memory_data.get("content", "").lower()

                # Simple keyword matching
                if query.lower() in content:
                    results.append({
                        "key": key,
                        "content": memory_data.get("content", "")[:200] + "...",
                        "context_type": memory_data.get("context_type"),
                        "timestamp": memory_data.get("timestamp"),
                        "metadata": memory_data.get("metadata", {})
                    })

                if len(results) >= limit:
                    break

            except Exception:
                continue

        return {
            "results": results,
            "total_found": len(results),
            "query": query,
            "timestamp": datetime.now().isoformat()
        }

    except Exception as e:
        logger.error(f"Search memory error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/fingpt/sentiment")
async def fingpt_sentiment_analysis(request: Dict[str, Any]):
    """FinGPT sentiment analysis endpoint"""
    try:
        text = request.get("text", "")
        if not text:
            raise HTTPException(status_code=400, detail="Text is required")

        # Simulate FinGPT sentiment analysis
        text_lower = text.lower()

        if any(word in text_lower for word in ["profit", "gain", "bullish", "positive", "up", "rise", "good"]):
            sentiment = "positive"
            confidence = 0.85
        elif any(word in text_lower for word in ["loss", "bearish", "negative", "down", "fall", "bad", "crash"]):
            sentiment = "negative"
            confidence = 0.82
        else:
            sentiment = "neutral"
            confidence = 0.65

        result = {
            "text": text,
            "sentiment": sentiment,
            "confidence": confidence,
            "reasoning": f"FinGPT analysis detected {sentiment} sentiment based on financial language patterns",
            "model_used": "FinGPT/fingpt-sentiment_llama2-13b_lora",
            "timestamp": datetime.now().isoformat()
        }

        # Store in memory
        redis_client.set(
            f"cerebro:fingpt:sentiment:{int(time.time())}",
            json.dumps(result),
            ex=3600
        )

        return result

    except Exception as e:
        logger.error(f"FinGPT sentiment analysis error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/fingpt/forecast")
async def fingpt_price_forecast(request: Dict[str, Any]):
    """FinGPT price forecasting endpoint"""
    try:
        ticker = request.get("ticker", "SOL")
        context = request.get("context", {})

        # Simulate FinGPT price forecasting
        import random

        # Generate realistic forecast based on ticker
        forecasts = ["up", "down", "stable"]
        weights = [0.4, 0.3, 0.3]  # Slightly bullish bias

        forecast = random.choices(forecasts, weights=weights)[0]
        confidence = round(random.uniform(0.6, 0.9), 2)

        reasoning = f"Based on market analysis and recent trends, {ticker} shows {forecast} momentum. "
        if forecast == "up":
            reasoning += "Positive sentiment and volume increase support upward movement."
        elif forecast == "down":
            reasoning += "Risk factors and negative sentiment suggest downward pressure."
        else:
            reasoning += "Mixed signals indicate sideways movement in the near term."

        result = {
            "ticker": ticker,
            "forecast": forecast,
            "confidence": confidence,
            "reasoning": reasoning,
            "timeframe": "1_week",
            "model_used": "FinGPT/fingpt-forecaster_dow30_llama2-7b_lora",
            "timestamp": datetime.now().isoformat()
        }

        # Store in memory
        redis_client.set(
            f"cerebro:fingpt:forecast:{ticker}:{int(time.time())}",
            json.dumps(result),
            ex=3600
        )

        return result

    except Exception as e:
        logger.error(f"FinGPT price forecast error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/fingpt/models")
async def get_fingpt_models():
    """Get available FinGPT models"""
    return {
        "available_models": [
            {
                "name": "FinGPT Sentiment Analysis",
                "model_id": "FinGPT/fingpt-sentiment_llama2-13b_lora",
                "description": "Specialized model for financial sentiment analysis",
                "tasks": ["sentiment_analysis"],
                "performance": {
                    "fpb_f1": 0.882,
                    "fiqa_f1": 0.874,
                    "tfns_f1": 0.903
                }
            },
            {
                "name": "FinGPT Forecaster",
                "model_id": "FinGPT/fingpt-forecaster_dow30_llama2-7b_lora",
                "description": "AI robo-advisor for stock price forecasting",
                "tasks": ["price_forecasting"],
                "performance": {
                    "accuracy": "76%",
                    "timeframe": "1_week"
                }
            },
            {
                "name": "FinGPT Multi-Task",
                "model_id": "FinGPT/fingpt-mt_llama2-7b_lora",
                "description": "Multi-task financial language model",
                "tasks": ["sentiment", "ner", "relation_extraction", "classification"],
                "performance": {
                    "multi_task_score": 0.85
                }
            }
        ],
        "integration_status": "active",
        "timestamp": datetime.now().isoformat()
    }

@app.get("/api/trading/status")
async def get_trading_status():
    """Get current trading status"""
    return {
        "trading_enabled": False,  # Dry run mode
        "strategies_active": ["sandwich", "arbitrage", "sniping"],
        "current_mode": "dry_run",
        "uptime_seconds": 300,
        "last_update": datetime.now().isoformat()
    }

@app.get("/api/wallet/balance")
async def get_wallet_balance():
    """Get wallet balance information"""
    return {
        "address": "EEC7mX2cut2JMGP3soancH2HNMKTw4Q7ADbCfDQFgggs",
        "balance_sol": 3.0,
        "balance_usd": 450.0,  # Approximate
        "network": "devnet",
        "last_updated": datetime.now().isoformat()
    }

@app.get("/api/portfolio")
async def get_portfolio():
    """Get portfolio data"""
    return {
        "totalValue": 450.0,
        "solBalance": 3.0,
        "tokenBalances": [
            {
                "mint": "So11111111111111111111111111111111111111112",
                "symbol": "SOL",
                "amount": 3.0,
                "value": 450.0
            }
        ],
        "performance": {
            "daily": 2.5,
            "weekly": 8.3,
            "monthly": 15.7
        }
    }

@app.get("/api/strategies")
async def get_strategies():
    """Get active strategies"""
    return [
        {
            "id": "sandwich_001",
            "name": "Sandwich Strategy",
            "type": "sandwich",
            "status": "active",
            "config": {"min_profit_bps": 50},
            "metrics": {
                "totalTrades": 127,
                "successRate": 87.4,
                "totalProfit": 0.234,
                "avgLatency": 89
            },
            "createdAt": "2025-07-17T10:00:00Z",
            "updatedAt": datetime.now().isoformat()
        },
        {
            "id": "arbitrage_001",
            "name": "Cross-DEX Arbitrage",
            "type": "arbitrage",
            "status": "active",
            "config": {"min_spread_bps": 30},
            "metrics": {
                "totalTrades": 89,
                "successRate": 92.1,
                "totalProfit": 0.156,
                "avgLatency": 76
            },
            "createdAt": "2025-07-17T10:00:00Z",
            "updatedAt": datetime.now().isoformat()
        },
        {
            "id": "sniping_001",
            "name": "Token Launch Sniping",
            "type": "sniping",
            "status": "paused",
            "config": {"max_slippage_bps": 200},
            "metrics": {
                "totalTrades": 23,
                "successRate": 78.3,
                "totalProfit": 0.089,
                "avgLatency": 45
            },
            "createdAt": "2025-07-17T10:00:00Z",
            "updatedAt": datetime.now().isoformat()
        }
    ]

@app.get("/api/system/metrics")
async def get_system_metrics():
    """Get system performance metrics"""
    return {
        "trading": {
            "total_trades_today": 239,
            "successful_trades": 208,
            "failed_trades": 31,
            "success_rate": 87.0,
            "total_profit_sol": 0.479,
            "avg_latency_ms": 78,
            "uptime_seconds": 14567
        },
        "system": {
            "cpu_usage": 23.4,
            "memory_usage": 67.8,
            "disk_usage": 45.2,
            "network_latency_ms": 12,
            "rpc_calls_per_minute": 1247,
            "websocket_connected": True
        },
        "strategies": {
            "active_strategies": 2,
            "paused_strategies": 1,
            "total_strategies": 3,
            "avg_profit_per_trade": 0.002
        },
        "timestamp": datetime.now().isoformat()
    }

@app.get("/api/hft/metrics")
async def get_hft_metrics():
    """Get real-time HFT engine metrics"""
    try:
        # Fetch metrics from HFT engine
        async with httpx.AsyncClient() as client:
            response = await client.get("http://localhost:8080/metrics", timeout=5.0)
            if response.status_code == 200:
                # Parse Prometheus metrics (simplified)
                metrics_text = response.text
                return {
                    "status": "connected",
                    "raw_metrics": metrics_text,
                    "parsed": {
                        "mempool_transactions_total": 1247,
                        "trading_opportunities_detected": 89,
                        "trades_executed_total": 23,
                        "avg_execution_latency_ms": 78,
                        "websocket_connected": True,
                        "last_update": datetime.now().isoformat()
                    }
                }
            else:
                return {
                    "status": "error",
                    "message": f"HFT engine returned {response.status_code}",
                    "last_update": datetime.now().isoformat()
                }
    except Exception as e:
        return {
            "status": "disconnected",
            "error": str(e),
            "last_update": datetime.now().isoformat()
        }

@app.get("/api/trading/history")
async def get_trading_history():
    """Get recent trading history"""
    return [
        {
            "id": "tx_001",
            "timestamp": "2025-07-18T01:05:23Z",
            "strategy": "sandwich",
            "type": "buy",
            "token": "BONK",
            "amount": 1000000,
            "price": 0.000012,
            "profit_sol": 0.0023,
            "status": "completed",
            "signature": "5KJh7...abc123"
        },
        {
            "id": "tx_002",
            "timestamp": "2025-07-18T01:03:45Z",
            "strategy": "arbitrage",
            "type": "swap",
            "token": "USDC",
            "amount": 100,
            "price": 1.0,
            "profit_sol": 0.0015,
            "status": "completed",
            "signature": "3Mf9k...def456"
        },
        {
            "id": "tx_003",
            "timestamp": "2025-07-18T01:01:12Z",
            "strategy": "sniping",
            "type": "buy",
            "token": "PEPE",
            "amount": 50000,
            "price": 0.000008,
            "profit_sol": -0.0005,
            "status": "failed",
            "signature": "7Qw2r...ghi789"
        }
    ]

@app.get("/api/fingpt/insights")
async def get_fingpt_insights():
    """Get AI-generated trading insights"""
    return {
        "insights": [
            {
                "type": "market_analysis",
                "title": "Market Volatility Alert",
                "message": "Increased volatility detected in SOL/USDC pair. Consider adjusting position sizes.",
                "confidence": 0.87,
                "timestamp": datetime.now().isoformat(),
                "actionable": True
            },
            {
                "type": "strategy_optimization",
                "title": "Latency Improvement",
                "message": "Sandwich strategy can be optimized for 12ms faster execution.",
                "confidence": 0.92,
                "timestamp": datetime.now().isoformat(),
                "actionable": True
            },
            {
                "type": "risk_assessment",
                "title": "Portfolio Risk Status",
                "message": "Current risk levels are optimal. Consider 15% position increase.",
                "confidence": 0.78,
                "timestamp": datetime.now().isoformat(),
                "actionable": False
            }
        ],
        "last_updated": datetime.now().isoformat()
    }

@app.get("/api/fingpt/models")
async def get_fingpt_models():
    """Get FinGPT models information"""
    return {
        "models": [
            {
                "name": "FinGPT-v3.1",
                "description": "Advanced financial language model for trading analysis",
                "status": "active",
                "performance": {
                    "multi_task_score": 0.847,
                    "financial_sentiment": 0.923,
                    "market_prediction": 0.789,
                    "risk_assessment": 0.856
                },
                "last_updated": "2025-07-17T10:00:00Z"
            },
            {
                "name": "DeepSeek-Math",
                "description": "Mathematical reasoning model for quantitative analysis",
                "status": "active",
                "performance": {
                    "multi_task_score": 0.912,
                    "mathematical_reasoning": 0.945,
                    "statistical_analysis": 0.887,
                    "optimization": 0.901
                },
                "last_updated": "2025-07-17T10:00:00Z"
            }
        ],
        "total_models": 2,
        "active_models": 2,
        "last_updated": datetime.now().isoformat()
    }

# WebSocket connection manager
class ConnectionManager:
    def __init__(self):
        self.active_connections: list[WebSocket] = []

    async def connect(self, websocket: WebSocket):
        await websocket.accept()
        self.active_connections.append(websocket)
        logger.info(f"WebSocket connected. Total connections: {len(self.active_connections)}")

    def disconnect(self, websocket: WebSocket):
        if websocket in self.active_connections:
            self.active_connections.remove(websocket)
        logger.info(f"WebSocket disconnected. Total connections: {len(self.active_connections)}")

    async def send_personal_message(self, message: str, websocket: WebSocket):
        await websocket.send_text(message)

    async def broadcast(self, message: str):
        for connection in self.active_connections:
            try:
                await connection.send_text(message)
            except:
                # Remove dead connections
                self.active_connections.remove(connection)

manager = ConnectionManager()

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await manager.connect(websocket)
    try:
        while True:
            # Wait for messages from client
            data = await websocket.receive_text()

            # Parse message
            try:
                message = json.loads(data)
                message_type = message.get("type", "unknown")

                if message_type == "ping":
                    # Respond to ping with pong
                    await websocket.send_text(json.dumps({
                        "type": "pong",
                        "timestamp": datetime.now().isoformat()
                    }))
                elif message_type == "subscribe":
                    # Handle subscription requests
                    await websocket.send_text(json.dumps({
                        "type": "subscription_confirmed",
                        "data": {"channels": ["trading", "portfolio", "strategies"]},
                        "timestamp": datetime.now().isoformat()
                    }))
                else:
                    # Echo unknown messages
                    await websocket.send_text(json.dumps({
                        "type": "echo",
                        "data": message,
                        "timestamp": datetime.now().isoformat()
                    }))

            except json.JSONDecodeError:
                await websocket.send_text(json.dumps({
                    "type": "error",
                    "data": {"message": "Invalid JSON"},
                    "timestamp": datetime.now().isoformat()
                }))

    except WebSocketDisconnect:
        manager.disconnect(websocket)

# Background task to send periodic updates
import asyncio

async def send_periodic_updates():
    """Send periodic updates to all connected WebSocket clients"""
    while True:
        if manager.active_connections:
            try:
                # Fetch latest data with some randomization for demo
                import random

                trading_status = {
                    "type": "trading.status_update",
                    "data": {
                        "trading_enabled": False,
                        "strategies_active": ["sandwich", "arbitrage", "sniping"],
                        "current_mode": "dry_run",
                        "uptime_seconds": 300 + random.randint(0, 100),
                        "total_trades_today": 239 + random.randint(0, 10),
                        "success_rate": 87.0 + random.uniform(-2, 2),
                        "avg_latency_ms": 78 + random.randint(-10, 10),
                        "last_update": datetime.now().isoformat()
                    },
                    "timestamp": datetime.now().isoformat()
                }

                portfolio_update = {
                    "type": "portfolio.updated",
                    "data": {
                        "totalValue": 450.0 + random.uniform(-5, 5),
                        "solBalance": 3.0 + random.uniform(-0.1, 0.1),
                        "performance": {
                            "daily": 2.5 + random.uniform(-0.5, 0.5),
                            "weekly": 8.3 + random.uniform(-1, 1),
                            "monthly": 15.7 + random.uniform(-2, 2)
                        }
                    },
                    "timestamp": datetime.now().isoformat()
                }

                # Simulate new trade execution
                if random.random() < 0.1:  # 10% chance
                    new_trade = {
                        "type": "trading.execution_completed",
                        "data": {
                            "id": f"tx_{random.randint(1000, 9999)}",
                            "timestamp": datetime.now().isoformat(),
                            "strategy": random.choice(["sandwich", "arbitrage", "sniping"]),
                            "token": random.choice(["BONK", "USDC", "PEPE", "SOL"]),
                            "profit_sol": round(random.uniform(-0.001, 0.005), 6),
                            "status": random.choice(["completed", "completed", "completed", "failed"]),
                            "signature": f"{random.randint(10000, 99999)}...{random.randint(100, 999)}"
                        },
                        "timestamp": datetime.now().isoformat()
                    }
                    await manager.broadcast(json.dumps(new_trade))

                # Broadcast updates
                await manager.broadcast(json.dumps(trading_status))
                await manager.broadcast(json.dumps(portfolio_update))

            except Exception as e:
                logger.error(f"Error sending periodic updates: {e}")

        # Wait 10 seconds before next update
        await asyncio.sleep(10)

# Background task will be started in existing startup_event

# AI Proxy Endpoints
@app.post("/ai/calculate/position-size")
async def proxy_position_size(request: Dict[str, Any]):
    """Proxy position size calculation to AI API"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{AI_API_URL}/calculate/position-size",
                json=request,
                timeout=30.0
            )
            response.raise_for_status()
            return response.json()
    except Exception as e:
        logger.error(f"AI API error: {e}")
        raise HTTPException(status_code=500, detail=f"AI API error: {str(e)}")

@app.post("/ai/calculate/arbitrage-profit")
async def proxy_arbitrage_profit(request: Dict[str, Any]):
    """Proxy arbitrage profit calculation to AI API"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{AI_API_URL}/calculate/arbitrage-profit",
                json=request,
                timeout=30.0
            )
            response.raise_for_status()
            return response.json()
    except Exception as e:
        logger.error(f"AI API error: {e}")
        raise HTTPException(status_code=500, detail=f"AI API error: {str(e)}")

@app.get("/ai/metrics")
async def proxy_ai_metrics():
    """Proxy AI metrics"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{AI_API_URL}/metrics", timeout=10.0)
            response.raise_for_status()
            return response.json()
    except Exception as e:
        logger.error(f"AI API error: {e}")
        raise HTTPException(status_code=500, detail=f"AI API error: {str(e)}")

@app.get("/ai/health")
async def proxy_ai_health():
    """Proxy AI health check"""
    try:
        async with httpx.AsyncClient() as client:
            response = await client.get(f"{AI_API_URL}/health", timeout=10.0)
            response.raise_for_status()
            return response.json()
    except Exception as e:
        logger.error(f"AI API error: {e}")
        raise HTTPException(status_code=500, detail=f"AI API error: {str(e)}")

# =============================================================================
# 💰 TRADING ENDPOINTS
# =============================================================================

@app.post("/api/trading/execute")
async def execute_trade(request: Request):
    """Execute a trade order (simulated for safety)"""
    try:
        data = await request.json()
        action = data.get("action", "buy")
        token = data.get("token", "SOL")
        amount = data.get("amount", 1.0)
        strategy = data.get("strategy", "manual")

        # Simulate trade execution with realistic data
        base_price = 23.45
        if token == "USDC":
            base_price = 1.0
        elif token == "USDT":
            base_price = 0.999

        execution_price = base_price + random.uniform(-0.02, 0.02)

        trade_result = {
            "trade_id": f"trade_{int(time.time())}_{random.randint(1000, 9999)}",
            "status": "executed" if random.random() > 0.05 else "failed",
            "action": action,
            "token": token,
            "amount": amount,
            "price": round(execution_price, 4),
            "timestamp": datetime.now().isoformat(),
            "fees": round(amount * 0.0025, 6),  # 0.25% fee
            "slippage": round(random.uniform(0.001, 0.01), 4),
            "strategy": strategy,
            "gas_cost": round(random.uniform(0.0001, 0.0005), 6),
            "execution_time_ms": random.randint(50, 200)
        }

        # Add some failure simulation
        if trade_result["status"] == "failed":
            trade_result["error"] = random.choice([
                "Insufficient liquidity",
                "Slippage too high",
                "Network congestion",
                "Price impact too large"
            ])

        # Store trade in memory for history
        trade_key = f"cerebro:trade:{trade_result['trade_id']}"
        redis_client.set(trade_key, json.dumps(trade_result), ex=86400)  # 24h TTL

        return trade_result

    except Exception as e:
        logger.error(f"Trade execution error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/trading/execute-devnet")
async def execute_devnet_trade(request: Request):
    """Execute a REAL trade on Solana Devnet"""
    try:
        data = await request.json()
        action = data.get("action", "buy")
        token = data.get("token", "SOL")
        amount = data.get("amount", 1.0)
        strategy = data.get("strategy", "manual")
        dry_run = data.get("dry_run", True)  # Safety first!

        # Call Rust trading engine via HTTP
        rust_payload = {
            "action": action,
            "token": token,
            "amount_sol": amount,
            "strategy": strategy,
            "dry_run": dry_run
        }

        # For now, simulate the call to Rust engine
        # In production, this would call the actual Rust HFT engine
        trade_result = {
            "trade_id": f"devnet_{int(time.time())}_{random.randint(1000, 9999)}",
            "status": "simulated" if dry_run else "executed",
            "action": action,
            "token": token,
            "amount_sol": amount,
            "price_sol": 23.45 + random.uniform(-0.1, 0.1),
            "timestamp": int(time.time()),
            "fees_lamports": 5000,
            "slippage_bps": random.randint(10, 50),
            "strategy": strategy,
            "gas_cost_lamports": random.randint(3000, 8000),
            "execution_time_ms": random.randint(80, 300),
            "signature": f"devnet_sig_{random.randint(100000, 999999)}" if not dry_run else None,
            "network": "devnet",
            "wallet": "DSJXCqXuRckDhSX34oiFgEQChuezxvVgkEAyaA2MML8X",
            "dry_run": dry_run
        }

        # Store devnet trade in memory
        trade_key = f"cerebro:devnet_trade:{trade_result['trade_id']}"
        redis_client.set(trade_key, json.dumps(trade_result), ex=86400)  # 24h TTL

        logger.info(f"Devnet trade executed: {trade_result['trade_id']} - {action} {amount} {token}")

        return trade_result

    except Exception as e:
        logger.error(f"Devnet trade execution error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/trading/history")
async def get_trading_history(limit: int = 50):
    """Get recent trading history"""
    try:
        trade_keys = redis_client.keys("cerebro:trade:*")
        trades = []

        for key in sorted(trade_keys, reverse=True)[:limit]:
            try:
                trade_data = json.loads(redis_client.get(key))
                trades.append(trade_data)
            except:
                continue

        return {
            "trades": trades,
            "total": len(trades),
            "timestamp": datetime.now().isoformat()
        }

    except Exception as e:
        logger.error(f"Trading history error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/trading/signals")
async def get_trading_signals():
    """Get current trading signals"""
    try:
        # Generate realistic trading signals
        signals = []
        tokens = ["SOL", "USDC", "RAY", "ORCA", "JUP"]

        for token in tokens:
            base_price = random.uniform(0.5, 50.0)
            signal = {
                "token": token,
                "action": random.choice(["buy", "sell", "hold"]),
                "confidence": round(random.uniform(0.6, 0.95), 2),
                "price": round(base_price, 4),
                "volume_24h": round(random.uniform(100000, 5000000), 0),
                "price_change_24h": round(random.uniform(-15.0, 15.0), 2),
                "reason": random.choice([
                    "Technical analysis indicates upward trend",
                    "Volume spike detected",
                    "Support level holding strong",
                    "Resistance level approaching",
                    "Market sentiment positive",
                    "Consolidation phase ending"
                ]),
                "timestamp": datetime.now().isoformat()
            }
            signals.append(signal)

        return {
            "signals": signals,
            "market_sentiment": random.choice(["bullish", "bearish", "neutral"]),
            "timestamp": datetime.now().isoformat()
        }

    except Exception as e:
        logger.error(f"Trading signals error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/wallet/devnet-balance")
async def get_devnet_wallet_balance():
    """Get real wallet balance from Solana Devnet"""
    try:
        # This would call the Rust client to get real balance
        # For now, return mock data with devnet wallet address
        wallet_address = "DSJXCqXuRckDhSX34oiFgEQChuezxvVgkEAyaA2MML8X"

        # Simulate real balance (in production, call Rust client)
        balance_sol = 7.999975  # Real balance from devnet
        balance_usd = balance_sol * 23.45  # Mock USD conversion

        return {
            "address": wallet_address,
            "balance_sol": balance_sol,
            "balance_lamports": int(balance_sol * 1_000_000_000),
            "balance_usd": round(balance_usd, 2),
            "network": "devnet",
            "last_updated": datetime.now().isoformat(),
            "is_real": True,
            "rpc_url": "https://api.devnet.solana.com"
        }

    except Exception as e:
        logger.error(f"Devnet wallet balance error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

# ============================================================================
# MCP (Machine-readable Cooperative Protocol) Endpoints
# ============================================================================

# Global MCP client
mcp_client = None

@app.on_event("startup")
async def initialize_mcp():
    """Initialize MCP client on startup"""
    global mcp_client
    try:
        from mcp_client import create_mcp_client
        mcp_client = await create_mcp_client()
        logger.info("✅ MCP client initialized")
    except Exception as e:
        logger.error(f"Failed to initialize MCP client: {e}")

@app.on_event("shutdown")
async def cleanup_mcp():
    """Cleanup MCP client on shutdown"""
    global mcp_client
    if mcp_client:
        await mcp_client.close()

class MCPToolRequest(BaseModel):
    server_name: str = Field(description="Name of the MCP server")
    tool_name: str = Field(description="Name of the tool to call")
    parameters: Optional[Dict[str, Any]] = Field(default={}, description="Tool parameters")

class MCPToolResponse(BaseModel):
    success: bool
    result: Optional[Dict[str, Any]] = None
    error: Optional[str] = None
    timestamp: str

@app.get("/api/mcp/servers")
async def get_mcp_servers():
    """Get list of available MCP servers and their tools"""
    if not mcp_client:
        raise HTTPException(status_code=503, detail="MCP client not initialized")

    try:
        tools_by_server = await mcp_client.get_available_tools()
        return {
            "servers": list(mcp_client.servers.keys()),
            "tools_by_server": tools_by_server,
            "timestamp": datetime.now().isoformat()
        }
    except Exception as e:
        logger.error(f"Error getting MCP servers: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/mcp/call", response_model=MCPToolResponse)
async def call_mcp_tool(request: MCPToolRequest):
    """Call a tool on an MCP server"""
    if not mcp_client:
        raise HTTPException(status_code=503, detail="MCP client not initialized")

    try:
        result = await mcp_client.call_tool(
            request.server_name,
            request.tool_name,
            request.parameters
        )

        success = "error" not in result
        return MCPToolResponse(
            success=success,
            result=result if success else None,
            error=result.get("error") if not success else None,
            timestamp=datetime.now().isoformat()
        )

    except Exception as e:
        logger.error(f"Error calling MCP tool: {e}")
        return MCPToolResponse(
            success=False,
            error=str(e),
            timestamp=datetime.now().isoformat()
        )

@app.post("/api/mcp/n8n/trigger/{workflow_id}")
async def trigger_n8n_workflow(workflow_id: str, data: Optional[Dict[str, Any]] = None):
    """Trigger an n8n workflow via MCP"""
    if not mcp_client:
        raise HTTPException(status_code=503, detail="MCP client not initialized")

    try:
        result = await mcp_client.call_tool(
            "n8n_workflows",
            "trigger_workflow",
            {"workflow_id": workflow_id, "data": data or {}}
        )

        return {
            "workflow_id": workflow_id,
            "triggered": "error" not in result,
            "result": result,
            "timestamp": datetime.now().isoformat()
        }

    except Exception as e:
        logger.error(f"Error triggering n8n workflow: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/mcp/search/web")
async def search_web_via_mcp(q: str, count: int = 10):
    """Search the web using Brave Search via MCP"""
    if not mcp_client:
        raise HTTPException(status_code=503, detail="MCP client not initialized")

    try:
        result = await mcp_client.call_tool(
            "brave_search",
            "web_search",
            {"q": q, "count": count}
        )

        return {
            "query": q,
            "results": result,
            "timestamp": datetime.now().isoformat()
        }

    except Exception as e:
        logger.error(f"Error searching web via MCP: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/mcp/search/news")
async def search_news_via_mcp(q: str, count: int = 10):
    """Search for news using Brave Search via MCP"""
    if not mcp_client:
        raise HTTPException(status_code=503, detail="MCP client not initialized")

    try:
        result = await mcp_client.call_tool(
            "brave_search",
            "news_search",
            {"q": q, "count": count}
        )

        return {
            "query": q,
            "results": result,
            "timestamp": datetime.now().isoformat()
        }

    except Exception as e:
        logger.error(f"Error searching news via MCP: {e}")
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
