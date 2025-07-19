#!/usr/bin/env python3
"""
Gradio External Tools for Cerebro Agent
Provides access to external AI models via Gradio Client API
"""

import asyncio
import json
import logging
from typing import Dict, Any, Optional, Type, List
from datetime import datetime

from langchain_core.tools import BaseTool
from pydantic import BaseModel, Field

try:
    from gradio_client import Client
    GRADIO_AVAILABLE = True
except ImportError:
    GRADIO_AVAILABLE = False
    logging.warning("gradio_client not available. Install with: pip install gradio-client")

logger = logging.getLogger(__name__)

class GradioToolInput(BaseModel):
    """Base input schema for Gradio tools"""
    text: str = Field(description="Input text for analysis")

class TokenRiskInput(BaseModel):
    """Input schema for token risk analysis"""
    token_address: str = Field(description="Solana token address to analyze")
    additional_context: Optional[str] = Field(default="", description="Additional context for analysis")

class SentimentAnalysisInput(BaseModel):
    """Input schema for sentiment analysis"""
    text: str = Field(description="Text to analyze for sentiment")
    source: Optional[str] = Field(default="general", description="Source type (twitter, news, discord)")

class MarketAnalysisInput(BaseModel):
    """Input schema for market analysis"""
    market_data: str = Field(description="Market data or news to analyze")
    timeframe: Optional[str] = Field(default="1h", description="Analysis timeframe")

class GradioTokenRiskAnalyzer(BaseTool):
    """Tool for analyzing token risk using external Gradio models"""
    
    name: str = "gradio_token_risk_analyzer"
    description: str = """
    Analyze Solana token risk using external specialized AI models via Gradio.
    
    This tool connects to external Gradio applications that specialize in:
    - Rug pull detection
    - Token contract analysis
    - Liquidity assessment
    - Creator wallet analysis
    
    Input: Solana token address
    Output: Risk assessment with score and detailed reasoning
    
    Example usage:
    - "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" (USDC)
    - "So11111111111111111111111111111111111111112" (SOL)
    """
    args_schema: Type[BaseModel] = TokenRiskInput
    
    def __init__(self, gradio_endpoints: Optional[Dict[str, str]] = None):
        super().__init__()
        self.gradio_endpoints = gradio_endpoints or {
            "rug_detector": "https://huggingface.co/spaces/CryptoGuard/SolanaRugDetector",
            "token_analyzer": "https://huggingface.co/spaces/DeFiAnalytics/TokenAnalyzer"
        }
        self.clients = {}
        
    def _get_client(self, endpoint_name: str) -> Optional[Client]:
        """Get or create Gradio client for endpoint"""
        if not GRADIO_AVAILABLE:
            return None
            
        if endpoint_name not in self.clients:
            try:
                endpoint_url = self.gradio_endpoints.get(endpoint_name)
                if endpoint_url:
                    self.clients[endpoint_name] = Client(endpoint_url)
                    logger.info(f"Connected to Gradio endpoint: {endpoint_name}")
            except Exception as e:
                logger.error(f"Failed to connect to {endpoint_name}: {e}")
                return None
                
        return self.clients.get(endpoint_name)
    
    def _run(self, token_address: str, additional_context: str = "") -> str:
        """Synchronous execution (fallback)"""
        return asyncio.run(self._arun(token_address, additional_context))
    
    async def _arun(self, token_address: str, additional_context: str = "") -> str:
        """Analyze token risk using external Gradio models"""
        if not GRADIO_AVAILABLE:
            return "Gradio client not available. Please install: pip install gradio-client"
        
        results = []
        
        # Try rug pull detector
        rug_client = self._get_client("rug_detector")
        if rug_client:
            try:
                # Note: API names and parameters depend on the actual Gradio app
                # This is a template - adjust based on real endpoints
                result = rug_client.predict(
                    token_address,
                    api_name="/predict_risk"
                )
                
                if isinstance(result, dict):
                    risk_score = result.get('risk_score', 0.5)
                    reason = result.get('reason', 'No specific reason provided')
                    results.append(f"Rug Pull Risk: {risk_score*100:.0f}% - {reason}")
                else:
                    results.append(f"Rug Pull Analysis: {result}")
                    
            except Exception as e:
                logger.error(f"Rug detector error: {e}")
                results.append(f"Rug detector unavailable: {str(e)}")
        
        # Try general token analyzer
        token_client = self._get_client("token_analyzer")
        if token_client:
            try:
                result = token_client.predict(
                    token_address,
                    additional_context,
                    api_name="/analyze_token"
                )
                
                if isinstance(result, dict):
                    overall_score = result.get('overall_score', 0.5)
                    analysis = result.get('analysis', 'No analysis provided')
                    results.append(f"Token Analysis: {overall_score*100:.0f}% - {analysis}")
                else:
                    results.append(f"Token Analysis: {result}")
                    
            except Exception as e:
                logger.error(f"Token analyzer error: {e}")
                results.append(f"Token analyzer unavailable: {str(e)}")
        
        if not results:
            return f"No external analysis available for token: {token_address}"
        
        return "\n".join(results)

