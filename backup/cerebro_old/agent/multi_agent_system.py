#!/usr/bin/env python3
"""
Multi-Agent Collaboration System for Cerebro
Inspired by TensorZero's multi-agent capabilities
"""

import asyncio
import json
import time
from typing import Dict, Any, List, Optional, Tuple
from datetime import datetime
from enum import Enum
from dataclasses import dataclass, asdict
from abc import ABC, abstractmethod
import logging

logger = logging.getLogger(__name__)

class AgentRole(Enum):
    """Roles for specialized agents"""
    SENTIMENT_ANALYZER = "sentiment_analyzer"
    TECHNICAL_ANALYST = "technical_analyst"
    RISK_ASSESSOR = "risk_assessor"
    STRATEGY_COORDINATOR = "strategy_coordinator"
    ACTION_EXECUTOR = "action_executor"

class MessageType(Enum):
    """Types of inter-agent messages"""
    ANALYSIS_REQUEST = "analysis_request"
    ANALYSIS_RESULT = "analysis_result"
    CONSENSUS_REQUEST = "consensus_request"
    CONSENSUS_RESPONSE = "consensus_response"
    ACTION_PROPOSAL = "action_proposal"
    ACTION_APPROVAL = "action_approval"
    STATUS_UPDATE = "status_update"

@dataclass
class AgentMessage:
    """Message between agents"""
    message_id: str
    sender: AgentRole
    recipient: Optional[AgentRole]  # None for broadcast
    message_type: MessageType
    content: Dict[str, Any]
    timestamp: str
    correlation_id: Optional[str] = None  # For tracking related messages

@dataclass
class AgentAnalysis:
    """Analysis result from an agent"""
    agent_role: AgentRole
    analysis_type: str
    confidence: float  # 0.0 to 1.0
    recommendation: str
    data: Dict[str, Any]
    reasoning: str
    timestamp: str

class BaseAgent(ABC):
    """Base class for all specialized agents"""
    
    def __init__(self, role: AgentRole, config: Dict[str, Any]):
        self.role = role
        self.config = config
        self.message_queue: asyncio.Queue = asyncio.Queue()
        self.running = False
        self.analysis_history: List[AgentAnalysis] = []
    
    @abstractmethod
    async def analyze(self, data: Dict[str, Any]) -> AgentAnalysis:
        """Perform analysis on given data"""
        pass
    
    async def start(self):
        """Start the agent"""
        self.running = True
        logger.info(f"{self.role.value} agent started")
    
    async def stop(self):
        """Stop the agent"""
        self.running = False
        logger.info(f"{self.role.value} agent stopped")
    
    async def send_message(self, message: AgentMessage, message_bus):
        """Send message via message bus"""
        await message_bus.send_message(message)
    
    async def receive_message(self, message: AgentMessage):
        """Receive message from message bus"""
        await self.message_queue.put(message)
    
    async def process_messages(self, message_bus):
        """Process incoming messages"""
        while self.running:
            try:
                message = await asyncio.wait_for(self.message_queue.get(), timeout=1.0)
                await self._handle_message(message, message_bus)
            except asyncio.TimeoutError:
                continue
            except Exception as e:
                logger.error(f"{self.role.value} message processing error: {e}")
    
    async def _handle_message(self, message: AgentMessage, message_bus):
        """Handle incoming message"""
        if message.message_type == MessageType.ANALYSIS_REQUEST:
            analysis = await self.analyze(message.content)
            
            response = AgentMessage(
                message_id=f"{self.role.value}_{int(time.time() * 1000)}",
                sender=self.role,
                recipient=message.sender,
                message_type=MessageType.ANALYSIS_RESULT,
                content=asdict(analysis),
                timestamp=datetime.now().isoformat(),
                correlation_id=message.correlation_id
            )
            
            await self.send_message(response, message_bus)

