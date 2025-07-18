#!/usr/bin/env python3
"""
TradingAnalystAgent - Main Cerebro AI Agent Class
Integrates all components: LLM Router, Tools, Memory, LangGraph Flow
"""

import asyncio
import json
import time
from typing import Dict, Any, List, Optional
from datetime import datetime
import logging

from langchain_core.tools import BaseTool
from langchain_core.language_models import BaseLanguageModel

from .langgraph_flow import CerebroLangGraphFlow
from .llm_router import LLMRouter
from .fingpt_integration import FinGPTManager, create_fingpt_manager
from .tools.fingpt_tool import (
    FinGPTSentimentTool,
    FinGPTForecastTool,
    FinGPTAnalysisTool,
    FinGPTMarketInsightTool
)
from .human_in_the_loop import (
    HumanInTheLoopManager,
    TradingDecision,
    RiskLevel,
    ApprovalStatus,
    assess_trading_risk,
    calculate_confidence_score
)
from .notification_system import (
    NotificationManager,
    DiscordNotificationChannel,
    TelegramNotificationChannel,
    WebSocketNotificationChannel
)
from .multi_agent_system import (
    MultiAgentCoordinator,
    AgentRole,
    AgentAnalysis
)
from ..memory.memory_manager import MemoryManager
from ..core.config import CerebroConfig

logger = logging.getLogger(__name__)

