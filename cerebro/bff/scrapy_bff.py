#!/usr/bin/env python3
"""
Simplified BFF for Scrapy integration testing
"""

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Dict, Any, Optional
import redis
import json
import os
from datetime import datetime
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# FastAPI app
app = FastAPI(
    title="Scrapy BFF",
    description="Simplified Backend-for-Frontend for Scrapy integration",
    version="1.0.0"
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Redis connection
redis_client = None

class ScrapyAlert(BaseModel):
    type: str
    message: Optional[str] = None
    source: Optional[str] = None
    timestamp: Optional[str] = None
    project: Optional[str] = None
    risk_score: Optional[float] = None
    issues: Optional[List[str]] = None
    severity: Optional[str] = "medium"

class ScrapyAlertsRequest(BaseModel):
    alerts: List[ScrapyAlert]

@app.on_event("startup")
async def startup_event():
    """Initialize Redis connection"""
    global redis_client
    
    try:
        redis_client = redis.Redis(
            host='localhost', 
            port=6379, 
            db=2, 
            decode_responses=True
        )
        redis_client.ping()
        logger.info("✅ Connected to Redis")
    except Exception as e:
        logger.error(f"❌ Redis connection failed: {e}")
        redis_client = None

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    redis_status = "healthy" if redis_client else "unhealthy"
    
    return {
        "status": "healthy" if redis_status == "healthy" else "degraded",
        "timestamp": datetime.now().isoformat(),
        "services": {
            "redis": redis_status
        }
    }

@app.post("/api/scrapy/alerts")
async def receive_scrapy_alerts(request: ScrapyAlertsRequest):
    """Receive alerts from Scrapy spiders"""
    if not redis_client:
        raise HTTPException(status_code=503, detail="Redis connection unavailable")
    
    try:
        # Store alerts in Redis
        timestamp = datetime.now().isoformat()
        key = f"alerts:scrapy:{timestamp}"
        
        redis_client.setex(
            key, 
            3600,  # 1 hour expiry
            json.dumps([alert.dict() for alert in request.alerts])
        )
        
        # Store in recent alerts list
        redis_client.lpush("alerts:scrapy:recent", key)
        redis_client.ltrim("alerts:scrapy:recent", 0, 99)  # Keep last 100
        
        # Count alerts by severity
        severity_counts = {}
        for alert in request.alerts:
            severity = alert.severity or "medium"
            severity_counts[severity] = severity_counts.get(severity, 0) + 1
        
        return {
            "status": "success",
            "alerts_received": len(request.alerts),
            "severity_breakdown": severity_counts,
            "timestamp": timestamp
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/scrapy/alerts/recent")
async def get_recent_alerts(limit: int = 20):
    """Get recent Scrapy alerts"""
    if not redis_client:
        raise HTTPException(status_code=503, detail="Redis connection unavailable")
    
    try:
        alert_keys = redis_client.lrange("alerts:scrapy:recent", 0, limit - 1)
        alerts = []
        
        for key in alert_keys:
            alert_data = redis_client.get(key)
            if alert_data:
                batch_alerts = json.loads(alert_data)
                for alert in batch_alerts:
                    alert['batch_timestamp'] = key.split(":")[-1]
                alerts.extend(batch_alerts)
        
        # Sort by timestamp (newest first)
        alerts.sort(key=lambda x: x.get('timestamp', ''), reverse=True)
        
        return {
            "alerts": alerts[:limit],
            "count": len(alerts[:limit]),
            "total_batches": len(alert_keys)
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/scrapy/data/{spider_name}")
async def get_spider_data(spider_name: str, limit: int = 100):
    """Get latest data from specific spider"""
    if not redis_client:
        raise HTTPException(status_code=503, detail="Redis connection unavailable")
    
    try:
        pattern = f"scrapy:{spider_name}:*"
        keys = redis_client.keys(pattern)
        
        if not keys:
            return {"data": [], "count": 0, "message": f"No data found for spider: {spider_name}"}
        
        # Get latest key
        latest_key = sorted(keys)[-1]
        data_str = redis_client.get(latest_key)
        
        if not data_str:
            return {"data": [], "count": 0, "message": "Data expired or unavailable"}
        
        data = json.loads(data_str)
        
        # Ensure data is a list
        if not isinstance(data, list):
            data = [data]
        
        return {
            "data": data[:limit],
            "count": len(data),
            "total_available": len(data),
            "timestamp": latest_key.split(":")[-1],
            "spider": spider_name
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/scrapy/status")
async def get_scrapy_status():
    """Get Scrapy system status"""
    if not redis_client:
        return {
            "status": "error",
            "message": "Redis connection unavailable",
            "spiders": {}
        }
    
    try:
        spiders = ["discord_monitor", "project_auditor", "news_aggregator", "dex_monitor"]
        status = {}
        
        for spider in spiders:
            keys = redis_client.keys(f"scrapy:{spider}:*")
            
            if keys:
                # Get latest data
                latest_key = sorted(keys)[-1]
                timestamp = latest_key.split(":")[-1]
                
                try:
                    data_str = redis_client.get(latest_key)
                    if data_str:
                        data = json.loads(data_str)
                        data_count = len(data) if isinstance(data, list) else 1
                    else:
                        data_count = 0
                except:
                    data_count = 0
                
                # Calculate time since last run
                try:
                    last_run_dt = datetime.fromisoformat(timestamp.replace('Z', '+00:00'))
                    time_diff = datetime.now() - last_run_dt.replace(tzinfo=None)
                    hours_since = time_diff.total_seconds() / 3600
                    
                    if hours_since < 2:
                        spider_status = "active"
                    elif hours_since < 24:
                        spider_status = "stale"
                    else:
                        spider_status = "inactive"
                except:
                    spider_status = "unknown"
                
                status[spider] = {
                    "last_run": timestamp,
                    "data_count": data_count,
                    "status": spider_status,
                    "total_runs": len(keys)
                }
            else:
                status[spider] = {
                    "last_run": None,
                    "data_count": 0,
                    "status": "never_run",
                    "total_runs": 0
                }
        
        # Overall system status
        active_spiders = sum(1 for s in status.values() if s["status"] == "active")
        total_spiders = len(spiders)
        
        return {
            "status": "healthy" if active_spiders > 0 else "degraded",
            "active_spiders": active_spiders,
            "total_spiders": total_spiders,
            "spiders": status,
            "last_check": datetime.now().isoformat()
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/api/scrapy/metrics")
async def get_scrapy_metrics():
    """Get Scrapy performance metrics"""
    if not redis_client:
        raise HTTPException(status_code=503, detail="Redis connection unavailable")
    
    try:
        metrics = {
            "total_alerts": 0,
            "alerts_by_type": {},
            "data_points_by_spider": {},
            "system_health": "unknown"
        }
        
        # Count recent alerts
        alert_keys = redis_client.lrange("alerts:scrapy:recent", 0, -1)
        for key in alert_keys:
            alert_data = redis_client.get(key)
            if alert_data:
                alerts = json.loads(alert_data)
                metrics["total_alerts"] += len(alerts)
                
                for alert in alerts:
                    alert_type = alert.get("type", "unknown")
                    metrics["alerts_by_type"][alert_type] = metrics["alerts_by_type"].get(alert_type, 0) + 1
        
        # Count data points by spider
        spiders = ["discord_monitor", "project_auditor", "news_aggregator", "dex_monitor"]
        active_spiders = 0
        
        for spider in spiders:
            keys = redis_client.keys(f"scrapy:{spider}:*")
            total_data_points = 0
            
            if keys:
                active_spiders += 1
                for key in keys:
                    try:
                        data_str = redis_client.get(key)
                        if data_str:
                            data = json.loads(data_str)
                            total_data_points += len(data) if isinstance(data, list) else 1
                    except:
                        continue
            
            metrics["data_points_by_spider"][spider] = total_data_points
        
        # Determine system health
        if active_spiders >= 3:
            metrics["system_health"] = "healthy"
        elif active_spiders >= 1:
            metrics["system_health"] = "degraded"
        else:
            metrics["system_health"] = "critical"
        
        metrics["active_spiders"] = active_spiders
        metrics["total_spiders"] = len(spiders)
        
        return metrics
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8002)
