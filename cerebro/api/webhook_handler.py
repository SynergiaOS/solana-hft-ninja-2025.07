#!/usr/bin/env python3
"""
Webhook Handler for HFT Ninja → Cerebro Communication
Handles real-time events from Rust HFT engine
"""

import asyncio
import json
import logging
from datetime import datetime
from typing import Dict, Any, Optional
from fastapi import FastAPI, HTTPException, BackgroundTasks
from pydantic import BaseModel, Field

from ..memory.rag_search import CerebroRAGSearch
from ..memory.schema import ContextEntry, ContextType, ContextSource
from ..agent.trading_analyst_agent import TradingAnalystAgent
from ..agent.langgraph_flow import CerebroLangGraphFlow

logger = logging.getLogger(__name__)

# Webhook payload models
class OpportunityEvent(BaseModel):
    """MEV opportunity detected by HFT Ninja"""
    event_type: str = "opportunity_detected"
    token_address: str
    opportunity_type: str  # sandwich, arbitrage, sniping, etc.
    confidence: float = Field(ge=0.0, le=1.0)
    profit_potential: float
    risk_score: float = Field(ge=0.0, le=1.0)
    trigger_wallet: Optional[str] = None
    dex_involved: str
    timestamp: float
    metadata: Dict[str, Any] = {}

class ExecutionEvent(BaseModel):
    """Trade execution result from HFT Ninja"""
    event_type: str = "execution_result"
    transaction_id: str
    strategy: str
    token_address: str
    outcome: str  # success, failure, partial
    pnl_sol: float
    execution_time_ms: int
    gas_used: int
    trigger_wallet: Optional[str] = None
    timestamp: float
    metadata: Dict[str, Any] = {}

class RiskEvent(BaseModel):
    """Risk management event from HFT Ninja"""
    event_type: str = "risk_event"
    risk_type: str  # stop_loss, circuit_breaker, position_limit
    severity: str  # low, medium, high, critical
    description: str
    affected_strategies: list[str]
    action_taken: str
    timestamp: float
    metadata: Dict[str, Any] = {}

class WalletEvent(BaseModel):
    """Wallet tracking event from HFT Ninja"""
    event_type: str = "wallet_event"
    wallet_address: str
    event_subtype: str  # new_token, large_trade, suspicious_activity
    token_address: Optional[str] = None
    amount_sol: Optional[float] = None
    confidence: float = Field(ge=0.0, le=1.0)
    timestamp: float
    metadata: Dict[str, Any] = {}

