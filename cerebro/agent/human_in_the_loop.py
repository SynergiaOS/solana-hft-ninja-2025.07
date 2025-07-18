#!/usr/bin/env python3
"""
Human-in-the-Loop System for Cerebro
Inspired by TensorZero's human oversight capabilities
"""

import asyncio
import json
import time
from typing import Dict, Any, List, Optional, Callable
from datetime import datetime, timedelta
from enum import Enum
from dataclasses import dataclass, asdict
import logging

logger = logging.getLogger(__name__)

class ApprovalStatus(Enum):
    """Status of approval requests"""
    PENDING = "pending"
    APPROVED = "approved"
    REJECTED = "rejected"
    TIMEOUT = "timeout"
    AUTO_APPROVED = "auto_approved"

class RiskLevel(Enum):
    """Risk levels for trading decisions"""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"

@dataclass
class TradingDecision:
    """Represents a trading decision that may need approval"""
    decision_id: str
    strategy_type: str
    action: str  # "buy", "sell", "hold", "close_position"
    token_symbol: str
    amount_sol: float
    confidence_score: float  # 0.0 to 1.0
    risk_level: RiskLevel
    reasoning: str
    market_conditions: Dict[str, Any]
    timestamp: str
    estimated_profit: Optional[float] = None
    max_loss: Optional[float] = None
    execution_deadline: Optional[str] = None

@dataclass
class ApprovalRequest:
    """Approval request for human oversight"""
    request_id: str
    decision: TradingDecision
    approval_status: ApprovalStatus
    created_at: str
    expires_at: str
    approved_by: Optional[str] = None
    approved_at: Optional[str] = None
    rejection_reason: Optional[str] = None
    notification_sent: bool = False

class HumanInTheLoopManager:
    """
    Manages human oversight for trading decisions
    Inspired by TensorZero's human-in-the-loop capabilities
    """
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.pending_requests: Dict[str, ApprovalRequest] = {}
        self.approval_history: List[ApprovalRequest] = []
        
        # Configuration
        self.auto_approval_thresholds = {
            RiskLevel.LOW: 0.85,      # Auto-approve if confidence > 85%
            RiskLevel.MEDIUM: 0.95,   # Auto-approve if confidence > 95%
            RiskLevel.HIGH: 1.0,      # Never auto-approve
            RiskLevel.CRITICAL: 1.0   # Never auto-approve
        }
        
        self.approval_timeouts = {
            RiskLevel.LOW: 300,       # 5 minutes
            RiskLevel.MEDIUM: 600,    # 10 minutes
            RiskLevel.HIGH: 1800,     # 30 minutes
            RiskLevel.CRITICAL: 3600  # 1 hour
        }
        
        # Notification callbacks
        self.notification_callbacks: List[Callable] = []
        
        logger.info("HumanInTheLoopManager initialized")
    
    def add_notification_callback(self, callback: Callable):
        """Add notification callback for approval requests"""
        self.notification_callbacks.append(callback)
    
    async def request_approval(self, decision: TradingDecision) -> ApprovalRequest:
        """
        Request approval for a trading decision
        Returns immediately with approval request object
        """
        request_id = f"approval_{int(time.time() * 1000)}"
        
        # Calculate expiration time
        timeout_seconds = self.approval_timeouts[decision.risk_level]
        expires_at = (datetime.now() + timedelta(seconds=timeout_seconds)).isoformat()
        
        # Create approval request
        approval_request = ApprovalRequest(
            request_id=request_id,
            decision=decision,
            approval_status=ApprovalStatus.PENDING,
            created_at=datetime.now().isoformat(),
            expires_at=expires_at
        )
        
        # Check for auto-approval
        if self._should_auto_approve(decision):
            approval_request.approval_status = ApprovalStatus.AUTO_APPROVED
            approval_request.approved_at = datetime.now().isoformat()
            approval_request.approved_by = "system"
            logger.info(f"Auto-approved decision {decision.decision_id} (confidence: {decision.confidence_score:.2f})")
        else:
            # Store pending request
            self.pending_requests[request_id] = approval_request
            
            # Send notifications
            await self._send_approval_notification(approval_request)
            
            logger.info(f"Approval requested for decision {decision.decision_id} (risk: {decision.risk_level.value})")
        
        return approval_request
    
    def _should_auto_approve(self, decision: TradingDecision) -> bool:
        """Determine if decision should be auto-approved"""
        threshold = self.auto_approval_thresholds[decision.risk_level]
        return decision.confidence_score >= threshold
    
    async def _send_approval_notification(self, request: ApprovalRequest):
        """Send notification to all registered callbacks"""
        try:
            for callback in self.notification_callbacks:
                await callback(request)
            request.notification_sent = True
        except Exception as e:
            logger.error(f"Failed to send approval notification: {e}")
    
    async def approve_request(self, request_id: str, approved_by: str) -> bool:
        """Approve a pending request"""
        if request_id not in self.pending_requests:
            logger.warning(f"Approval request {request_id} not found")
            return False
        
        request = self.pending_requests[request_id]
        
        # Check if not expired
        if datetime.now() > datetime.fromisoformat(request.expires_at):
            request.approval_status = ApprovalStatus.TIMEOUT
            logger.warning(f"Approval request {request_id} has expired")
            return False
        
        # Approve the request
        request.approval_status = ApprovalStatus.APPROVED
        request.approved_by = approved_by
        request.approved_at = datetime.now().isoformat()
        
        # Move to history
        self.approval_history.append(request)
        del self.pending_requests[request_id]
        
        logger.info(f"Request {request_id} approved by {approved_by}")
        return True
    
    async def reject_request(self, request_id: str, rejected_by: str, reason: str) -> bool:
        """Reject a pending request"""
        if request_id not in self.pending_requests:
            logger.warning(f"Approval request {request_id} not found")
            return False
        
        request = self.pending_requests[request_id]
        request.approval_status = ApprovalStatus.REJECTED
        request.approved_by = rejected_by
        request.approved_at = datetime.now().isoformat()
        request.rejection_reason = reason
        
        # Move to history
        self.approval_history.append(request)
        del self.pending_requests[request_id]
        
        logger.info(f"Request {request_id} rejected by {rejected_by}: {reason}")
        return True
    
    async def wait_for_approval(self, request_id: str, timeout_seconds: Optional[int] = None) -> ApprovalStatus:
        """
        Wait for approval decision
        Returns the final approval status
        """
        if request_id not in self.pending_requests:
            # Check if it's in history (already processed)
            for historical_request in self.approval_history:
                if historical_request.request_id == request_id:
                    return historical_request.approval_status
            return ApprovalStatus.TIMEOUT
        
        request = self.pending_requests[request_id]
        
        # Calculate timeout
        if timeout_seconds is None:
            expires_at = datetime.fromisoformat(request.expires_at)
            timeout_seconds = max(1, int((expires_at - datetime.now()).total_seconds()))
        
        # Poll for approval
        start_time = time.time()
        while time.time() - start_time < timeout_seconds:
            if request_id not in self.pending_requests:
                # Request was processed
                for historical_request in self.approval_history:
                    if historical_request.request_id == request_id:
                        return historical_request.approval_status
                return ApprovalStatus.TIMEOUT
            
            await asyncio.sleep(1)  # Check every second
        
        # Timeout reached
        request.approval_status = ApprovalStatus.TIMEOUT
        self.approval_history.append(request)
        del self.pending_requests[request_id]
        
        logger.warning(f"Approval request {request_id} timed out")
        return ApprovalStatus.TIMEOUT
    
    def get_pending_requests(self) -> List[ApprovalRequest]:
        """Get all pending approval requests"""
        return list(self.pending_requests.values())
    
    def get_approval_history(self, limit: int = 100) -> List[ApprovalRequest]:
        """Get approval history"""
        return self.approval_history[-limit:]
    
    def get_approval_stats(self) -> Dict[str, Any]:
        """Get approval statistics"""
        total_requests = len(self.approval_history)
        if total_requests == 0:
            return {"total_requests": 0}
        
        approved = sum(1 for r in self.approval_history if r.approval_status == ApprovalStatus.APPROVED)
        auto_approved = sum(1 for r in self.approval_history if r.approval_status == ApprovalStatus.AUTO_APPROVED)
        rejected = sum(1 for r in self.approval_history if r.approval_status == ApprovalStatus.REJECTED)
        timeout = sum(1 for r in self.approval_history if r.approval_status == ApprovalStatus.TIMEOUT)
        
        return {
            "total_requests": total_requests,
            "approved": approved,
            "auto_approved": auto_approved,
            "rejected": rejected,
            "timeout": timeout,
            "approval_rate": (approved + auto_approved) / total_requests,
            "auto_approval_rate": auto_approved / total_requests,
            "pending": len(self.pending_requests)
        }

