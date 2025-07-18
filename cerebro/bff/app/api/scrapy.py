from fastapi import APIRouter, HTTPException, BackgroundTasks
from pydantic import BaseModel
from typing import List, Dict, Any
import redis
import json
from datetime import datetime

router = APIRouter(prefix="/scrapy", tags=["scrapy"])

# Redis connection
redis_client = redis.Redis(host="dragonflydb", port=6379, db=2, decode_responses=True)

class ScrapyAlert(BaseModel):
    type: str
    message: str = None
    source: str = None
    timestamp: str = None
    project: str = None
    risk_score: float = None
    issues: List[str] = None

class ScrapyAlertsRequest(BaseModel):
    alerts: List[ScrapyAlert]

@router.post("/alerts")
async def receive_scrapy_alerts(request: ScrapyAlertsRequest):
    """Receive alerts from Scrapy spiders"""
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
        
        return {
            "status": "success",
            "alerts_received": len(request.alerts),
            "timestamp": timestamp
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/alerts/recent")
async def get_recent_alerts():
    """Get recent Scrapy alerts"""
    try:
        alert_keys = redis_client.lrange("alerts:scrapy:recent", 0, 19)  # Last 20
        alerts = []
        
        for key in alert_keys:
            alert_data = redis_client.get(key)
            if alert_data:
                alerts.extend(json.loads(alert_data))
        
        return {
            "alerts": alerts,
            "count": len(alerts)
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/data/{spider_name}")
async def get_spider_data(spider_name: str, limit: int = 100):
    """Get latest data from specific spider"""
    try:
        pattern = f"scrapy:{spider_name}:*"
        keys = redis_client.keys(pattern)
        
        if not keys:
            return {"data": [], "count": 0}
        
        # Get latest key
        latest_key = sorted(keys)[-1]
        data = json.loads(redis_client.get(latest_key))
        
        return {
            "data": data[:limit],
            "count": len(data),
            "timestamp": latest_key.split(":")[-1]
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/status")
async def get_scrapy_status():
    """Get Scrapy system status"""
    try:
        spiders = ["discord_monitor", "project_auditor", "news_aggregator", "dex_monitor"]
        status = {}
        
        for spider in spiders:
            keys = redis_client.keys(f"scrapy:{spider}:*")
            if keys:
                latest_key = sorted(keys)[-1]
                timestamp = latest_key.split(":")[-1]
                data_count = len(json.loads(redis_client.get(latest_key)))
                
                status[spider] = {
                    "last_run": timestamp,
                    "data_count": data_count,
                    "status": "active"
                }
            else:
                status[spider] = {
                    "last_run": None,
                    "data_count": 0,
                    "status": "inactive"
                }
        
        return {"spiders": status}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/metrics")
async def get_scrapy_metrics():
    """Get Scrapy performance metrics"""
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

@router.post("/trigger/{spider_name}")
async def trigger_spider(spider_name: str, background_tasks: BackgroundTasks):
    """Trigger a specific spider run (placeholder for future implementation)"""
    valid_spiders = ["discord_monitor", "project_auditor", "news_aggregator", "dex_monitor"]

    if spider_name not in valid_spiders:
        raise HTTPException(status_code=400, detail=f"Invalid spider name. Valid options: {valid_spiders}")

    # This would integrate with Kestra or direct Scrapy execution
    # For now, return a placeholder response
    return {
        "status": "triggered",
        "spider": spider_name,
        "message": f"Spider {spider_name} trigger request received",
        "timestamp": datetime.now().isoformat(),
        "note": "Integration with Kestra/Scrapy execution pending"
    }