class SentimentAnalyzerAgent(BaseAgent):
    """Agent specialized in market sentiment analysis"""
    
    def __init__(self, config: Dict[str, Any]):
        super().__init__(AgentRole.SENTIMENT_ANALYZER, config)
        self.fingpt_manager = None  # Will be injected
    
    async def analyze(self, data: Dict[str, Any]) -> AgentAnalysis:
        """Analyze market sentiment"""
        try:
            # Extract relevant data
            token_symbol = data.get("token_symbol", "")
            market_data = data.get("market_data", {})
            news_data = data.get("news_data", [])
            social_data = data.get("social_data", [])
            
            # Perform sentiment analysis
            sentiment_score = await self._calculate_sentiment(token_symbol, news_data, social_data)
            market_sentiment = await self._analyze_market_sentiment(market_data)
            
            # Generate recommendation
            overall_sentiment = (sentiment_score + market_sentiment) / 2
            
            if overall_sentiment > 0.7:
                recommendation = "BULLISH"
            elif overall_sentiment > 0.3:
                recommendation = "NEUTRAL"
            else:
                recommendation = "BEARISH"
            
            return AgentAnalysis(
                agent_role=self.role,
                analysis_type="sentiment_analysis",
                confidence=min(0.9, abs(overall_sentiment - 0.5) * 2),  # Higher confidence for extreme sentiments
                recommendation=recommendation,
                data={
                    "sentiment_score": sentiment_score,
                    "market_sentiment": market_sentiment,
                    "overall_sentiment": overall_sentiment,
                    "news_count": len(news_data),
                    "social_mentions": len(social_data)
                },
                reasoning=f"Sentiment analysis based on {len(news_data)} news items and {len(social_data)} social mentions. Overall sentiment: {overall_sentiment:.2f}",
                timestamp=datetime.now().isoformat()
            )
            
        except Exception as e:
            logger.error(f"Sentiment analysis error: {e}")
            return AgentAnalysis(
                agent_role=self.role,
                analysis_type="sentiment_analysis",
                confidence=0.0,
                recommendation="NEUTRAL",
                data={"error": str(e)},
                reasoning=f"Analysis failed: {e}",
                timestamp=datetime.now().isoformat()
            )
    
    async def _calculate_sentiment(self, token_symbol: str, news_data: List, social_data: List) -> float:
        """Calculate sentiment from news and social data"""
        # Placeholder implementation - would use FinGPT or other sentiment models
        positive_keywords = ["bullish", "moon", "pump", "buy", "long", "up", "gain", "profit"]
        negative_keywords = ["bearish", "dump", "sell", "short", "down", "loss", "crash", "bear"]
        
        total_score = 0.5  # Neutral baseline
        total_weight = 0
        
        # Analyze news sentiment
        for news_item in news_data:
            text = news_item.get("content", "").lower()
            score = 0.5
            
            positive_count = sum(1 for keyword in positive_keywords if keyword in text)
            negative_count = sum(1 for keyword in negative_keywords if keyword in text)
            
            if positive_count > negative_count:
                score = 0.5 + (positive_count - negative_count) * 0.1
            elif negative_count > positive_count:
                score = 0.5 - (negative_count - positive_count) * 0.1
            
            weight = news_item.get("importance", 1.0)
            total_score += score * weight
            total_weight += weight
        
        # Analyze social sentiment
        for social_item in social_data:
            text = social_item.get("content", "").lower()
            score = 0.5
            
            positive_count = sum(1 for keyword in positive_keywords if keyword in text)
            negative_count = sum(1 for keyword in negative_keywords if keyword in text)
            
            if positive_count > negative_count:
                score = 0.5 + (positive_count - negative_count) * 0.05
            elif negative_count > positive_count:
                score = 0.5 - (negative_count - positive_count) * 0.05
            
            weight = social_item.get("engagement", 1.0)
            total_score += score * weight
            total_weight += weight
        
        return min(1.0, max(0.0, total_score / max(1, total_weight)))
    
    async def _analyze_market_sentiment(self, market_data: Dict[str, Any]) -> float:
        """Analyze market sentiment from price/volume data"""
        # Placeholder implementation
        price_change = market_data.get("price_change_24h", 0)
        volume_change = market_data.get("volume_change_24h", 0)
        
        # Simple sentiment based on price and volume changes
        sentiment = 0.5
        
        if price_change > 0.05:  # 5% price increase
            sentiment += 0.2
        elif price_change < -0.05:  # 5% price decrease
            sentiment -= 0.2
        
        if volume_change > 0.2:  # 20% volume increase
            sentiment += 0.1
        
        return min(1.0, max(0.0, sentiment))