class WebhookHandler:
    """Handles webhooks from HFT Ninja Rust engine"""
    
    def __init__(self):
        self.rag_search = CerebroRAGSearch()
        self.trading_agent = None
        self.langgraph_flow = None
        self.processing_queue = asyncio.Queue()
        self.is_processing = False
        
    async def initialize(self):
        """Initialize webhook handler"""
        try:
            await self.rag_search.connect()
            
            # Initialize trading agent
            self.trading_agent = TradingAnalystAgent()
            await self.trading_agent.initialize()
            
            # Initialize LangGraph flow
            self.langgraph_flow = CerebroLangGraphFlow()
            await self.langgraph_flow.initialize()
            
            # Start background processing
            asyncio.create_task(self._process_events())
            self.is_processing = True
            
            logger.info("✅ Webhook handler initialized")
            
        except Exception as e:
            logger.error(f"❌ Failed to initialize webhook handler: {e}")
            raise
    
    async def handle_opportunity_event(self, event: OpportunityEvent, background_tasks: BackgroundTasks):
        """Handle MEV opportunity detection from HFT Ninja"""
        try:
            logger.info(f"🎯 Opportunity detected: {event.opportunity_type} for {event.token_address}")
            
            # Store in memory immediately
            context = ContextEntry(
                context_id=f"opp_{event.token_address}_{int(event.timestamp)}",
                content=f"MEV opportunity detected: {event.opportunity_type} for token {event.token_address} "
                       f"with {event.confidence:.2%} confidence and {event.profit_potential:.4f} SOL potential profit",
                context_type=ContextType.MEV_OPPORTUNITY,
                source=ContextSource.HFT_NINJA_WEBHOOK,
                timestamp=event.timestamp,
                confidence=event.confidence,
                related_strategy=event.opportunity_type,
                metadata={
                    "token_address": event.token_address,
                    "profit_potential": event.profit_potential,
                    "risk_score": event.risk_score,
                    "trigger_wallet": event.trigger_wallet,
                    "dex_involved": event.dex_involved,
                    **event.metadata
                }
            )
            
            await self.rag_search.store_context(context)
            
            # Queue for AI analysis
            await self.processing_queue.put(("opportunity", event))
            
            return {"status": "received", "context_id": context.context_id}
            
        except Exception as e:
            logger.error(f"❌ Failed to handle opportunity event: {e}")
            raise HTTPException(status_code=500, detail=str(e))
    
    async def handle_execution_event(self, event: ExecutionEvent, background_tasks: BackgroundTasks):
        """Handle trade execution result from HFT Ninja"""
        try:
            logger.info(f"📊 Execution result: {event.outcome} for {event.strategy} - PnL: {event.pnl_sol:.4f} SOL")
            
            # Determine context type based on outcome
            context_type = ContextType.TRADE_OUTCOME_SUCCESS if event.outcome == "success" else ContextType.TRADE_OUTCOME_FAILURE
            
            # Store in memory
            context = ContextEntry(
                context_id=f"exec_{event.transaction_id}_{int(event.timestamp)}",
                content=f"Trade execution {event.outcome}: {event.strategy} strategy on {event.token_address} "
                       f"resulted in {event.pnl_sol:.4f} SOL PnL in {event.execution_time_ms}ms",
                context_type=context_type,
                source=ContextSource.HFT_NINJA_WEBHOOK,
                timestamp=event.timestamp,
                confidence=1.0 if event.outcome == "success" else 0.3,
                related_strategy=event.strategy,
                metadata={
                    "transaction_id": event.transaction_id,
                    "token_address": event.token_address,
                    "pnl_sol": event.pnl_sol,
                    "execution_time_ms": event.execution_time_ms,
                    "gas_used": event.gas_used,
                    "trigger_wallet": event.trigger_wallet,
                    **event.metadata
                }
            )
            
            await self.rag_search.store_context(context)
            
            # Queue for AI learning
            await self.processing_queue.put(("execution", event))
            
            return {"status": "received", "context_id": context.context_id}
            
        except Exception as e:
            logger.error(f"❌ Failed to handle execution event: {e}")
            raise HTTPException(status_code=500, detail=str(e))
    
    async def handle_risk_event(self, event: RiskEvent, background_tasks: BackgroundTasks):
        """Handle risk management event from HFT Ninja"""
        try:
            logger.warning(f"⚠️ Risk event: {event.risk_type} - {event.severity} - {event.description}")
            
            # Store in memory
            context = ContextEntry(
                context_id=f"risk_{event.risk_type}_{int(event.timestamp)}",
                content=f"Risk event triggered: {event.risk_type} with {event.severity} severity. "
                       f"Description: {event.description}. Action taken: {event.action_taken}",
                context_type=ContextType.RISK_ALERT,
                source=ContextSource.HFT_NINJA_WEBHOOK,
                timestamp=event.timestamp,
                confidence=0.9,  # High confidence for risk events
                metadata={
                    "risk_type": event.risk_type,
                    "severity": event.severity,
                    "affected_strategies": event.affected_strategies,
                    "action_taken": event.action_taken,
                    **event.metadata
                }
            )
            
            await self.rag_search.store_context(context)
            
            # Queue for immediate AI analysis if critical
            if event.severity in ["high", "critical"]:
                await self.processing_queue.put(("risk_critical", event))
            else:
                await self.processing_queue.put(("risk", event))
            
            return {"status": "received", "context_id": context.context_id}
            
        except Exception as e:
            logger.error(f"❌ Failed to handle risk event: {e}")
            raise HTTPException(status_code=500, detail=str(e))
    
    async def handle_wallet_event(self, event: WalletEvent, background_tasks: BackgroundTasks):
        """Handle wallet tracking event from HFT Ninja"""
        try:
            logger.info(f"👛 Wallet event: {event.event_subtype} from {event.wallet_address}")
            
            # Store in memory
            context = ContextEntry(
                context_id=f"wallet_{event.wallet_address}_{int(event.timestamp)}",
                content=f"Wallet tracking event: {event.event_subtype} from wallet {event.wallet_address}. "
                       f"Token: {event.token_address}, Amount: {event.amount_sol} SOL",
                context_type=ContextType.WALLET_ACTIVITY,
                source=ContextSource.HFT_NINJA_WEBHOOK,
                timestamp=event.timestamp,
                confidence=event.confidence,
                metadata={
                    "wallet_address": event.wallet_address,
                    "event_subtype": event.event_subtype,
                    "token_address": event.token_address,
                    "amount_sol": event.amount_sol,
                    **event.metadata
                }
            )
            
            await self.rag_search.store_context(context)
            
            # Queue for AI analysis
            await self.processing_queue.put(("wallet", event))
            
            return {"status": "received", "context_id": context.context_id}
            
        except Exception as e:
            logger.error(f"❌ Failed to handle wallet event: {e}")
            raise HTTPException(status_code=500, detail=str(e))
    
    async def _process_events(self):
        """Background task to process events with AI"""
        while self.is_processing:
            try:
                # Get event from queue with timeout
                event_type, event_data = await asyncio.wait_for(
                    self.processing_queue.get(), 
                    timeout=1.0
                )
                
                # Process based on event type
                if event_type == "opportunity":
                    await self._process_opportunity_ai(event_data)
                elif event_type == "execution":
                    await self._process_execution_ai(event_data)
                elif event_type == "risk_critical":
                    await self._process_critical_risk_ai(event_data)
                elif event_type == "risk":
                    await self._process_risk_ai(event_data)
                elif event_type == "wallet":
                    await self._process_wallet_ai(event_data)
                
                # Mark task as done
                self.processing_queue.task_done()
                
            except asyncio.TimeoutError:
                # No events to process, continue
                continue
            except Exception as e:
                logger.error(f"❌ Error processing event: {e}")
                continue
    
    async def _process_opportunity_ai(self, event: OpportunityEvent):
        """Process opportunity with AI analysis"""
        try:
            # Use LangGraph flow for complex analysis
            analysis_result = await self.langgraph_flow.analyze_opportunity({
                "token_address": event.token_address,
                "opportunity_type": event.opportunity_type,
                "confidence": event.confidence,
                "profit_potential": event.profit_potential,
                "risk_score": event.risk_score,
                "trigger_wallet": event.trigger_wallet,
                "dex_involved": event.dex_involved,
                "metadata": event.metadata
            })
            
            logger.info(f"🧠 AI analysis for opportunity: {analysis_result.get('recommendation', 'No recommendation')}")
            
        except Exception as e:
            logger.error(f"❌ Failed to process opportunity with AI: {e}")
    
    async def _process_execution_ai(self, event: ExecutionEvent):
        """Process execution result with AI learning"""
        try:
            # Use trading agent for learning from execution
            learning_result = await self.trading_agent.learn_from_execution({
                "strategy": event.strategy,
                "outcome": event.outcome,
                "pnl_sol": event.pnl_sol,
                "execution_time_ms": event.execution_time_ms,
                "token_address": event.token_address,
                "trigger_wallet": event.trigger_wallet,
                "metadata": event.metadata
            })
            
            logger.info(f"🧠 AI learning from execution: {learning_result.get('insights', 'No insights')}")
            
        except Exception as e:
            logger.error(f"❌ Failed to process execution with AI: {e}")
    
    async def _process_critical_risk_ai(self, event: RiskEvent):
        """Process critical risk event with immediate AI response"""
        try:
            # Use LangGraph flow for immediate risk assessment
            risk_analysis = await self.langgraph_flow.assess_critical_risk({
                "risk_type": event.risk_type,
                "severity": event.severity,
                "description": event.description,
                "affected_strategies": event.affected_strategies,
                "action_taken": event.action_taken,
                "metadata": event.metadata
            })
            
            logger.warning(f"🚨 Critical risk AI analysis: {risk_analysis.get('recommendation', 'No recommendation')}")
            
        except Exception as e:
            logger.error(f"❌ Failed to process critical risk with AI: {e}")
    
    async def _process_risk_ai(self, event: RiskEvent):
        """Process regular risk event with AI analysis"""
        try:
            # Store risk pattern for future reference
            risk_pattern = {
                "risk_type": event.risk_type,
                "severity": event.severity,
                "description": event.description,
                "timestamp": event.timestamp
            }
            
            # Use trading agent to analyze risk patterns
            pattern_analysis = await self.trading_agent.analyze_risk_pattern(risk_pattern)
            
            logger.info(f"🛡️ Risk pattern analysis: {pattern_analysis.get('insights', 'No insights')}")
            
        except Exception as e:
            logger.error(f"❌ Failed to process risk with AI: {e}")
    
    async def _process_wallet_ai(self, event: WalletEvent):
        """Process wallet event with AI analysis"""
        try:
            # Use LangGraph flow for wallet behavior analysis
            wallet_analysis = await self.langgraph_flow.analyze_wallet_behavior({
                "wallet_address": event.wallet_address,
                "event_subtype": event.event_subtype,
                "token_address": event.token_address,
                "amount_sol": event.amount_sol,
                "confidence": event.confidence,
                "metadata": event.metadata
            })
            
            logger.info(f"👛 Wallet AI analysis: {wallet_analysis.get('behavior_score', 'No score')}")
            
        except Exception as e:
            logger.error(f"❌ Failed to process wallet event with AI: {e}")
    
    async def get_recent_events(self, event_type: Optional[str] = None, limit: int = 50) -> list:
        """Get recent events from memory"""
        try:
            from ..memory.schema import SearchQuery
            
            # Build search query
            query = SearchQuery(
                query_text=f"recent {event_type or 'events'}",
                context_types=[ContextType.MEV_OPPORTUNITY, ContextType.TRADE_OUTCOME_SUCCESS, 
                              ContextType.TRADE_OUTCOME_FAILURE, ContextType.RISK_ALERT, 
                              ContextType.WALLET_ACTIVITY] if not event_type else None,
                max_results=limit,
                similarity_threshold=0.1  # Low threshold for recent events
            )
            
            # Search in memory
            results = await self.rag_search.search(query)
            
            # Convert to simple format
            events = []
            for result in results:
                events.append({
                    "context_id": result.context_entry.context_id,
                    "content": result.context_entry.content,
                    "type": result.context_entry.context_type.value,
                    "timestamp": result.context_entry.timestamp,
                    "confidence": result.context_entry.confidence,
                    "similarity": result.similarity_score
                })
            
            return events
            
        except Exception as e:
            logger.error(f"❌ Failed to get recent events: {e}")
            return []
    
    async def shutdown(self):
        """Shutdown webhook handler"""
        self.is_processing = False
        await self.rag_search.close()
        logger.info("✅ Webhook handler shutdown complete")

# Global webhook handler instance
webhook_handler = WebhookHandler()
