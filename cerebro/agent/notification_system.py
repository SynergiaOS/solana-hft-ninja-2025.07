#!/usr/bin/env python3
"""
Real-Time Notification System for Cerebro
Supports Discord, Telegram, and WebSocket notifications
"""

import asyncio
import json
import aiohttp
from typing import Dict, Any, List, Optional
from datetime import datetime
import logging

try:
    import discord
    from discord.ext import commands
    DISCORD_AVAILABLE = True
except ImportError:
    DISCORD_AVAILABLE = False

try:
    from telegram import Bot
    from telegram.error import TelegramError
    TELEGRAM_AVAILABLE = True
except ImportError:
    TELEGRAM_AVAILABLE = False

logger = logging.getLogger(__name__)

class NotificationChannel:
    """Base class for notification channels"""
    
    async def send_approval_request(self, request) -> bool:
        """Send approval request notification"""
        raise NotImplementedError
    
    async def send_trading_alert(self, alert: Dict[str, Any]) -> bool:
        """Send trading alert notification"""
        raise NotImplementedError
    
    async def send_system_status(self, status: Dict[str, Any]) -> bool:
        """Send system status notification"""
        raise NotImplementedError

class DiscordNotificationChannel(NotificationChannel):
    """Discord notification channel"""
    
    def __init__(self, webhook_url: str, channel_id: Optional[str] = None):
        self.webhook_url = webhook_url
        self.channel_id = channel_id
        self.session = None
    
    async def _ensure_session(self):
        """Ensure aiohttp session exists"""
        if self.session is None:
            self.session = aiohttp.ClientSession()
    
    async def send_approval_request(self, request) -> bool:
        """Send approval request to Discord"""
        try:
            await self._ensure_session()
            
            decision = request.decision
            
            # Create rich embed
            embed = {
                "title": "🚨 Trading Decision Approval Required",
                "color": self._get_risk_color(decision.risk_level),
                "timestamp": datetime.now().isoformat(),
                "fields": [
                    {
                        "name": "Strategy",
                        "value": decision.strategy_type.upper(),
                        "inline": True
                    },
                    {
                        "name": "Action",
                        "value": f"{decision.action.upper()} {decision.token_symbol}",
                        "inline": True
                    },
                    {
                        "name": "Amount",
                        "value": f"{decision.amount_sol:.3f} SOL",
                        "inline": True
                    },
                    {
                        "name": "Confidence",
                        "value": f"{decision.confidence_score:.1%}",
                        "inline": True
                    },
                    {
                        "name": "Risk Level",
                        "value": decision.risk_level.value.upper(),
                        "inline": True
                    },
                    {
                        "name": "Est. Profit",
                        "value": f"{decision.estimated_profit:.3f} SOL" if decision.estimated_profit else "Unknown",
                        "inline": True
                    },
                    {
                        "name": "Reasoning",
                        "value": decision.reasoning[:1000],  # Limit length
                        "inline": False
                    },
                    {
                        "name": "Expires",
                        "value": f"<t:{int(datetime.fromisoformat(request.expires_at).timestamp())}:R>",
                        "inline": True
                    }
                ],
                "footer": {
                    "text": f"Request ID: {request.request_id}"
                }
            }
            
            # Add action buttons (if using Discord bot)
            components = [
                {
                    "type": 1,
                    "components": [
                        {
                            "type": 2,
                            "style": 3,  # Green
                            "label": "✅ Approve",
                            "custom_id": f"approve_{request.request_id}"
                        },
                        {
                            "type": 2,
                            "style": 4,  # Red
                            "label": "❌ Reject",
                            "custom_id": f"reject_{request.request_id}"
                        }
                    ]
                }
            ]
            
            payload = {
                "embeds": [embed],
                "components": components
            }
            
            async with self.session.post(self.webhook_url, json=payload) as response:
                if response.status == 204:
                    logger.info(f"Discord notification sent for request {request.request_id}")
                    return True
                else:
                    logger.error(f"Discord notification failed: {response.status}")
                    return False
                    
        except Exception as e:
            logger.error(f"Discord notification error: {e}")
            return False
    
    async def send_trading_alert(self, alert: Dict[str, Any]) -> bool:
        """Send trading alert to Discord"""
        try:
            await self._ensure_session()
            
            embed = {
                "title": f"📊 {alert.get('title', 'Trading Alert')}",
                "description": alert.get('message', ''),
                "color": self._get_alert_color(alert.get('type', 'info')),
                "timestamp": datetime.now().isoformat(),
                "fields": []
            }
            
            # Add fields from alert data
            for key, value in alert.get('data', {}).items():
                embed["fields"].append({
                    "name": key.replace('_', ' ').title(),
                    "value": str(value),
                    "inline": True
                })
            
            payload = {"embeds": [embed]}
            
            async with self.session.post(self.webhook_url, json=payload) as response:
                return response.status == 204
                
        except Exception as e:
            logger.error(f"Discord alert error: {e}")
            return False
    
    async def send_system_status(self, status: Dict[str, Any]) -> bool:
        """Send system status to Discord"""
        try:
            await self._ensure_session()
            
            embed = {
                "title": "🤖 Cerebro System Status",
                "color": 0x00ff00 if status.get('healthy', True) else 0xff0000,
                "timestamp": datetime.now().isoformat(),
                "fields": [
                    {
                        "name": "Status",
                        "value": "🟢 Healthy" if status.get('healthy', True) else "🔴 Issues Detected",
                        "inline": True
                    },
                    {
                        "name": "Uptime",
                        "value": status.get('uptime', 'Unknown'),
                        "inline": True
                    },
                    {
                        "name": "Active Strategies",
                        "value": str(status.get('active_strategies', 0)),
                        "inline": True
                    }
                ]
            }
            
            payload = {"embeds": [embed]}
            
            async with self.session.post(self.webhook_url, json=payload) as response:
                return response.status == 204
                
        except Exception as e:
            logger.error(f"Discord status error: {e}")
            return False
    
    def _get_risk_color(self, risk_level) -> int:
        """Get color for risk level"""
        colors = {
            "low": 0x00ff00,      # Green
            "medium": 0xffff00,   # Yellow
            "high": 0xff8800,     # Orange
            "critical": 0xff0000  # Red
        }
        return colors.get(risk_level.value if hasattr(risk_level, 'value') else risk_level, 0x808080)
    
    def _get_alert_color(self, alert_type: str) -> int:
        """Get color for alert type"""
        colors = {
            "success": 0x00ff00,
            "warning": 0xffff00,
            "error": 0xff0000,
            "info": 0x0099ff
        }
        return colors.get(alert_type, 0x808080)
    
    async def close(self):
        """Close the session"""
        if self.session:
            await self.session.close()