class TechnicalAnalysisAgent(BaseAgent):
    """Agent specialized in technical analysis"""
    
    def __init__(self, config: Dict[str, Any]):
        super().__init__(AgentRole.TECHNICAL_ANALYST, config)
    
    async def analyze(self, data: Dict[str, Any]) -> AgentAnalysis:
        """Perform technical analysis"""
        try:
            price_data = data.get("price_data", [])
            volume_data = data.get("volume_data", [])
            
            # Calculate technical indicators
            indicators = await self._calculate_indicators(price_data, volume_data)
            
            # Generate signals
            signals = await self._generate_signals(indicators)
            
            # Determine overall recommendation
            bullish_signals = sum(1 for signal in signals if signal["direction"] == "bullish")
            bearish_signals = sum(1 for signal in signals if signal["direction"] == "bearish")
            
            if bullish_signals > bearish_signals:
                recommendation = "BUY"
                confidence = bullish_signals / len(signals)
            elif bearish_signals > bullish_signals:
                recommendation = "SELL"
                confidence = bearish_signals / len(signals)
            else:
                recommendation = "HOLD"
                confidence = 0.5
            
            return AgentAnalysis(
                agent_role=self.role,
                analysis_type="technical_analysis",
                confidence=confidence,
                recommendation=recommendation,
                data={
                    "indicators": indicators,
                    "signals": signals,
                    "bullish_signals": bullish_signals,
                    "bearish_signals": bearish_signals
                },
                reasoning=f"Technical analysis based on {len(signals)} indicators. {bullish_signals} bullish, {bearish_signals} bearish signals.",
                timestamp=datetime.now().isoformat()
            )
            
        except Exception as e:
            logger.error(f"Technical analysis error: {e}")
            return AgentAnalysis(
                agent_role=self.role,
                analysis_type="technical_analysis",
                confidence=0.0,
                recommendation="HOLD",
                data={"error": str(e)},
                reasoning=f"Analysis failed: {e}",
                timestamp=datetime.now().isoformat()
            )
    
    async def _calculate_indicators(self, price_data: List, volume_data: List) -> Dict[str, Any]:
        """Calculate technical indicators"""
        if not price_data:
            return {}
        
        prices = [float(p) for p in price_data[-50:]]  # Last 50 prices
        
        # Simple Moving Averages
        sma_20 = sum(prices[-20:]) / 20 if len(prices) >= 20 else prices[-1]
        sma_50 = sum(prices) / len(prices)
        
        # RSI (simplified)
        rsi = await self._calculate_rsi(prices)
        
        # MACD (simplified)
        macd = await self._calculate_macd(prices)
        
        return {
            "sma_20": sma_20,
            "sma_50": sma_50,
            "current_price": prices[-1],
            "rsi": rsi,
            "macd": macd
        }
    
    async def _calculate_rsi(self, prices: List[float]) -> float:
        """Calculate RSI indicator"""
        if len(prices) < 14:
            return 50.0  # Neutral
        
        gains = []
        losses = []
        
        for i in range(1, len(prices)):
            change = prices[i] - prices[i-1]
            if change > 0:
                gains.append(change)
                losses.append(0)
            else:
                gains.append(0)
                losses.append(abs(change))
        
        avg_gain = sum(gains[-14:]) / 14
        avg_loss = sum(losses[-14:]) / 14
        
        if avg_loss == 0:
            return 100.0
        
        rs = avg_gain / avg_loss
        rsi = 100 - (100 / (1 + rs))
        
        return rsi
    
    async def _calculate_macd(self, prices: List[float]) -> Dict[str, float]:
        """Calculate MACD indicator"""
        if len(prices) < 26:
            return {"macd": 0.0, "signal": 0.0, "histogram": 0.0}
        
        # Simplified MACD calculation
        ema_12 = prices[-1]  # Placeholder
        ema_26 = sum(prices[-26:]) / 26
        
        macd_line = ema_12 - ema_26
        signal_line = macd_line * 0.9  # Simplified signal
        histogram = macd_line - signal_line
        
        return {
            "macd": macd_line,
            "signal": signal_line,
            "histogram": histogram
        }
    
    async def _generate_signals(self, indicators: Dict[str, Any]) -> List[Dict[str, Any]]:
        """Generate trading signals from indicators"""
        signals = []
        
        # SMA signals
        if indicators.get("current_price", 0) > indicators.get("sma_20", 0):
            signals.append({"indicator": "SMA", "direction": "bullish", "strength": 0.6})
        else:
            signals.append({"indicator": "SMA", "direction": "bearish", "strength": 0.6})
        
        # RSI signals
        rsi = indicators.get("rsi", 50)
        if rsi < 30:
            signals.append({"indicator": "RSI", "direction": "bullish", "strength": 0.8})
        elif rsi > 70:
            signals.append({"indicator": "RSI", "direction": "bearish", "strength": 0.8})
        else:
            signals.append({"indicator": "RSI", "direction": "neutral", "strength": 0.3})
        
        # MACD signals
        macd_data = indicators.get("macd", {})
        if isinstance(macd_data, dict):
            if macd_data.get("histogram", 0) > 0:
                signals.append({"indicator": "MACD", "direction": "bullish", "strength": 0.7})
            else:
                signals.append({"indicator": "MACD", "direction": "bearish", "strength": 0.7})
        
        return signals