# Risk Assessment Functions
def assess_trading_risk(decision: TradingDecision) -> RiskLevel:
    """Assess risk level of a trading decision"""
    
    # High amount = higher risk
    if decision.amount_sol > 2.0:  # More than 25% of 8 SOL portfolio
        return RiskLevel.HIGH
    
    # Low confidence = higher risk
    if decision.confidence_score < 0.6:
        return RiskLevel.HIGH
    elif decision.confidence_score < 0.8:
        return RiskLevel.MEDIUM
    
    # Large potential loss = higher risk
    if decision.max_loss and decision.max_loss > 0.5:  # More than 0.5 SOL loss
        return RiskLevel.HIGH
    
    # Strategy-specific risk assessment
    if decision.strategy_type in ["sandwich", "liquidation"]:
        return RiskLevel.MEDIUM  # MEV strategies are inherently riskier
    
    return RiskLevel.LOW

def calculate_confidence_score(
    strategy_confidence: float,
    market_conditions: Dict[str, Any],
    historical_performance: Optional[Dict[str, Any]] = None
) -> float:
    """Calculate overall confidence score for a trading decision"""
    
    base_confidence = strategy_confidence
    
    # Adjust based on market conditions
    volatility = market_conditions.get("volatility", 0.5)
    if volatility > 0.8:
        base_confidence *= 0.8  # Reduce confidence in high volatility
    
    liquidity = market_conditions.get("liquidity_score", 0.5)
    if liquidity < 0.3:
        base_confidence *= 0.7  # Reduce confidence in low liquidity
    
    # Adjust based on historical performance
    if historical_performance:
        success_rate = historical_performance.get("success_rate", 0.5)
        base_confidence = (base_confidence + success_rate) / 2
    
    return min(1.0, max(0.0, base_confidence))