class TelegramNotificationChannel(NotificationChannel):
    """Telegram notification channel"""
    
    def __init__(self, bot_token: str, chat_id: str):
        self.bot_token = bot_token
        self.chat_id = chat_id
        self.bot = None
        
        if TELEGRAM_AVAILABLE:
            self.bot = Bot(token=bot_token)
    
    async def send_approval_request(self, request) -> bool:
        """Send approval request to Telegram"""
        if not self.bot:
            logger.warning("Telegram bot not available")
            return False
        
        try:
            decision = request.decision
            
            message = f"""
🚨 **Trading Decision Approval Required**

**Strategy:** {decision.strategy_type.upper()}
**Action:** {decision.action.upper()} {decision.token_symbol}
**Amount:** {decision.amount_sol:.3f} SOL
**Confidence:** {decision.confidence_score:.1%}
**Risk Level:** {decision.risk_level.value.upper()}
**Est. Profit:** {decision.estimated_profit:.3f} SOL if decision.estimated_profit else "Unknown"

**Reasoning:** {decision.reasoning[:500]}

**Request ID:** `{request.request_id}`
**Expires:** {request.expires_at}

Reply with:
• `/approve {request.request_id}` to approve
• `/reject {request.request_id} [reason]` to reject
            """
            
            await self.bot.send_message(
                chat_id=self.chat_id,
                text=message,
                parse_mode='Markdown'
            )
            
            logger.info(f"Telegram notification sent for request {request.request_id}")
            return True
            
        except Exception as e:
            logger.error(f"Telegram notification error: {e}")
            return False
    
    async def send_trading_alert(self, alert: Dict[str, Any]) -> bool:
        """Send trading alert to Telegram"""
        if not self.bot:
            return False
        
        try:
            message = f"📊 **{alert.get('title', 'Trading Alert')}**\n\n{alert.get('message', '')}"
            
            await self.bot.send_message(
                chat_id=self.chat_id,
                text=message,
                parse_mode='Markdown'
            )
            
            return True
            
        except Exception as e:
            logger.error(f"Telegram alert error: {e}")
            return False
    
    async def send_system_status(self, status: Dict[str, Any]) -> bool:
        """Send system status to Telegram"""
        if not self.bot:
            return False
        
        try:
            status_emoji = "🟢" if status.get('healthy', True) else "🔴"
            message = f"""
🤖 **Cerebro System Status**

**Status:** {status_emoji} {"Healthy" if status.get('healthy', True) else "Issues Detected"}
**Uptime:** {status.get('uptime', 'Unknown')}
**Active Strategies:** {status.get('active_strategies', 0)}
            """
            
            await self.bot.send_message(
                chat_id=self.chat_id,
                text=message,
                parse_mode='Markdown'
            )
            
            return True
            
        except Exception as e:
            logger.error(f"Telegram status error: {e}")
            return False