class RiskAssessmentAgent(BaseAgent):
    """Agent specialized in risk assessment"""
    
    def __init__(self, config: Dict[str, Any]):
        super().__init__(AgentRole.RISK_ASSESSOR, config)
    
    async def analyze(self, data: Dict[str, Any]) -> AgentAnalysis:
        """Perform risk assessment"""
        try:
            position_data = data.get("position_data", {})
            market_data = data.get("market_data", {})
            portfolio_data = data.get("portfolio_data", {})
            
            # Calculate various risk metrics
            risk_metrics = await self._calculate_risk_metrics(position_data, market_data, portfolio_data)
            
            # Determine overall risk level
            risk_score = await self._calculate_risk_score(risk_metrics)
            
            if risk_score > 0.8:
                recommendation = "HIGH_RISK"
                confidence = 0.9
            elif risk_score > 0.6:
                recommendation = "MEDIUM_RISK"
                confidence = 0.8
            elif risk_score > 0.4:
                recommendation = "LOW_RISK"
                confidence = 0.7
            else:
                recommendation = "MINIMAL_RISK"
                confidence = 0.8
            
            return AgentAnalysis(
                agent_role=self.role,
                analysis_type="risk_assessment",
                confidence=confidence,
                recommendation=recommendation,
                data=risk_metrics,
                reasoning=f"Risk assessment based on portfolio exposure, volatility, and market conditions. Risk score: {risk_score:.2f}",
                timestamp=datetime.now().isoformat()
            )
            
        except Exception as e:
            logger.error(f"Risk assessment error: {e}")
            return AgentAnalysis(
                agent_role=self.role,
                analysis_type="risk_assessment",
                confidence=0.0,
                recommendation="UNKNOWN_RISK",
                data={"error": str(e)},
                reasoning=f"Risk assessment failed: {e}",
                timestamp=datetime.now().isoformat()
            )
    
    async def _calculate_risk_metrics(self, position_data: Dict, market_data: Dict, portfolio_data: Dict) -> Dict[str, Any]:
        """Calculate various risk metrics"""
        metrics = {}
        
        # Position size risk
        position_size = position_data.get("amount_sol", 0)
        total_portfolio = portfolio_data.get("total_sol", 8.0)
        position_risk = position_size / total_portfolio
        metrics["position_risk"] = position_risk
        
        # Volatility risk
        volatility = market_data.get("volatility", 0.5)
        metrics["volatility_risk"] = volatility
        
        # Liquidity risk
        liquidity = market_data.get("liquidity_score", 0.5)
        metrics["liquidity_risk"] = 1.0 - liquidity
        
        # Concentration risk
        token_exposure = portfolio_data.get("token_concentration", {})
        max_concentration = max(token_exposure.values()) if token_exposure else 0
        metrics["concentration_risk"] = max_concentration
        
        # Market risk
        market_trend = market_data.get("trend_strength", 0)
        metrics["market_risk"] = abs(market_trend - 0.5) * 2  # Higher risk in extreme trends
        
        return metrics
    
    async def _calculate_risk_score(self, risk_metrics: Dict[str, Any]) -> float:
        """Calculate overall risk score"""
        weights = {
            "position_risk": 0.3,
            "volatility_risk": 0.25,
            "liquidity_risk": 0.2,
            "concentration_risk": 0.15,
            "market_risk": 0.1
        }
        
        total_score = 0
        total_weight = 0
        
        for metric, weight in weights.items():
            if metric in risk_metrics:
                total_score += risk_metrics[metric] * weight
                total_weight += weight
        
        return total_score / max(total_weight, 1)

