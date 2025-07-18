#!/usr/bin/env python3
"""
Cerebro Agent Tools
Tools for the TradingAnalystAgent to interact with HFT Ninja and external systems
"""

import asyncio
import httpx
import json
import logging
from typing import Dict, List, Any, Optional
from datetime import datetime, timedelta
import time

from ..memory.rag_search import CerebroRAGSearch
from ..memory.schema import SearchQuery, ContextType, ContextSource

logger = logging.getLogger(__name__)


class AgentTools:
    """Collection of tools for the Cerebro agent"""

    def __init__(
        self,
        hft_ninja_url: str = "http://localhost:8080",
        prometheus_url: str = "http://localhost:9090",
        rag_search: Optional[CerebroRAGSearch] = None
    ):
        self.hft_ninja_url = hft_ninja_url.rstrip('/')
        self.prometheus_url = prometheus_url.rstrip('/')
        self.http_client = httpx.AsyncClient(timeout=30.0)
        self.rag_search = rag_search

    async def close(self):
        """Close HTTP client"""
        await self.http_client.aclose()

    async def get_hft_stats(self, time_range: str = "24h") -> Dict[str, Any]:
        """Get HFT Ninja statistics"""
        try:
            response = await self.http_client.get(f"{self.hft_ninja_url}/api/stats?range={time_range}")

            if response.status_code == 200:
                stats = response.json()
                logger.info(f"✅ Retrieved HFT stats for {time_range}")
                return stats
            else:
                # Fallback: get from metrics endpoint
                metrics_response = await self.http_client.get(f"{self.hft_ninja_url}/metrics")
                if metrics_response.status_code == 200:
                    metrics = self._parse_prometheus_metrics(metrics_response.text)
                    return self._convert_metrics_to_stats(metrics, time_range)
                else:
                    return {"error": f"HTTP {response.status_code}", "time_range": time_range}

        except Exception as e:
            logger.error(f"Failed to get HFT stats: {e}")
            return {"error": str(e), "time_range": time_range}

    def _parse_prometheus_metrics(self, metrics_text: str) -> Dict[str, float]:
        """Parse Prometheus metrics format"""
        metrics = {}
        for line in metrics_text.split('\n'):
            line = line.strip()
            if line and not line.startswith('#'):
                try:
                    if ' ' in line:
                        metric_name, value = line.rsplit(' ', 1)
                        if '{' in metric_name:
                            metric_name = metric_name.split('{')[0]
                        try:
                            metrics[metric_name] = float(value)
                        except ValueError:
                            pass
                except Exception:
                    continue
        return metrics

    def _convert_metrics_to_stats(self, metrics: Dict[str, float], time_range: str) -> Dict[str, Any]:
        """Convert raw metrics to stats format"""
        return {
            "time_range": time_range,
            "total_trades": int(metrics.get("hft_trades_total", 0)),
            "successful_trades": int(metrics.get("hft_trades_successful_total", 0)),
            "failed_trades": int(metrics.get("hft_trades_failed_total", 0)),
            "total_profit_sol": metrics.get("hft_profit_sol_total", 0.0),
            "total_loss_sol": metrics.get("hft_loss_sol_total", 0.0),
            "average_latency_ms": metrics.get("hft_execution_latency_ms", 0.0),
            "mev_opportunities": int(metrics.get("hft_mev_opportunities_total", 0)),
            "active_strategies": int(metrics.get("hft_active_strategies", 0)),
            "timestamp": datetime.now().isoformat()
        }

    async def query_prometheus(self, query: str, time_range: str = "1h") -> Dict[str, Any]:
        """Query Prometheus for metrics"""
        try:
            params = {
                "query": query,
                "time": int(time.time())
            }

            response = await self.http_client.get(
                f"{self.prometheus_url}/api/v1/query",
                params=params
            )

            if response.status_code == 200:
                data = response.json()
                logger.info(f"✅ Prometheus query executed: {query}")
                return data
            else:
                return {"error": f"HTTP {response.status_code}", "query": query}

        except Exception as e:
            logger.error(f"Prometheus query failed: {e}")
            return {"error": str(e), "query": query}

    async def update_config(self, config_updates: Dict[str, Any]) -> Dict[str, Any]:
        """Update HFT Ninja configuration"""
        try:
            response = await self.http_client.post(
                f"{self.hft_ninja_url}/api/config/update",
                json=config_updates
            )

            if response.status_code == 200:
                result = response.json()
                logger.info(f"✅ Configuration updated: {list(config_updates.keys())}")
                return {"status": "success", "result": result}
            else:
                return {"status": "error", "error": f"HTTP {response.status_code}"}

        except Exception as e:
            logger.error(f"Config update failed: {e}")
            return {"status": "error", "error": str(e)}

    async def search_memory(self, query_text: str, context_types: Optional[List[str]] = None, max_results: int = 5) -> Dict[str, Any]:
        """Search memory for relevant contexts"""
        try:
            if not self.rag_search:
                return {"error": "RAG search not initialized", "results": []}

            # Convert string context types to enum
            context_type_enums = None
            if context_types:
                try:
                    context_type_enums = [ContextType(ct) for ct in context_types]
                except ValueError as e:
                    return {"error": f"Invalid context type: {e}", "results": []}

            # Create search query
            search_query = SearchQuery(
                query_text=query_text,
                context_types=context_type_enums,
                max_results=max_results,
                similarity_threshold=0.6  # Lower threshold for more results
            )

            # Perform search
            results = await self.rag_search.search(search_query)

            # Convert to simple format
            formatted_results = []
            for result in results:
                formatted_results.append({
                    "content": result.context_entry.content,
                    "type": result.context_entry.context_type.value,
                    "source": result.context_entry.source.value,
                    "similarity": result.similarity_score,
                    "timestamp": result.context_entry.timestamp
                })

            logger.info(f"✅ Memory search completed: {len(formatted_results)} results")
            return {"results": formatted_results, "query": query_text}

        except Exception as e:
            logger.error(f"Memory search failed: {e}")
            return {"error": str(e), "results": []}

    async def get_market_sentiment(self, token_symbol: str = "SOL") -> Dict[str, Any]:
        """Get market sentiment for token (mock implementation)"""
        try:
            # This would typically call external APIs like Twitter, Reddit, etc.
            # For now, return mock data

            mock_sentiment = {
                "token": token_symbol,
                "sentiment_score": 0.65,  # 0-1 scale (0=bearish, 1=bullish)
                "confidence": 0.8,
                "sources": ["twitter", "reddit", "news"],
                "key_topics": ["defi", "trading", "volume"],
                "trend": "bullish",
                "timestamp": datetime.now().isoformat(),
                "note": "Mock data - integrate with real sentiment APIs"
            }

            logger.info(f"✅ Market sentiment retrieved for {token_symbol}")
            return mock_sentiment

        except Exception as e:
            logger.error(f"Market sentiment failed: {e}")
            return {"error": str(e), "token": token_symbol}

    async def get_strategy_status(self, strategy_name: Optional[str] = None) -> Dict[str, Any]:
        """Get status of trading strategies"""
        try:
            url = f"{self.hft_ninja_url}/api/strategies"
            if strategy_name:
                url += f"/{strategy_name}"

            response = await self.http_client.get(url)

            if response.status_code == 200:
                data = response.json()
                logger.info(f"✅ Strategy status retrieved: {strategy_name or 'all'}")
                return data
            else:
                return {"error": f"HTTP {response.status_code}"}

        except Exception as e:
            logger.error(f"Strategy status failed: {e}")
            return {"error": str(e)}

    async def get_recent_logs(self, lines: int = 50, level: str = "info") -> Dict[str, Any]:
        """Get recent logs from HFT Ninja"""
        try:
            params = {"lines": lines, "level": level}
            response = await self.http_client.get(f"{self.hft_ninja_url}/api/logs", params=params)

            if response.status_code == 200:
                data = response.json()
                logger.info(f"✅ Retrieved {lines} log lines")
                return data
            else:
                return {"error": f"HTTP {response.status_code}"}

        except Exception as e:
            logger.error(f"Log retrieval failed: {e}")
            return {"error": str(e)}

    def get_available_tools(self) -> List[Dict[str, str]]:
        """Get list of available tools with descriptions"""
        return [
            {
                "name": "get_hft_stats",
                "description": "Get HFT Ninja trading statistics for specified time range",
                "parameters": "time_range (str): '1h', '24h', '7d', etc."
            },
            {
                "name": "query_prometheus",
                "description": "Query Prometheus metrics with PromQL",
                "parameters": "query (str): PromQL query, time_range (str): time range"
            },
            {
                "name": "update_config",
                "description": "Update HFT Ninja configuration",
                "parameters": "config_updates (dict): configuration changes"
            },
            {
                "name": "search_memory",
                "description": "Search Cerebro memory for relevant contexts",
                "parameters": "query_text (str): search query, context_types (list): filter by types"
            },
            {
                "name": "get_market_sentiment",
                "description": "Get market sentiment for specified token",
                "parameters": "token_symbol (str): token symbol (e.g., 'SOL')"
            },
            {
                "name": "get_strategy_status",
                "description": "Get status of trading strategies",
                "parameters": "strategy_name (str, optional): specific strategy name"
            },
            {
                "name": "get_recent_logs",
                "description": "Get recent logs from HFT Ninja",
                "parameters": "lines (int): number of lines, level (str): log level"
            }
        ]