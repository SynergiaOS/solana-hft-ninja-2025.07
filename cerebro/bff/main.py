#!/usr/bin/env python3
"""
Project Cerebro - Backend for Frontend (BFF)
FastAPI server providing unified API for Cerebro components
"""

from fastapi import FastAPI, HTTPException, BackgroundTasks
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel
from typing import Dict, List, Any, Optional
import asyncio
import redis
import httpx
import json
import time
import logging
import os
from datetime import datetime

# Import memory API
import sys
sys.path.append('..')
from memory.api import router as memory_router

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Environment variables
DRAGONFLY_URL = os.getenv("DRAGONFLY_URL", "redis://:cerebro_secure_2025@localhost:6379")
HFT_NINJA_API_URL = os.getenv("HFT_NINJA_API_URL", "http://localhost:8080")
KESTRA_API_URL = os.getenv("KESTRA_API_URL", "http://localhost:8081")
ENVIRONMENT = os.getenv("ENVIRONMENT", "development")

# FastAPI app
app = FastAPI(
    title="Project Cerebro BFF",
    description="Backend-for-Frontend for Solana HFT Ninja AI Assistant",
    version="1.0.0",
    docs_url="/docs" if ENVIRONMENT == "development" else None
)

# Include memory router
app.include_router(memory_router)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000", "http://localhost:8080"],
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

class PromptRequest(BaseModel):
    prompt: str
    context: Optional[Dict[str, Any]] = None
    user_id: Optional[str] = "default"

class PromptResponse(BaseModel):
    response: str
    timestamp: str
    execution_time_ms: int
    sources: List[str]

class ActionRequest(BaseModel):
    action_type: str
    parameters: Dict[str, Any]
    confirmation: bool = False

class DashboardData(BaseModel):
    hft_stats: Dict[str, Any]
    cerebro_status: Dict[str, Any]
    recent_analyses: List[Dict[str, Any]]
    suggestions: List[Dict[str, Any]]

@app.on_event("startup")
async def startup_event():
    """Initialize connections on startup"""
    global redis_client, http_client

    try:
        # Initialize Redis connection for DragonflyDB Cloud
        if DRAGONFLY_URL.startswith('rediss://'):
            # DragonflyDB Cloud with SSL
            import urllib.parse
            parsed = urllib.parse.urlparse(DRAGONFLY_URL)

            redis_client = redis.Redis(
                host=parsed.hostname,
                port=parsed.port or 6385,
                password=parsed.password,
                ssl=True,
                ssl_cert_reqs=None,
                decode_responses=True
            )
        else:
            # Local Redis/DragonflyDB
            redis_client = redis.from_url(DRAGONFLY_URL, decode_responses=True)

        # Test connection
        redis_client.ping()
        logger.info("✅ Connected to DragonflyDB Cloud")

        # Initialize HTTP client
        http_client = httpx.AsyncClient(timeout=30.0)
        logger.info("✅ HTTP client initialized")

        # Test HFT Ninja connection
        try:
            response = await http_client.get(f"{HFT_NINJA_API_URL}/health")
            if response.status_code == 200:
                logger.info("✅ HFT Ninja API accessible")
            else:
                logger.warning(f"⚠️ HFT Ninja API returned {response.status_code}")
        except Exception as e:
            logger.warning(f"⚠️ HFT Ninja API not accessible: {e}")

        logger.info("🚀 Cerebro BFF started successfully")

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

# API Endpoints