class MessageBus:
    """Message bus for inter-agent communication"""
    
    def __init__(self):
        self.agents: Dict[AgentRole, BaseAgent] = {}
        self.message_history: List[AgentMessage] = []
    
    def register_agent(self, agent: BaseAgent):
        """Register an agent with the message bus"""
        self.agents[agent.role] = agent
    
    async def send_message(self, message: AgentMessage):
        """Send message to recipient(s)"""
        self.message_history.append(message)
        
        if message.recipient:
            # Send to specific agent
            if message.recipient in self.agents:
                await self.agents[message.recipient].receive_message(message)
        else:
            # Broadcast to all agents except sender
            for role, agent in self.agents.items():
                if role != message.sender:
                    await agent.receive_message(message)
    
    async def request_analysis(self, requester: AgentRole, data: Dict[str, Any], target_agents: List[AgentRole] = None) -> List[AgentAnalysis]:
        """Request analysis from multiple agents"""
        correlation_id = f"analysis_{int(time.time() * 1000)}"
        
        if target_agents is None:
            target_agents = [role for role in self.agents.keys() if role != requester]
        
        # Send analysis requests
        for target in target_agents:
            message = AgentMessage(
                message_id=f"{requester.value}_{int(time.time() * 1000)}",
                sender=requester,
                recipient=target,
                message_type=MessageType.ANALYSIS_REQUEST,
                content=data,
                timestamp=datetime.now().isoformat(),
                correlation_id=correlation_id
            )
            await self.send_message(message)
        
        # Wait for responses (with timeout)
        responses = []
        timeout = 30  # 30 seconds
        start_time = time.time()
        
        while len(responses) < len(target_agents) and (time.time() - start_time) < timeout:
            # Check for responses in message history
            for message in reversed(self.message_history):
                if (message.correlation_id == correlation_id and 
                    message.message_type == MessageType.ANALYSIS_RESULT and
                    message.sender in target_agents):
                    
                    # Convert message content back to AgentAnalysis
                    analysis_data = message.content
                    analysis = AgentAnalysis(**analysis_data)
                    
                    if analysis not in responses:
                        responses.append(analysis)
            
            await asyncio.sleep(0.1)
        
        return responses

