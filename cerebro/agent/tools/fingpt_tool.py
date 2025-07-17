#!/usr/bin/env python3
"""
FinGPT Tool for Cerebro Agent
Provides access to FinGPT models for financial analysis
"""

import asyncio
import json
import logging
from typing import Dict, Any, Optional, Type
from datetime import datetime

from langchain_core.tools import BaseTool
from pydantic import BaseModel, Field

logger = logging.getLogger(__name__)

class FinGPTSentimentInput(BaseModel):
    """Input schema for FinGPT sentiment analysis"""
    text: str = Field(description="Financial text to analyze for sentiment")

class FinGPTForecastInput(BaseModel):
    """Input schema for FinGPT price forecasting"""
    ticker: str = Field(description="Stock ticker symbol (e.g., AAPL, MSFT)")
    context: Dict[str, Any] = Field(description="Market context and historical data")

class FinGPTAnalysisInput(BaseModel):
    """Input schema for FinGPT general analysis"""
    text: str = Field(description="Financial text to analyze")
    task: str = Field(default="general", description="Analysis task type")

class FinGPTSentimentTool(BaseTool):
    """Tool for financial sentiment analysis using FinGPT"""
    
    name: str = "fingpt_sentiment_analysis"
    description: str = """
    Analyze financial sentiment using FinGPT specialized model.
    
    This tool uses the state-of-the-art FinGPT sentiment analysis model
    to determine if financial news, reports, or market commentary is
    positive, negative, or neutral.
    
    Input: Financial text (news, reports, social media posts)
    Output: Sentiment (positive/negative/neutral) with confidence score
    
    Example usage:
    - "Tesla reports record quarterly earnings"
    - "Market volatility increases amid inflation concerns"
    - "Bitcoin reaches new all-time high"
    """
    args_schema: Type[BaseModel] = FinGPTSentimentInput
    
    def __init__(self, fingpt_manager):
        super().__init__()
        self.fingpt_manager = fingpt_manager
    
    def _run(self, text: str) -> str:
        """Synchronous run method (not used in async context)"""
        return "Use async version"
    
    async def _arun(self, text: str) -> str:
        """Analyze financial sentiment using FinGPT"""
        try:
            logger.info(f"Analyzing sentiment for text: {text[:100]}...")
            
            # Use FinGPT for sentiment analysis
            result = await self.fingpt_manager.analyze_sentiment(text)
            
            # Format response
            response = {
                "sentiment": result["sentiment"],
                "confidence": result["confidence"],
                "reasoning": result["reasoning"],
                "model_used": result["model_used"],
                "analysis_timestamp": result["timestamp"]
            }
            
            logger.info(f"Sentiment analysis complete: {result['sentiment']} ({result['confidence']:.2f})")
            
            return json.dumps(response, indent=2)
            
        except Exception as e:
            logger.error(f"FinGPT sentiment analysis failed: {e}")
            return json.dumps({
                "sentiment": "neutral",
                "confidence": 0.0,
                "reasoning": f"Analysis failed: {str(e)}",
                "model_used": "none",
                "analysis_timestamp": datetime.now().isoformat()
            })

class FinGPTForecastTool(BaseTool):
    """Tool for price forecasting using FinGPT"""
    
    name: str = "fingpt_price_forecast"
    description: str = """
    Generate price forecasts using FinGPT forecasting model.
    
    This tool uses FinGPT-Forecaster to predict stock price movements
    based on market data, news sentiment, and historical patterns.
    
    Input: Stock ticker and market context
    Output: Price direction forecast (up/down/stable) with reasoning
    
    Example usage:
    - Forecast AAPL price movement for next week
    - Predict SOL price direction based on recent news
    - Analyze TSLA price outlook with earnings context
    """
    args_schema: Type[BaseModel] = FinGPTForecastInput
    
    def __init__(self, fingpt_manager):
        super().__init__()
        self.fingpt_manager = fingpt_manager
    
    def _run(self, ticker: str, context: Dict[str, Any]) -> str:
        """Synchronous run method (not used in async context)"""
        return "Use async version"
    
    async def _arun(self, ticker: str, context: Dict[str, Any]) -> str:
        """Generate price forecast using FinGPT"""
        try:
            logger.info(f"Generating price forecast for {ticker}")
            
            # Use FinGPT for price forecasting
            result = await self.fingpt_manager.forecast_price(ticker, context)
            
            # Format response
            response = {
                "ticker": result["ticker"],
                "forecast": result["forecast"],
                "confidence": result["confidence"],
                "reasoning": result["reasoning"],
                "timeframe": result["timeframe"],
                "model_used": result["model_used"],
                "forecast_timestamp": result["timestamp"]
            }
            
            logger.info(f"Price forecast complete: {ticker} -> {result['forecast']} ({result['confidence']:.2f})")
            
            return json.dumps(response, indent=2)
            
        except Exception as e:
            logger.error(f"FinGPT price forecast failed: {e}")
            return json.dumps({
                "ticker": ticker,
                "forecast": "neutral",
                "confidence": 0.0,
                "reasoning": f"Forecast failed: {str(e)}",
                "timeframe": "unknown",
                "model_used": "none",
                "forecast_timestamp": datetime.now().isoformat()
            })