class GradioSentimentAnalyzer(BaseTool):
    """Tool for sentiment analysis using external Gradio models"""
    
    name: str = "gradio_sentiment_analyzer"
    description: str = """
    Analyze sentiment using external specialized AI models via Gradio.
    
    This tool connects to external Gradio applications that specialize in:
    - Crypto-specific sentiment analysis
    - Social media sentiment
    - News sentiment analysis
    - Multi-language sentiment
    
    Input: Text to analyze
    Output: Sentiment classification with confidence score
    """
    args_schema: Type[BaseModel] = SentimentAnalysisInput
    
    def __init__(self, gradio_endpoints: Optional[Dict[str, str]] = None):
        super().__init__()
        self.gradio_endpoints = gradio_endpoints or {
            "crypto_sentiment": "https://huggingface.co/spaces/CryptoSentiment/AnalyzerV2",
            "general_sentiment": "https://huggingface.co/spaces/cardiffnlp/twitter-roberta-base-sentiment-latest"
        }
        self.clients = {}
    
    def _get_client(self, endpoint_name: str) -> Optional[Client]:
        """Get or create Gradio client for endpoint"""
        if not GRADIO_AVAILABLE:
            return None
            
        if endpoint_name not in self.clients:
            try:
                endpoint_url = self.gradio_endpoints.get(endpoint_name)
                if endpoint_url:
                    self.clients[endpoint_name] = Client(endpoint_url)
                    logger.info(f"Connected to Gradio endpoint: {endpoint_name}")
            except Exception as e:
                logger.error(f"Failed to connect to {endpoint_name}: {e}")
                return None
                
        return self.clients.get(endpoint_name)
    
    def _run(self, text: str, source: str = "general") -> str:
        """Synchronous execution (fallback)"""
        return asyncio.run(self._arun(text, source))
    
    async def _arun(self, text: str, source: str = "general") -> str:
        """Analyze sentiment using external Gradio models"""
        if not GRADIO_AVAILABLE:
            return "Gradio client not available. Please install: pip install gradio-client"
        
        results = []
        
        # Try crypto-specific sentiment analyzer
        crypto_client = self._get_client("crypto_sentiment")
        if crypto_client:
            try:
                result = crypto_client.predict(
                    text,
                    api_name="/predict"
                )
                
                if isinstance(result, dict):
                    sentiment = result.get('label', 'NEUTRAL')
                    confidence = result.get('score', 0.5)
                    results.append(f"Crypto Sentiment: {sentiment} ({confidence*100:.1f}%)")
                else:
                    results.append(f"Crypto Sentiment: {result}")
                    
            except Exception as e:
                logger.error(f"Crypto sentiment error: {e}")
                results.append(f"Crypto sentiment unavailable: {str(e)}")
        
        # Try general sentiment analyzer
        general_client = self._get_client("general_sentiment")
        if general_client:
            try:
                result = general_client.predict(
                    text,
                    api_name="/predict"
                )
                
                if isinstance(result, dict):
                    sentiment = result.get('label', 'NEUTRAL')
                    confidence = result.get('score', 0.5)
                    results.append(f"General Sentiment: {sentiment} ({confidence*100:.1f}%)")
                else:
                    results.append(f"General Sentiment: {result}")
                    
            except Exception as e:
                logger.error(f"General sentiment error: {e}")
                results.append(f"General sentiment unavailable: {str(e)}")
        
        if not results:
            return f"No external sentiment analysis available for: {text[:100]}..."
        
        return "\n".join(results)