class MultiAgentCoordinator:
    """Coordinates multiple agents for collaborative analysis"""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.message_bus = MessageBus()
        self.agents: Dict[AgentRole, BaseAgent] = {}
        
        # Initialize agents
        self._initialize_agents()
    
    def _initialize_agents(self):
        """Initialize all specialized agents"""
        self.agents[AgentRole.SENTIMENT_ANALYZER] = SentimentAnalyzerAgent(self.config)
        self.agents[AgentRole.TECHNICAL_ANALYST] = TechnicalAnalysisAgent(self.config)
        self.agents[AgentRole.RISK_ASSESSOR] = RiskAssessmentAgent(self.config)
        
        # Register agents with message bus
        for agent in self.agents.values():
            self.message_bus.register_agent(agent)
    
    async def start_all_agents(self):
        """Start all agents"""
        for agent in self.agents.values():
            await agent.start()
        
        # Start message processing for all agents
        tasks = []
        for agent in self.agents.values():
            task = asyncio.create_task(agent.process_messages(self.message_bus))
            tasks.append(task)
        
        return tasks
    
    async def stop_all_agents(self):
        """Stop all agents"""
        for agent in self.agents.values():
            await agent.stop()
    
    async def collaborative_analysis(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Perform collaborative analysis using all agents"""
        try:
            # Request analysis from all agents
            analyses = await self.message_bus.request_analysis(
                requester=AgentRole.STRATEGY_COORDINATOR,
                data=data,
                target_agents=[AgentRole.SENTIMENT_ANALYZER, AgentRole.TECHNICAL_ANALYST, AgentRole.RISK_ASSESSOR]
            )
            
            # Synthesize results
            synthesis = await self._synthesize_analyses(analyses)
            
            return {
                "individual_analyses": [asdict(analysis) for analysis in analyses],
                "synthesis": synthesis,
                "timestamp": datetime.now().isoformat()
            }
            
        except Exception as e:
            logger.error(f"Collaborative analysis error: {e}")
            return {
                "error": str(e),
                "timestamp": datetime.now().isoformat()
            }
    
    async def _synthesize_analyses(self, analyses: List[AgentAnalysis]) -> Dict[str, Any]:
        """Synthesize multiple agent analyses into unified recommendation"""
        if not analyses:
            return {"recommendation": "HOLD", "confidence": 0.0, "reasoning": "No analyses available"}
        
        # Weight analyses by confidence
        total_weight = sum(analysis.confidence for analysis in analyses)
        
        if total_weight == 0:
            return {"recommendation": "HOLD", "confidence": 0.0, "reasoning": "All analyses have zero confidence"}
        
        # Calculate weighted recommendation
        buy_weight = 0
        sell_weight = 0
        hold_weight = 0
        
        for analysis in analyses:
            weight = analysis.confidence
            
            if analysis.recommendation in ["BUY", "BULLISH"]:
                buy_weight += weight
            elif analysis.recommendation in ["SELL", "BEARISH"]:
                sell_weight += weight
            else:
                hold_weight += weight
        
        # Determine final recommendation
        if buy_weight > sell_weight and buy_weight > hold_weight:
            recommendation = "BUY"
            confidence = buy_weight / total_weight
        elif sell_weight > buy_weight and sell_weight > hold_weight:
            recommendation = "SELL"
            confidence = sell_weight / total_weight
        else:
            recommendation = "HOLD"
            confidence = hold_weight / total_weight
        
        # Generate reasoning
        reasoning_parts = []
        for analysis in analyses:
            reasoning_parts.append(f"{analysis.agent_role.value}: {analysis.recommendation} ({analysis.confidence:.2f})")
        
        reasoning = f"Collaborative analysis: {'; '.join(reasoning_parts)}"
        
        return {
            "recommendation": recommendation,
            "confidence": confidence,
            "reasoning": reasoning,
            "agent_count": len(analyses),
            "weights": {
                "buy": buy_weight,
                "sell": sell_weight,
                "hold": hold_weight
            }
        }
