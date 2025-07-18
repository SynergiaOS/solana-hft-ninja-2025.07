#!/usr/bin/env python3
"""
HFT Ninja API Client for Project Cerebro
Provides integration with Solana HFT Ninja APIs
"""

import httpx
import json
import logging
from typing import Dict, List, Any, Optional
from datetime import datetime, timedelta

logger = logging.getLogger(__name__)


class HFTNinjaClient:
    """Client for HFT Ninja API integration"""

    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url.rstrip('/')
        self.client = httpx.AsyncClient(timeout=30.0)

    async def close(self):
        """Close HTTP client"""
        await self.client.aclose()

    async def get_health(self) -> Dict[str, Any]:
        """Get HFT Ninja health status"""
        try:
            response = await self.client.get(f"{self.base_url}/health")
            if response.status_code == 200:
                return response.json()
            else:
                return {"status": "unhealthy", "error": f"HTTP {response.status_code}"}
        except Exception as e:
            logger.error(f"Health check failed: {e}")
            return {"status": "unreachable", "error": str(e)}

    async def get_metrics(self) -> Dict[str, Any]:
        """Get Prometheus metrics from HFT Ninja"""
        try:
            response = await self.client.get(f"{self.base_url}/metrics")
            if response.status_code == 200:
                # Parse Prometheus metrics
                metrics_text = response.text
                return self._parse_prometheus_metrics(metrics_text)
            else:
                return {"error": f"HTTP {response.status_code}"}
        except Exception as e:
            logger.error(f"Metrics fetch failed: {e}")
            return {"error": str(e)}

    def _parse_prometheus_metrics(self, metrics_text: str) -> Dict[str, Any]:
        """Parse Prometheus metrics text format"""
        metrics = {}

        for line in metrics_text.split('\n'):
            line = line.strip()
            if line and not line.startswith('#'):
                try:
                    if ' ' in line:
                        metric_name, value = line.rsplit(' ', 1)
                        # Remove labels for simplicity
                        if '{' in metric_name:
                            metric_name = metric_name.split('{')[0]

                        try:
                            metrics[metric_name] = float(value)
                        except ValueError:
                            metrics[metric_name] = value
                except Exception:
                    continue

        return metrics

    async def get_trading_stats(self, hours: int = 24) -> Dict[str, Any]:
        """Get trading statistics for specified time period"""
        try:
            # Try to get from HFT Ninja API
            response = await self.client.get(f"{self.base_url}/api/stats?hours={hours}")
            if response.status_code == 200:
                return response.json()

            # Fallback: construct from metrics
            metrics = await self.get_metrics()

            return {
                "period_hours": hours,
                "total_trades": metrics.get("hft_trades_total", 0),
                "successful_trades": metrics.get("hft_trades_successful_total", 0),
                "failed_trades": metrics.get("hft_trades_failed_total", 0),
                "total_profit_sol": metrics.get("hft_profit_sol_total", 0.0),
                "total_loss_sol": metrics.get("hft_loss_sol_total", 0.0),
                "average_latency_ms": metrics.get("hft_execution_latency_ms", 0.0),
                "mev_opportunities": metrics.get("hft_mev_opportunities_total", 0),
                "active_strategies": metrics.get("hft_active_strategies", 0),
                "timestamp": datetime.now().isoformat()
            }

        except Exception as e:
            logger.error(f"Trading stats fetch failed: {e}")
            return {"error": str(e)}

    async def get_config(self) -> Dict[str, Any]:
        """Get current HFT Ninja configuration"""
        try:
            response = await self.client.get(f"{self.base_url}/api/config")
            if response.status_code == 200:
                return response.json()
            else:
                return {"error": f"HTTP {response.status_code}"}
        except Exception as e:
            logger.error(f"Config fetch failed: {e}")
            return {"error": str(e)}

    async def update_config(self, config_updates: Dict[str, Any]) -> Dict[str, Any]:
        """Update HFT Ninja configuration"""
        try:
            response = await self.client.post(
                f"{self.base_url}/api/config/update",
                json=config_updates
            )
            if response.status_code == 200:
                return {"status": "success", "result": response.json()}
            else:
                return {"status": "error", "error": f"HTTP {response.status_code}"}
        except Exception as e:
            logger.error(f"Config update failed: {e}")
            return {"status": "error", "error": str(e)}

    async def get_strategy_status(self, strategy_name: Optional[str] = None) -> Dict[str, Any]:
        """Get status of trading strategies"""
        try:
            url = f"{self.base_url}/api/strategies"
            if strategy_name:
                url += f"/{strategy_name}"

            response = await self.client.get(url)
            if response.status_code == 200:
                return response.json()
            else:
                return {"error": f"HTTP {response.status_code}"}
        except Exception as e:
            logger.error(f"Strategy status fetch failed: {e}")
            return {"error": str(e)}

    async def restart_strategy(self, strategy_name: str) -> Dict[str, Any]:
        """Restart a specific trading strategy"""
        try:
            response = await self.client.post(f"{self.base_url}/api/strategies/{strategy_name}/restart")
            if response.status_code == 200:
                return {"status": "success", "message": f"Strategy {strategy_name} restarted"}
            else:
                return {"status": "error", "error": f"HTTP {response.status_code}"}
        except Exception as e:
            logger.error(f"Strategy restart failed: {e}")
            return {"status": "error", "error": str(e)}

    async def get_logs(self, lines: int = 100, level: str = "info") -> Dict[str, Any]:
        """Get recent logs from HFT Ninja"""
        try:
            response = await self.client.get(f"{self.base_url}/api/logs?lines={lines}&level={level}")
            if response.status_code == 200:
                return response.json()
            else:
                return {"error": f"HTTP {response.status_code}"}
        except Exception as e:
            logger.error(f"Logs fetch failed: {e}")
            return {"error": str(e)}