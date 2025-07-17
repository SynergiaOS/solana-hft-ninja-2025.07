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
        
        # Performance metrics
        self.metrics = {
            "total_queries": 0,
            "successful_responses": 0,
            "average_response_time": 0.0,
            "total_actions_executed": 0,
            "memory_entries_created": 0
        }
        
        logger.info("TradingAnalystAgent initialized")
    
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
            
            logger.info("✅ TradingAnalystAgent shutdown complete")
            
        except Exception as e:
            logger.error(f"Error during shutdown: {e}")

# Factory function for easy instantiation
async def create_trading_analyst_agent(config: CerebroConfig) -> TradingAnalystAgent:
    """Factory function to create and initialize a TradingAnalystAgent"""
    agent = TradingAnalystAgent(config)
    await agent.initialize()
    return agent