@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint"""
    services = {}

    # Check DragonflyDB
    try:
        redis_client.ping()
        services["dragonflydb"] = "healthy"
    except:
        services["dragonflydb"] = "unhealthy"

    # Check HFT Ninja API
    try:
        response = await http_client.get(f"{HFT_NINJA_API_URL}/health", timeout=5.0)
        services["hft_ninja"] = "healthy" if response.status_code == 200 else "unhealthy"
    except:
        services["hft_ninja"] = "unhealthy"

    # Check Kestra
    try:
        response = await http_client.get(f"{KESTRA_API_URL}/api/v1/health", timeout=5.0)
        services["kestra"] = "healthy" if response.status_code == 200 else "unhealthy"
    except:
        services["kestra"] = "unhealthy"

    overall_status = "healthy" if all(s == "healthy" for s in services.values()) else "degraded"

    return HealthResponse(
        status=overall_status,
        timestamp=datetime.now().isoformat(),
        services=services,
        version="1.0.0"
    )

@app.get("/api/dashboard", response_model=DashboardData)
async def get_dashboard_data():
    """Get aggregated dashboard data"""
    try:
        # Get HFT Ninja stats
        hft_stats = {}
        try:
            response = await http_client.get(f"{HFT_NINJA_API_URL}/metrics")
            if response.status_code == 200:
                hft_stats = response.json()
        except Exception as e:
            logger.warning(f"Failed to get HFT stats: {e}")
            hft_stats = {"error": "HFT Ninja unavailable"}

        # Get Cerebro status from Redis
        cerebro_status = {
            "active_analyses": 0,
            "memory_usage": "0MB",
            "last_analysis": None
        }

        try:
            # Count active analyses
            analysis_keys = await redis_client.keys("cerebro:analysis:*")
            cerebro_status["active_analyses"] = len(analysis_keys)

            # Get memory info
            info = await redis_client.info("memory")
            memory_mb = int(info.get("used_memory", 0)) / 1024 / 1024
            cerebro_status["memory_usage"] = f"{memory_mb:.1f}MB"

            # Get last analysis timestamp
            last_analysis = await redis_client.get("cerebro:last_analysis")
            if last_analysis:
                cerebro_status["last_analysis"] = last_analysis

        except Exception as e:
            logger.warning(f"Failed to get Cerebro status: {e}")

        # Get recent analyses
        recent_analyses = []
        try:
            analysis_keys = await redis_client.keys("cerebro:analysis:*")
            for key in analysis_keys[-5:]:  # Last 5 analyses
                analysis_data = await redis_client.get(key)
                if analysis_data:
                    recent_analyses.append(json.loads(analysis_data))
        except Exception as e:
            logger.warning(f"Failed to get recent analyses: {e}")

        # Get suggestions
        suggestions = []
        try:
            suggestion_keys = await redis_client.keys("cerebro:suggestion:*")
            for key in suggestion_keys:
                suggestion_data = await redis_client.get(key)
                if suggestion_data:
                    suggestions.append(json.loads(suggestion_data))
        except Exception as e:
            logger.warning(f"Failed to get suggestions: {e}")

        return DashboardData(
            hft_stats=hft_stats,
            cerebro_status=cerebro_status,
            recent_analyses=recent_analyses,
            suggestions=suggestions
        )

    except Exception as e:
        logger.error(f"Dashboard data error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/prompt", response_model=PromptResponse)
async def process_prompt(request: PromptRequest):
    """Process user prompt through Cerebro AI"""
    start_time = time.time()

    try:
        # Store prompt in memory for context
        prompt_key = f"cerebro:prompt:{int(time.time())}"
        prompt_data = {
            "prompt": request.prompt,
            "user_id": request.user_id,
            "timestamp": datetime.now().isoformat(),
            "context": request.context or {}
        }
        await redis_client.set(prompt_key, json.dumps(prompt_data), ex=3600)  # 1 hour TTL

        # Send to Kestra for processing
        kestra_payload = {
            "prompt": request.prompt,
            "context": request.context,
            "user_id": request.user_id
        }

        try:
            response = await http_client.post(
                f"{KESTRA_API_URL}/api/v1/cerebro/prompt",
                json=kestra_payload,
                timeout=60.0
            )

            if response.status_code == 200:
                result = response.json()
                ai_response = result.get("response", "No response from AI")
                sources = result.get("sources", ["kestra"])
            else:
                # Fallback response
                ai_response = "I'm currently processing your request. Please try again in a moment."
                sources = ["fallback"]

        except Exception as e:
            logger.warning(f"Kestra unavailable, using fallback: {e}")
            # Simple fallback logic
            if "profit" in request.prompt.lower() or "loss" in request.prompt.lower():
                ai_response = "I'm analyzing your trading performance. Based on recent data, I recommend reviewing your risk management settings."
            elif "strategy" in request.prompt.lower():
                ai_response = "I'm evaluating your current strategies. Consider optimizing your MEV detection parameters for better performance."
            else:
                ai_response = "I'm processing your request. The Cerebro AI system is learning from your trading patterns to provide better insights."
            sources = ["fallback_logic"]

        execution_time = int((time.time() - start_time) * 1000)

        # Store response for history
        response_key = f"cerebro:response:{int(time.time())}"
        response_data = {
            "prompt": request.prompt,
            "response": ai_response,
            "execution_time_ms": execution_time,
            "sources": sources,
            "timestamp": datetime.now().isoformat()
        }
        await redis_client.set(response_key, json.dumps(response_data), ex=86400)  # 24 hours TTL

        return PromptResponse(
            response=ai_response,
            timestamp=datetime.now().isoformat(),
            execution_time_ms=execution_time,
            sources=sources
        )

    except Exception as e:
        logger.error(f"Prompt processing error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/api/action")
async def execute_action(request: ActionRequest):
    """Execute action on HFT Ninja system"""
    try:
        if not request.confirmation:
            return {"status": "confirmation_required", "message": "Please confirm this action"}

        # Log action
        action_key = f"cerebro:action:{int(time.time())}"
        action_data = {
            "action_type": request.action_type,
            "parameters": request.parameters,
            "timestamp": datetime.now().isoformat(),
            "status": "executing"
        }
        await redis_client.set(action_key, json.dumps(action_data), ex=3600)

        # Execute based on action type
        if request.action_type == "update_config":
            # Update HFT Ninja configuration
            response = await http_client.post(
                f"{HFT_NINJA_API_URL}/api/config/update",
                json=request.parameters
            )

            if response.status_code == 200:
                action_data["status"] = "completed"
                action_data["result"] = response.json()
            else:
                action_data["status"] = "failed"
                action_data["error"] = f"HTTP {response.status_code}"

        elif request.action_type == "restart_strategy":
            # Restart specific strategy
            strategy_name = request.parameters.get("strategy_name")
            response = await http_client.post(
                f"{HFT_NINJA_API_URL}/api/strategy/{strategy_name}/restart"
            )

            if response.status_code == 200:
                action_data["status"] = "completed"
                action_data["result"] = {"message": f"Strategy {strategy_name} restarted"}
            else:
                action_data["status"] = "failed"
                action_data["error"] = f"HTTP {response.status_code}"

        else:
            action_data["status"] = "failed"
            action_data["error"] = f"Unknown action type: {request.action_type}"

        # Update action status
        await redis_client.set(action_key, json.dumps(action_data), ex=3600)

        return {
            "status": action_data["status"],
            "action_id": action_key,
            "result": action_data.get("result"),
            "error": action_data.get("error")
        }

    except Exception as e:
        logger.error(f"Action execution error: {e}")
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)