class TradingAnalystAgent:
    """
    Main Cerebro AI Agent that combines all components into a unified system
    
    This agent can:
    - Analyze trading performance
    - Provide strategic recommendations
    - Monitor market conditions
    - Learn from interactions
    - Execute complex multi-step analysis
    """
    
    def __init__(self, config: CerebroConfig):
        self.config = config
        self.memory_manager = None
        self.llm_router = None
        self.fingpt_manager = None
        self.tools = []
        self.langgraph_flow = None
        self.session_id = None
        self.conversation_history = []

        # TensorZero-inspired enhancements
        self.human_loop_manager = None
        self.notification_manager = None
        self.multi_agent_coordinator = None

        # Performance metrics
        self.metrics = {
            "total_queries": 0,
            "successful_responses": 0,
            "average_response_time": 0.0,
            "total_actions_executed": 0,
            "memory_entries_created": 0,
            "human_approvals_requested": 0,
            "auto_approvals": 0,
            "multi_agent_analyses": 0
        }

        logger.info("TradingAnalystAgent initialized with TensorZero enhancements")
    
    async def initialize(self):
        """Initialize all components"""
        try:
            logger.info("Initializing TradingAnalystAgent components...")
            
            # Initialize memory manager
            self.memory_manager = MemoryManager(self.config)
            await self.memory_manager.initialize()
            
            # Initialize LLM router
            self.llm_router = LLMRouter()

            # Initialize FinGPT manager
            self.fingpt_manager = await create_fingpt_manager(["sentiment_analysis"])

            # Initialize tools
            self.tools = await self._initialize_tools()

            # Initialize TensorZero-inspired enhancements
            await self._initialize_tensorZero_enhancements()

            # Initialize LangGraph flow
            primary_llm = await self.llm_router.get_primary_llm()
            self.langgraph_flow = CerebroLangGraphFlow(
                tools=self.tools,
                llm=primary_llm,
                memory_manager=self.memory_manager,
                max_iterations=self.config.agent.max_iterations
            )
            
            # Generate session ID
            self.session_id = f"session_{int(time.time())}"
            
            logger.info("✅ TradingAnalystAgent fully initialized")

        except Exception as e:
            logger.error(f"Failed to initialize TradingAnalystAgent: {e}")
            raise

    async def _initialize_tensorZero_enhancements(self):
        """Initialize TensorZero-inspired enhancements"""
        try:
            logger.info("Initializing TensorZero-inspired enhancements...")

            # Initialize Human-in-the-Loop Manager
            hitl_config = {
                "auto_approval_thresholds": {
                    "low": 0.85,
                    "medium": 0.95,
                    "high": 1.0,
                    "critical": 1.0
                },
                "approval_timeouts": {
                    "low": 300,      # 5 minutes
                    "medium": 600,   # 10 minutes
                    "high": 1800,    # 30 minutes
                    "critical": 3600 # 1 hour
                }
            }
            self.human_loop_manager = HumanInTheLoopManager(hitl_config)

            # Initialize Notification Manager
            self.notification_manager = NotificationManager()

            # Add notification channels if configured
            if hasattr(self.config, 'discord_webhook_url') and self.config.discord_webhook_url:
                discord_channel = DiscordNotificationChannel(self.config.discord_webhook_url)
                self.notification_manager.add_channel(discord_channel)

            if hasattr(self.config, 'telegram_bot_token') and self.config.telegram_bot_token:
                telegram_channel = TelegramNotificationChannel(
                    self.config.telegram_bot_token,
                    self.config.telegram_chat_id
                )
                self.notification_manager.add_channel(telegram_channel)

            # Register notification callback with human loop manager
            self.human_loop_manager.add_notification_callback(
                self.notification_manager.send_approval_request
            )

            # Initialize Multi-Agent Coordinator
            multi_agent_config = {
                "sentiment_analysis": True,
                "technical_analysis": True,
                "risk_assessment": True,
                "collaboration_timeout": 30
            }
            self.multi_agent_coordinator = MultiAgentCoordinator(multi_agent_config)

            # Start multi-agent system
            await self.multi_agent_coordinator.start_all_agents()

            logger.info("TensorZero enhancements initialized successfully")

        except Exception as e:
            logger.error(f"Failed to initialize TensorZero enhancements: {e}")
            # Continue without enhancements rather than failing completely
            self.human_loop_manager = None
            self.notification_manager = None
            self.multi_agent_coordinator = None
    
    async def _initialize_tools(self) -> List[BaseTool]:
        """Initialize all agent tools including FinGPT tools"""
        tools = []

        try:
            # FinGPT Sentiment Analysis Tool
            fingpt_sentiment_tool = FinGPTSentimentTool(self.fingpt_manager)
            tools.append(fingpt_sentiment_tool)

            # FinGPT Price Forecast Tool
            fingpt_forecast_tool = FinGPTForecastTool(self.fingpt_manager)
            tools.append(fingpt_forecast_tool)

            # FinGPT Financial Analysis Tool
            fingpt_analysis_tool = FinGPTAnalysisTool(self.fingpt_manager)
            tools.append(fingpt_analysis_tool)

            # FinGPT Market Insights Tool
            fingpt_insights_tool = FinGPTMarketInsightTool(self.fingpt_manager)
            tools.append(fingpt_insights_tool)

            logger.info(f"Initialized {len(tools)} FinGPT agent tools")
            return tools

        except Exception as e:
            logger.error(f"Failed to initialize FinGPT tools: {e}")
            raise
    
    async def analyze(self, query: str, context: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Main analysis method - processes user query and returns comprehensive response
        
        Args:
            query: User's question or request
            context: Additional context (user_id, session_data, etc.)
            
        Returns:
            Dict containing response, metadata, and execution details
        """
        start_time = time.time()
        self.metrics["total_queries"] += 1
        
        try:
            logger.info(f"Starting analysis for query: {query[:100]}...")
            
            # Store query in conversation history
            self.conversation_history.append({
                "type": "user",
                "content": query,
                "timestamp": datetime.now().isoformat(),
                "context": context
            })
            
            # Determine intent and select appropriate LLM
            intent = await self._analyze_intent(query)
            selected_llm = await self.llm_router.route_query(query, intent)
            
            # Update LangGraph flow with selected LLM
            self.langgraph_flow.llm = selected_llm
            
            # Execute the LangGraph flow
            result = await self.langgraph_flow.execute(query)
            
            # Store response in conversation history
            self.conversation_history.append({
                "type": "assistant",
                "content": result["response"],
                "timestamp": datetime.now().isoformat(),
                "metadata": result["metadata"]
            })
            
            # Update metrics
            execution_time = time.time() - start_time
            self.metrics["successful_responses"] += 1
            self.metrics["average_response_time"] = (
                (self.metrics["average_response_time"] * (self.metrics["successful_responses"] - 1) + execution_time) 
                / self.metrics["successful_responses"]
            )
            self.metrics["total_actions_executed"] += result["actions_count"]
            
            # Store successful interaction in memory
            await self._store_interaction_memory(query, result, intent, execution_time)
            
            # Prepare final response
            response = {
                "response": result["response"],
                "intent": intent,
                "llm_used": selected_llm.__class__.__name__,
                "execution_time": execution_time,
                "iterations": result["iterations"],
                "actions_executed": result["actions_count"],
                "observations_made": result["observations_count"],
                "session_id": self.session_id,
                "timestamp": datetime.now().isoformat(),
                "metadata": result["metadata"]
            }
            
            logger.info(f"✅ Analysis completed in {execution_time:.2f}s")
            return response
            
        except Exception as e:
            logger.error(f"Analysis failed: {e}")
            
            # Return error response
            return {
                "response": f"I apologize, but I encountered an error while analyzing your request: {str(e)}",
                "error": str(e),
                "execution_time": time.time() - start_time,
                "session_id": self.session_id,
                "timestamp": datetime.now().isoformat()
            }
    
    async def _analyze_intent(self, query: str) -> str:
        """Analyze user intent to help with LLM routing"""
        query_lower = query.lower()
        
        # Mathematical/analytical queries
        if any(word in query_lower for word in ["calculate", "math", "formula", "percentage", "ratio", "statistics"]):
            return "mathematical"
        
        # Performance analysis
        if any(word in query_lower for word in ["performance", "profit", "loss", "roi", "pnl"]):
            return "performance_analysis"
        
        # Strategy optimization
        if any(word in query_lower for word in ["strategy", "optimize", "improve", "settings", "parameters"]):
            return "strategy_optimization"
        
        # Market analysis
        if any(word in query_lower for word in ["market", "price", "trend", "sentiment", "volatility"]):
            return "market_analysis"
        
        # Configuration changes
        if any(word in query_lower for word in ["config", "setting", "change", "update", "modify"]):
            return "configuration"
        
        # General inquiry
        return "general"

    async def enhanced_trading_analysis(self, query: str, context: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Enhanced trading analysis using TensorZero-inspired features:
        - Multi-agent collaboration
        - Human-in-the-loop approval
        - Advanced confidence scoring
        """
        start_time = time.time()
        self.metrics["total_queries"] += 1

        try:
            logger.info(f"Starting enhanced analysis for: {query[:100]}...")

            # Step 1: Multi-Agent Collaborative Analysis
            collaborative_result = None
            if self.multi_agent_coordinator:
                logger.info("Requesting multi-agent collaborative analysis...")

                analysis_data = {
                    "query": query,
                    "context": context or {},
                    "market_data": await self._get_current_market_data(),
                    "portfolio_data": await self._get_portfolio_data(),
                    "historical_data": await self._get_historical_data()
                }

                collaborative_result = await self.multi_agent_coordinator.collaborative_analysis(analysis_data)
                self.metrics["multi_agent_analyses"] += 1

            # Step 2: Generate Trading Decision
            trading_decision = await self._generate_trading_decision(query, context, collaborative_result)

            # Step 3: Human-in-the-Loop Approval (if needed)
            approval_result = None
            if self.human_loop_manager and trading_decision:
                logger.info("Checking if human approval is needed...")

                approval_request = await self.human_loop_manager.request_approval(trading_decision)

                if approval_request.approval_status == ApprovalStatus.PENDING:
                    self.metrics["human_approvals_requested"] += 1
                    logger.info(f"Human approval requested: {approval_request.request_id}")

                    # Wait for approval (non-blocking for analysis, but log the request)
                    approval_result = {
                        "status": "pending",
                        "request_id": approval_request.request_id,
                        "expires_at": approval_request.expires_at
                    }
                elif approval_request.approval_status == ApprovalStatus.AUTO_APPROVED:
                    self.metrics["auto_approvals"] += 1
                    approval_result = {
                        "status": "auto_approved",
                        "confidence": trading_decision.confidence_score
                    }

            # Step 4: Generate Comprehensive Response
            response = await self._generate_enhanced_response(
                query,
                collaborative_result,
                trading_decision,
                approval_result
            )

            # Step 5: Store in Memory
            if self.memory_manager:
                await self.memory_manager.store_interaction(
                    query=query,
                    response=response["response"],
                    context={
                        "collaborative_analysis": collaborative_result,
                        "trading_decision": trading_decision.__dict__ if trading_decision else None,
                        "approval_result": approval_result,
                        "execution_time": time.time() - start_time
                    }
                )
                self.metrics["memory_entries_created"] += 1

            # Update metrics
            self.metrics["successful_responses"] += 1
            execution_time = time.time() - start_time
            self.metrics["average_response_time"] = (
                (self.metrics["average_response_time"] * (self.metrics["successful_responses"] - 1) + execution_time)
                / self.metrics["successful_responses"]
            )

            logger.info(f"Enhanced analysis completed in {execution_time:.2f}s")

            return {
                **response,
                "execution_time": execution_time,
                "enhancements_used": {
                    "multi_agent": collaborative_result is not None,
                    "human_loop": approval_result is not None,
                    "advanced_confidence": trading_decision is not None
                }
            }

        except Exception as e:
            logger.error(f"Enhanced analysis failed: {e}")
            return {
                "response": f"Analysis failed: {e}",
                "error": str(e),
                "execution_time": time.time() - start_time,
                "enhancements_used": {
                    "multi_agent": False,
                    "human_loop": False,
                    "advanced_confidence": False
                }
            }
    
    async def _store_interaction_memory(self, query: str, result: Dict[str, Any], 
                                      intent: str, execution_time: float):
        """Store successful interaction in memory for future reference"""
        try:
            memory_content = {
                "query": query,
                "response_summary": result["response"][:200] + "..." if len(result["response"]) > 200 else result["response"],
                "intent": intent,
                "execution_time": execution_time,
                "actions_count": result["actions_count"],
                "success": True
            }
            
            await self.memory_manager.store_context(
                content=json.dumps(memory_content),
                context_type="successful_interaction",
                metadata={
                    "source": "trading_analyst_agent",
                    "session_id": self.session_id,
                    "intent": intent,
                    "timestamp": datetime.now().isoformat()
                }
            )
            
            self.metrics["memory_entries_created"] += 1
            
        except Exception as e:
            logger.warning(f"Failed to store interaction memory: {e}")
    
    async def get_conversation_history(self, limit: int = 10) -> List[Dict[str, Any]]:
        """Get recent conversation history"""
        return self.conversation_history[-limit:] if self.conversation_history else []
    
    async def get_agent_status(self) -> Dict[str, Any]:
        """Get current agent status and metrics"""
        return {
            "status": "active" if self.langgraph_flow else "inactive",
            "session_id": self.session_id,
            "metrics": self.metrics,
            "components": {
                "memory_manager": "active" if self.memory_manager else "inactive",
                "llm_router": "active" if self.llm_router else "inactive",
                "tools_count": len(self.tools),
                "langgraph_flow": "active" if self.langgraph_flow else "inactive"
            },
            "conversation_length": len(self.conversation_history),
            "timestamp": datetime.now().isoformat()
        }
    
    async def reset_session(self):
        """Reset the current session"""
        self.session_id = f"session_{int(time.time())}"
        self.conversation_history = []
        logger.info(f"Session reset: {self.session_id}")
    
    async def get_recommendations(self) -> List[Dict[str, Any]]:
        """Get proactive recommendations based on recent data"""
        try:
            # Query recent performance data
            recommendations = []
            
            # Check if we have recent memory entries to analyze
            recent_context = await self.memory_manager.search_relevant_context(
                "performance analysis trading", 
                limit=5
            )
            
            if recent_context:
                recommendations.append({
                    "type": "performance_review",
                    "title": "Recent Performance Analysis Available",
                    "description": "I found recent trading data that might need your attention.",
                    "priority": "medium",
                    "action": "Ask me about your recent trading performance"
                })
            
            # Add more recommendation logic here
            recommendations.append({
                "type": "system_health",
                "title": "System Health Check",
                "description": "Regular system monitoring is recommended.",
                "priority": "low",
                "action": "Ask me to check system health"
            })
            
            return recommendations
            
        except Exception as e:
            logger.error(f"Failed to generate recommendations: {e}")
            return []
    
    async def shutdown(self):
        """Gracefully shutdown the agent"""
        try:
            logger.info("Shutting down TradingAnalystAgent...")
            
            if self.memory_manager:
                await self.memory_manager.close()
            
            if self.llm_router:
                await self.llm_router.close()

            # Shutdown TensorZero enhancements
            if self.multi_agent_coordinator:
                await self.multi_agent_coordinator.stop_all_agents()

            if self.notification_manager:
                await self.notification_manager.close_all()

            logger.info("✅ TradingAnalystAgent shutdown complete")

        except Exception as e:
            logger.error(f"Error during shutdown: {e}")

    async def _generate_trading_decision(self, query: str, context: Optional[Dict], collaborative_result: Optional[Dict]) -> Optional[TradingDecision]:
        """Generate a trading decision from analysis"""
        try:
            # Extract trading intent from query
            if not any(word in query.lower() for word in ["buy", "sell", "trade", "position", "strategy"]):
                return None  # Not a trading query

            # Determine action and token
            action = "hold"
            token_symbol = "SOL"
            amount_sol = 0.1  # Default small amount

            if "buy" in query.lower():
                action = "buy"
            elif "sell" in query.lower():
                action = "sell"

            # Extract token symbol if mentioned
            tokens = ["SOL", "USDC", "RAY", "ORCA", "JUP"]
            for token in tokens:
                if token.lower() in query.lower():
                    token_symbol = token
                    break

            # Calculate confidence from collaborative analysis
            base_confidence = 0.6  # Default
            if collaborative_result and "synthesis" in collaborative_result:
                synthesis = collaborative_result["synthesis"]
                base_confidence = synthesis.get("confidence", 0.6)

            # Get market conditions
            market_conditions = await self._get_current_market_data()

            # Calculate final confidence
            confidence_score = calculate_confidence_score(
                base_confidence,
                market_conditions,
                {"success_rate": 0.75}  # Historical performance placeholder
            )

            # Create trading decision
            decision = TradingDecision(
                decision_id=f"decision_{int(time.time() * 1000)}",
                strategy_type="manual_analysis",
                action=action,
                token_symbol=token_symbol,
                amount_sol=amount_sol,
                confidence_score=confidence_score,
                risk_level=RiskLevel.LOW,  # Will be updated below
                reasoning=f"Analysis based on query: {query}",
                market_conditions=market_conditions,
                timestamp=datetime.now().isoformat(),
                estimated_profit=amount_sol * 0.05,  # 5% estimated profit
                max_loss=amount_sol * 0.1  # 10% max loss
            )

            # Assess risk level
            decision.risk_level = assess_trading_risk(decision)

            return decision

        except Exception as e:
            logger.error(f"Failed to generate trading decision: {e}")
            return None

    async def _get_current_market_data(self) -> Dict[str, Any]:
        """Get current market data"""
        # Placeholder - would integrate with real market data
        return {
            "volatility": 0.3,
            "liquidity_score": 0.7,
            "trend_strength": 0.6,
            "price_change_24h": 0.02,
            "volume_change_24h": 0.15
        }

    async def _get_portfolio_data(self) -> Dict[str, Any]:
        """Get current portfolio data"""
        # Placeholder - would integrate with real portfolio data
        return {
            "total_sol": 8.0,
            "available_sol": 6.5,
            "token_concentration": {"SOL": 0.8, "USDC": 0.2}
        }

    async def _get_historical_data(self) -> Dict[str, Any]:
        """Get historical performance data"""
        # Placeholder - would integrate with real historical data
        return {
            "success_rate": 0.75,
            "average_profit": 0.03,
            "max_drawdown": 0.15,
            "total_trades": 150
        }

    async def _generate_enhanced_response(self, query: str, collaborative_result: Optional[Dict],
                                        trading_decision: Optional[TradingDecision],
                                        approval_result: Optional[Dict]) -> Dict[str, Any]:
        """Generate enhanced response with all analysis components"""

        response_parts = []

        # Add collaborative analysis summary
        if collaborative_result:
            synthesis = collaborative_result.get("synthesis", {})
            response_parts.append(f"🤖 **Multi-Agent Analysis**: {synthesis.get('recommendation', 'HOLD')} "
                                f"(confidence: {synthesis.get('confidence', 0):.1%})")

            # Add individual agent insights
            individual_analyses = collaborative_result.get("individual_analyses", [])
            for analysis in individual_analyses:
                agent_role = analysis.get("agent_role", "unknown")
                recommendation = analysis.get("recommendation", "HOLD")
                confidence = analysis.get("confidence", 0)
                response_parts.append(f"  • {agent_role}: {recommendation} ({confidence:.1%})")

        # Add trading decision info
        if trading_decision:
            response_parts.append(f"\n💡 **Trading Decision**: {trading_decision.action.upper()} "
                                f"{trading_decision.amount_sol:.3f} {trading_decision.token_symbol}")
            response_parts.append(f"  • Confidence: {trading_decision.confidence_score:.1%}")
            response_parts.append(f"  • Risk Level: {trading_decision.risk_level.value.upper()}")
            response_parts.append(f"  • Est. Profit: {trading_decision.estimated_profit:.3f} SOL")

        # Add approval status
        if approval_result:
            if approval_result["status"] == "pending":
                response_parts.append(f"\n⏳ **Human Approval Required**: Request {approval_result['request_id']}")
                response_parts.append(f"  • Expires: {approval_result['expires_at']}")
            elif approval_result["status"] == "auto_approved":
                response_parts.append(f"\n✅ **Auto-Approved**: High confidence ({approval_result['confidence']:.1%})")

        # Add reasoning
        response_parts.append(f"\n📊 **Analysis**: {query}")

        if collaborative_result and "synthesis" in collaborative_result:
            reasoning = collaborative_result["synthesis"].get("reasoning", "")
            if reasoning:
                response_parts.append(f"  • {reasoning}")

        return {
            "response": "\n".join(response_parts),
            "metadata": {
                "collaborative_analysis": collaborative_result,
                "trading_decision": trading_decision.__dict__ if trading_decision else None,
                "approval_result": approval_result,
                "timestamp": datetime.now().isoformat()
            }
        }

# Factory function for easy instantiation
async def create_trading_analyst_agent(config: CerebroConfig) -> TradingAnalystAgent:
    """Factory function to create and initialize a TradingAnalystAgent"""
    agent = TradingAnalystAgent(config)
    await agent.initialize()
    return agent