class WebSocketNotificationChannel(NotificationChannel):
    """WebSocket notification channel for real-time dashboard updates"""
    
    def __init__(self, websocket_manager):
        self.websocket_manager = websocket_manager
    
    async def send_approval_request(self, request) -> bool:
        """Send approval request via WebSocket"""
        try:
            message = {
                "type": "approval_request",
                "data": {
                    "request_id": request.request_id,
                    "decision": {
                        "decision_id": request.decision.decision_id,
                        "strategy_type": request.decision.strategy_type,
                        "action": request.decision.action,
                        "token_symbol": request.decision.token_symbol,
                        "amount_sol": request.decision.amount_sol,
                        "confidence_score": request.decision.confidence_score,
                        "risk_level": request.decision.risk_level.value,
                        "reasoning": request.decision.reasoning,
                        "estimated_profit": request.decision.estimated_profit,
                        "max_loss": request.decision.max_loss
                    },
                    "expires_at": request.expires_at,
                    "created_at": request.created_at
                }
            }
            
            await self.websocket_manager.broadcast(json.dumps(message))
            return True
            
        except Exception as e:
            logger.error(f"WebSocket notification error: {e}")
            return False
    
    async def send_trading_alert(self, alert: Dict[str, Any]) -> bool:
        """Send trading alert via WebSocket"""
        try:
            message = {
                "type": "trading_alert",
                "data": alert,
                "timestamp": datetime.now().isoformat()
            }
            
            await self.websocket_manager.broadcast(json.dumps(message))
            return True
            
        except Exception as e:
            logger.error(f"WebSocket alert error: {e}")
            return False
    
    async def send_system_status(self, status: Dict[str, Any]) -> bool:
        """Send system status via WebSocket"""
        try:
            message = {
                "type": "system_status",
                "data": status,
                "timestamp": datetime.now().isoformat()
            }
            
            await self.websocket_manager.broadcast(json.dumps(message))
            return True
            
        except Exception as e:
            logger.error(f"WebSocket status error: {e}")
            return False

class NotificationManager:
    """Manages multiple notification channels"""
    
    def __init__(self):
        self.channels: List[NotificationChannel] = []
    
    def add_channel(self, channel: NotificationChannel):
        """Add a notification channel"""
        self.channels.append(channel)
    
    async def send_approval_request(self, request) -> Dict[str, bool]:
        """Send approval request to all channels"""
        results = {}
        
        for i, channel in enumerate(self.channels):
            try:
                result = await channel.send_approval_request(request)
                results[f"channel_{i}"] = result
            except Exception as e:
                logger.error(f"Channel {i} failed: {e}")
                results[f"channel_{i}"] = False
        
        return results
    
    async def send_trading_alert(self, alert: Dict[str, Any]) -> Dict[str, bool]:
        """Send trading alert to all channels"""
        results = {}
        
        for i, channel in enumerate(self.channels):
            try:
                result = await channel.send_trading_alert(alert)
                results[f"channel_{i}"] = result
            except Exception as e:
                logger.error(f"Channel {i} failed: {e}")
                results[f"channel_{i}"] = False
        
        return results
    
    async def send_system_status(self, status: Dict[str, Any]) -> Dict[str, bool]:
        """Send system status to all channels"""
        results = {}
        
        for i, channel in enumerate(self.channels):
            try:
                result = await channel.send_system_status(status)
                results[f"channel_{i}"] = result
            except Exception as e:
                logger.error(f"Channel {i} failed: {e}")
                results[f"channel_{i}"] = False
        
        return results
    
    async def close_all(self):
        """Close all channels"""
        for channel in self.channels:
            if hasattr(channel, 'close'):
                await channel.close()