class FinGPTAnalysisTool(BaseTool):
    """Tool for general financial analysis using FinGPT"""
    
    name: str = "fingpt_financial_analysis"
    description: str = """
    Perform comprehensive financial analysis using FinGPT multi-task model.
    
    This tool uses FinGPT's multi-task capabilities for various financial
    analysis tasks including NER, relation extraction, and text classification.
    
    Input: Financial text and analysis task type
    Output: Structured analysis results based on task type
    
    Supported tasks:
    - sentiment: Sentiment analysis
    - ner: Named entity recognition
    - relation_extraction: Financial relationship extraction
    - headline_classification: News headline classification
    - general: General financial analysis
    """
    args_schema: Type[BaseModel] = FinGPTAnalysisInput
    
    def __init__(self, fingpt_manager):
        super().__init__()
        self.fingpt_manager = fingpt_manager
    
    def _run(self, text: str, task: str = "general") -> str:
        """Synchronous run method (not used in async context)"""
        return "Use async version"
    
    async def _arun(self, text: str, task: str = "general") -> str:
        """Perform financial analysis using FinGPT"""
        try:
            logger.info(f"Performing {task} analysis for text: {text[:100]}...")
            
            # Use FinGPT for financial analysis
            result = await self.fingpt_manager.analyze_financial_text(text, task)
            
            # Format response
            response = {
                "text": result["text"],
                "task": result["task"],
                "analysis_result": result["result"],
                "model_used": result["model_used"],
                "analysis_timestamp": result["timestamp"]
            }
            
            logger.info(f"Financial analysis complete: {task}")
            
            return json.dumps(response, indent=2)
            
        except Exception as e:
            logger.error(f"FinGPT financial analysis failed: {e}")
            return json.dumps({
                "text": text,
                "task": task,
                "analysis_result": f"Analysis failed: {str(e)}",
                "model_used": "none",
                "analysis_timestamp": datetime.now().isoformat()
            })

class FinGPTMarketInsightTool(BaseTool):
    """Tool for market insights using FinGPT"""
    
    name: str = "fingpt_market_insights"
    description: str = """
    Generate market insights and trading recommendations using FinGPT.
    
    This tool combines multiple FinGPT capabilities to provide comprehensive
    market analysis, sentiment assessment, and trading insights.
    
    Input: Market data, news, and trading context
    Output: Actionable market insights and recommendations
    
    Use this tool for:
    - Market condition analysis
    - Trading opportunity identification
    - Risk assessment and recommendations
    - Strategy optimization suggestions
    """
    args_schema: Type[BaseModel] = FinGPTAnalysisInput
    
    def __init__(self, fingpt_manager):
        super().__init__()
        self.fingpt_manager = fingpt_manager
    
    def _run(self, text: str, task: str = "market_insights") -> str:
        """Synchronous run method (not used in async context)"""
        return "Use async version"
    
    async def _arun(self, text: str, task: str = "market_insights") -> str:
        """Generate market insights using FinGPT"""
        try:
            logger.info(f"Generating market insights for: {text[:100]}...")
            
            # Perform sentiment analysis first
            sentiment_result = await self.fingpt_manager.analyze_sentiment(text)
            
            # Perform general financial analysis
            analysis_result = await self.fingpt_manager.analyze_financial_text(text, "general")
            
            # Combine results into market insights
            insights = {
                "market_sentiment": {
                    "sentiment": sentiment_result["sentiment"],
                    "confidence": sentiment_result["confidence"],
                    "reasoning": sentiment_result["reasoning"]
                },
                "financial_analysis": {
                    "result": analysis_result["result"]
                },
                "trading_recommendations": self._generate_trading_recommendations(
                    sentiment_result, analysis_result
                ),
                "risk_assessment": self._assess_market_risk(sentiment_result),
                "model_used": sentiment_result["model_used"],
                "analysis_timestamp": datetime.now().isoformat()
            }
            
            logger.info("Market insights generation complete")
            
            return json.dumps(insights, indent=2)
            
        except Exception as e:
            logger.error(f"FinGPT market insights failed: {e}")
            return json.dumps({
                "market_sentiment": {"sentiment": "neutral", "confidence": 0.0},
                "financial_analysis": {"result": f"Analysis failed: {str(e)}"},
                "trading_recommendations": ["Unable to generate recommendations"],
                "risk_assessment": "Unknown risk level",
                "model_used": "none",
                "analysis_timestamp": datetime.now().isoformat()
            })
    
    def _generate_trading_recommendations(self, sentiment_result: Dict, analysis_result: Dict) -> List[str]:
        """Generate trading recommendations based on analysis"""
        recommendations = []
        
        sentiment = sentiment_result.get("sentiment", "neutral")
        confidence = sentiment_result.get("confidence", 0.0)
        
        if sentiment == "positive" and confidence > 0.7:
            recommendations.extend([
                "Consider increasing position sizes for bullish strategies",
                "Monitor for breakout opportunities",
                "Reduce hedging positions if market sentiment continues"
            ])
        elif sentiment == "negative" and confidence > 0.7:
            recommendations.extend([
                "Implement defensive strategies",
                "Consider reducing exposure to volatile assets",
                "Increase stop-loss protection"
            ])
        else:
            recommendations.extend([
                "Maintain current position sizing",
                "Monitor market developments closely",
                "Prepare for potential volatility"
            ])
        
        return recommendations
    
    def _assess_market_risk(self, sentiment_result: Dict) -> str:
        """Assess market risk based on sentiment"""
        sentiment = sentiment_result.get("sentiment", "neutral")
        confidence = sentiment_result.get("confidence", 0.0)
        
        if sentiment == "negative" and confidence > 0.8:
            return "High risk - Strong negative sentiment detected"
        elif sentiment == "positive" and confidence > 0.8:
            return "Low risk - Strong positive sentiment detected"
        elif confidence < 0.5:
            return "Medium risk - Uncertain market sentiment"
        else:
            return "Medium risk - Neutral market conditions"
