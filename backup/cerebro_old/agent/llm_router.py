#!/usr/bin/env python3
"""
Cerebro LLM Router
Routes queries to appropriate language models based on intent
"""

import re
import logging
from typing import Dict, List, Any, Optional, Tuple
from enum import Enum
from dataclasses import dataclass

logger = logging.getLogger(__name__)


class ModelType(Enum):
    """Available language models"""
    DEEPSEEK_MATH = "deepseek_math"
    FINGPT = "fingpt"
    FALLBACK = "fallback"


class QueryIntent(Enum):
    """Types of query intents"""
    QUANTITATIVE_ANALYSIS = "quantitative_analysis"
    FINANCIAL_ANALYSIS = "financial_analysis"
    STRATEGY_OPTIMIZATION = "strategy_optimization"
    RISK_ASSESSMENT = "risk_assessment"
    MARKET_ANALYSIS = "market_analysis"
    PERFORMANCE_REVIEW = "performance_review"
    GENERAL_QUESTION = "general_question"
    TECHNICAL_ISSUE = "technical_issue"


@dataclass
class RoutingDecision:
    """Result of routing decision"""
    model_type: ModelType
    intent: QueryIntent
    confidence: float
    reasoning: str
    suggested_prompt: Optional[str] = None


class LLMRouter:
    """Router for selecting appropriate LLM based on query intent"""

    def __init__(self):
        # Keywords for intent detection
        self.intent_keywords = {
            QueryIntent.QUANTITATIVE_ANALYSIS: [
                "calculate", "compute", "math", "statistics", "probability",
                "percentage", "ratio", "correlation", "regression", "variance",
                "standard deviation", "mean", "median", "quantile", "distribution",
                "optimization", "algorithm", "formula", "equation"
            ],
            QueryIntent.FINANCIAL_ANALYSIS: [
                "profit", "loss", "revenue", "cost", "margin", "roi", "return",
                "investment", "portfolio", "risk", "volatility", "sharpe",
                "drawdown", "pnl", "balance", "capital", "liquidity",
                "valuation", "price", "market cap", "volume"
            ],
            QueryIntent.STRATEGY_OPTIMIZATION: [
                "strategy", "optimize", "improve", "enhance", "tune", "adjust",
                "parameter", "setting", "configuration", "performance",
                "efficiency", "effectiveness", "backtest", "forward test"
            ],
            QueryIntent.RISK_ASSESSMENT: [
                "risk", "danger", "threat", "exposure", "hedge", "protection",
                "safety", "security", "limit", "stop loss", "take profit",
                "circuit breaker", "position size", "leverage"
            ],
            QueryIntent.MARKET_ANALYSIS: [
                "market", "trend", "sentiment", "analysis", "forecast",
                "prediction", "outlook", "condition", "environment",
                "opportunity", "signal", "indicator", "pattern"
            ],
            QueryIntent.PERFORMANCE_REVIEW: [
                "performance", "review", "report", "summary", "analysis",
                "results", "outcome", "achievement", "success", "failure",
                "metrics", "kpi", "benchmark", "comparison"
            ],
            QueryIntent.TECHNICAL_ISSUE: [
                "error", "bug", "issue", "problem", "fix", "debug", "troubleshoot",
                "crash", "fail", "exception", "timeout", "connection",
                "configuration", "setup", "install"
            ]
        }

        # Model preferences for each intent
        self.model_preferences = {
            QueryIntent.QUANTITATIVE_ANALYSIS: ModelType.DEEPSEEK_MATH,
            QueryIntent.FINANCIAL_ANALYSIS: ModelType.FINGPT,
            QueryIntent.STRATEGY_OPTIMIZATION: ModelType.FINGPT,
            QueryIntent.RISK_ASSESSMENT: ModelType.FINGPT,
            QueryIntent.MARKET_ANALYSIS: ModelType.FINGPT,
            QueryIntent.PERFORMANCE_REVIEW: ModelType.FINGPT,
            QueryIntent.GENERAL_QUESTION: ModelType.FINGPT,
            QueryIntent.TECHNICAL_ISSUE: ModelType.DEEPSEEK_MATH
        }

    def route_query(self, query: str, context: Optional[Dict[str, Any]] = None) -> RoutingDecision:
        """Route query to appropriate model"""
        try:
            # Detect intent
            intent, confidence = self._detect_intent(query)

            # Select model based on intent
            model_type = self.model_preferences.get(intent, ModelType.FALLBACK)

            # Generate reasoning
            reasoning = self._generate_reasoning(query, intent, model_type, confidence)

            # Suggest optimized prompt
            suggested_prompt = self._optimize_prompt(query, intent, model_type, context)

            return RoutingDecision(
                model_type=model_type,
                intent=intent,
                confidence=confidence,
                reasoning=reasoning,
                suggested_prompt=suggested_prompt
            )

        except Exception as e:
            logger.error(f"Routing failed: {e}")
            return RoutingDecision(
                model_type=ModelType.FALLBACK,
                intent=QueryIntent.GENERAL_QUESTION,
                confidence=0.0,
                reasoning=f"Routing error: {e}",
                suggested_prompt=query
            )

    def _detect_intent(self, query: str) -> Tuple[QueryIntent, float]:
        """Detect query intent based on keywords"""
        query_lower = query.lower()
        intent_scores = {}

        # Score each intent based on keyword matches
        for intent, keywords in self.intent_keywords.items():
            score = 0
            matched_keywords = []

            for keyword in keywords:
                if keyword in query_lower:
                    score += 1
                    matched_keywords.append(keyword)

            # Normalize score by number of keywords
            normalized_score = score / len(keywords) if keywords else 0
            intent_scores[intent] = (normalized_score, matched_keywords)

        # Find best match
        if not intent_scores:
            return QueryIntent.GENERAL_QUESTION, 0.0

        best_intent = max(intent_scores.keys(), key=lambda x: intent_scores[x][0])
        best_score, matched = intent_scores[best_intent]

        # If no keywords matched, default to general question
        if best_score == 0:
            return QueryIntent.GENERAL_QUESTION, 0.1

        # Convert to confidence (0-1 scale)
        confidence = min(best_score * 10, 1.0)  # Scale up and cap at 1.0

        logger.info(f"Intent detected: {best_intent.value} (confidence: {confidence:.2f}, keywords: {matched})")
        return best_intent, confidence

    def _generate_reasoning(self, query: str, intent: QueryIntent, model_type: ModelType, confidence: float) -> str:
        """Generate human-readable reasoning for routing decision"""
        reasoning_parts = [
            f"Query intent detected as '{intent.value}' with {confidence:.1%} confidence.",
            f"Routing to {model_type.value} model."
        ]

        # Add model-specific reasoning
        if model_type == ModelType.DEEPSEEK_MATH:
            reasoning_parts.append("DeepSeek-Math selected for quantitative analysis and technical problem-solving.")
        elif model_type == ModelType.FINGPT:
            reasoning_parts.append("FinGPT selected for financial analysis and trading strategy insights.")
        else:
            reasoning_parts.append("Using fallback model due to routing uncertainty.")

        return " ".join(reasoning_parts)

    def _optimize_prompt(self, query: str, intent: QueryIntent, model_type: ModelType, context: Optional[Dict[str, Any]]) -> str:
        """Optimize prompt for specific model and intent"""

        # Base prompt optimization based on model type
        if model_type == ModelType.DEEPSEEK_MATH:
            prompt_prefix = "As a quantitative analyst, please analyze the following:\n\n"
        elif model_type == ModelType.FINGPT:
            prompt_prefix = "As a financial trading expert, please provide insights on:\n\n"
        else:
            prompt_prefix = "Please help with the following question:\n\n"

        # Add context if available
        context_str = ""
        if context:
            if "hft_stats" in context:
                context_str += f"Current HFT Performance: {context['hft_stats']}\n"
            if "recent_trades" in context:
                context_str += f"Recent Trading Activity: {context['recent_trades']}\n"
            if "market_conditions" in context:
                context_str += f"Market Conditions: {context['market_conditions']}\n"

        # Intent-specific optimizations
        intent_instructions = {
            QueryIntent.QUANTITATIVE_ANALYSIS: "Please provide detailed calculations and statistical analysis.",
            QueryIntent.FINANCIAL_ANALYSIS: "Focus on financial metrics, profitability, and risk assessment.",
            QueryIntent.STRATEGY_OPTIMIZATION: "Suggest specific improvements and optimization strategies.",
            QueryIntent.RISK_ASSESSMENT: "Evaluate risks and recommend risk mitigation measures.",
            QueryIntent.MARKET_ANALYSIS: "Analyze market trends and trading opportunities.",
            QueryIntent.PERFORMANCE_REVIEW: "Provide comprehensive performance analysis with actionable insights."
        }

        instruction = intent_instructions.get(intent, "Provide a helpful and detailed response.")

        optimized_prompt = f"{prompt_prefix}{context_str}{query}\n\n{instruction}"

        return optimized_prompt