class GradioMarketAnalyzer(BaseTool):
    """Tool for market analysis using external Gradio models"""
    
    name: str = "gradio_market_analyzer"
    description: str = """
    Analyze market conditions using external specialized AI models via Gradio.
    
    This tool connects to external Gradio applications that specialize in:
    - Technical analysis
    - Market trend prediction
    - Price movement analysis
    - Volume analysis
    
    Input: Market data or news
    Output: Market analysis with predictions and confidence
    """
    args_schema: Type[BaseModel] = MarketAnalysisInput
    
    def __init__(self, gradio_endpoints: Optional[Dict[str, str]] = None):
        super().__init__()
        self.gradio_endpoints = gradio_endpoints or {
            "market_predictor": "https://huggingface.co/spaces/MarketAI/TrendPredictor",
            "technical_analyzer": "https://huggingface.co/spaces/TechAnalysis/CryptoTA"
        }
        self.clients = {}
    
    def _get_client(self, endpoint_name: str) -> Optional[Client]:
        """Get or create Gradio client for endpoint"""
        if not GRADIO_AVAILABLE:
            return None
            
        if endpoint_name not in self.clients:
            try:
                endpoint_url = self.gradio_endpoints.get(endpoint_name)
                if endpoint_url:
                    self.clients[endpoint_name] = Client(endpoint_url)
                    logger.info(f"Connected to Gradio endpoint: {endpoint_name}")
            except Exception as e:
                logger.error(f"Failed to connect to {endpoint_name}: {e}")
                return None
                
        return self.clients.get(endpoint_name)
    
    def _run(self, market_data: str, timeframe: str = "1h") -> str:
        """Synchronous execution (fallback)"""
        return asyncio.run(self._arun(market_data, timeframe))
    
    async def _arun(self, market_data: str, timeframe: str = "1h") -> str:
        """Analyze market using external Gradio models"""
        if not GRADIO_AVAILABLE:
            return "Gradio client not available. Please install: pip install gradio-client"
        
        results = []
        
        # Try market predictor
        predictor_client = self._get_client("market_predictor")
        if predictor_client:
            try:
                result = predictor_client.predict(
                    market_data,
                    timeframe,
                    api_name="/predict_trend"
                )
                
                if isinstance(result, dict):
                    trend = result.get('trend', 'NEUTRAL')
                    confidence = result.get('confidence', 0.5)
                    results.append(f"Market Trend: {trend} ({confidence*100:.1f}%)")
                else:
                    results.append(f"Market Prediction: {result}")
                    
            except Exception as e:
                logger.error(f"Market predictor error: {e}")
                results.append(f"Market predictor unavailable: {str(e)}")
        
        if not results:
            return f"No external market analysis available for timeframe: {timeframe}"
        
        return "\n".join(results)

# Factory function to create all Gradio tools
def create_gradio_tools(custom_endpoints: Optional[Dict[str, Dict[str, str]]] = None) -> List[BaseTool]:
    """
    Create all Gradio external tools with optional custom endpoints
    
    Args:
        custom_endpoints: Dict with tool names as keys and endpoint configs as values
                         Example: {
                             "token_risk": {"rug_detector": "https://..."},
                             "sentiment": {"crypto_sentiment": "https://..."}
                         }
    
    Returns:
        List of configured Gradio tools
    """
    tools = []
    
    if custom_endpoints is None:
        custom_endpoints = {}
    
    # Token Risk Analyzer
    token_endpoints = custom_endpoints.get("token_risk", {})
    tools.append(GradioTokenRiskAnalyzer(token_endpoints if token_endpoints else None))
    
    # Sentiment Analyzer
    sentiment_endpoints = custom_endpoints.get("sentiment", {})
    tools.append(GradioSentimentAnalyzer(sentiment_endpoints if sentiment_endpoints else None))
    
    # Market Analyzer
    market_endpoints = custom_endpoints.get("market", {})
    tools.append(GradioMarketAnalyzer(market_endpoints if market_endpoints else None))
    
    return tools
