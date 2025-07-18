"""
🧮 DeepSeek-Math Integration for Solana HFT Trading
Small expert model for mathematical trading calculations and risk assessment.
Cost-effective alternative to large models - optimized for <$1 operational cost.
"""

import asyncio
import json
import logging
import time
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, asdict
from pathlib import Path
import numpy as np

import torch
from transformers import (
    AutoTokenizer, 
    AutoModelForCausalLM, 
    BitsAndBytesConfig,
    pipeline
)
from peft import PeftModel, LoraConfig, get_peft_model
import vllm
from lmcache import LMCache

# Local imports
from .base_ai import BaseAI
from ..config.ai_config import DeepSeekConfig
from ..memory.rag_search import RAGSearch
from ..utils.metrics import AIMetrics

logger = logging.getLogger(__name__)

@dataclass
class TradingCalculation:
    """Mathematical trading calculation result"""
    calculation_type: str
    input_params: Dict[str, Any]
    result: Union[float, Dict[str, float]]
    confidence: float
    reasoning: str
    execution_time_ms: int
    model_used: str

@dataclass
class RiskAssessment:
    """Risk assessment for trading decision"""
    risk_score: float  # 0.0 (safe) to 1.0 (dangerous)
    risk_factors: List[str]
    recommended_position_size: float
    max_loss_estimate: float
    confidence: float
    reasoning: str

class DeepSeekMath(BaseAI):
    """
    DeepSeek-Math integration for trading calculations.
    Optimized for small portfolio (<$100) with minimal operational costs.
    """
    
    def __init__(self, config: DeepSeekConfig):
        super().__init__(config)
        self.config = config
        self.model = None
        self.tokenizer = None
        self.pipeline = None
        self.lmcache = None
        self.metrics = AIMetrics("deepseek_math")
        
        # Trading-specific prompts
        self.prompts = {
            "position_sizing": """
Calculate optimal position size for Solana trading:
- Available capital: {capital} SOL
- Risk tolerance: {risk_tolerance}%
- Expected return: {expected_return}%
- Market volatility: {volatility}
- Strategy: {strategy}

Use Kelly Criterion and risk-adjusted sizing. Return JSON with:
{{"position_size": float, "risk_score": float, "reasoning": str}}
""",
            
            "arbitrage_profit": """
Calculate arbitrage profit potential:
- Token: {token}
- Price DEX A: {price_a} SOL
- Price DEX B: {price_b} SOL
- Liquidity A: {liquidity_a} SOL
- Liquidity B: {liquidity_b} SOL
- Gas costs: {gas_cost} SOL

Calculate net profit after slippage and fees. Return JSON:
{{"profit_sol": float, "profit_percentage": float, "feasible": bool, "reasoning": str}}
""",
            
            "sandwich_calculation": """
Calculate sandwich attack parameters:
- Target transaction: {target_tx_size} SOL
- Pool liquidity: {pool_liquidity} SOL
- Current price: {current_price}
- Slippage tolerance: {slippage}%

Calculate optimal front-run and back-run sizes. Return JSON:
{{"front_run_size": float, "back_run_size": float, "expected_profit": float, "risk_score": float}}
""",
            
            "risk_assessment": """
Assess trading risk for:
- Strategy: {strategy}
- Token: {token}
- Position size: {position_size} SOL
- Market conditions: {market_conditions}
- Historical volatility: {volatility}
- Liquidity: {liquidity} SOL

Provide comprehensive risk analysis. Return JSON:
{{"risk_score": float, "risk_factors": [str], "max_loss": float, "recommended_action": str}}
"""
        }
    
    async def initialize(self) -> bool:
        """Initialize DeepSeek-Math model with optimizations"""
        try:
            logger.info("🧮 Initializing DeepSeek-Math model...")
            start_time = time.time()
            
            # Configure quantization for memory efficiency
            quantization_config = BitsAndBytesConfig(
                load_in_4bit=True,
                bnb_4bit_compute_dtype=torch.float16,
                bnb_4bit_use_double_quant=True,
                bnb_4bit_quant_type="nf4"
            )
            
            # Load tokenizer
            self.tokenizer = AutoTokenizer.from_pretrained(
                self.config.model_name,
                trust_remote_code=True,
                padding_side="left"
            )
            
            if self.tokenizer.pad_token is None:
                self.tokenizer.pad_token = self.tokenizer.eos_token
            
            # Load model with optimizations
            if self.config.use_quantization:
                self.model = AutoModelForCausalLM.from_pretrained(
                    self.config.model_name,
                    quantization_config=quantization_config,
                    device_map="auto",
                    trust_remote_code=True,
                    torch_dtype=torch.float16
                )
            else:
                self.model = AutoModelForCausalLM.from_pretrained(
                    self.config.model_name,
                    device_map="auto",
                    trust_remote_code=True,
                    torch_dtype=torch.float16
                )
            
            # Load LoRA adapter if specified
            if self.config.lora_adapter_path:
                logger.info(f"🔧 Loading LoRA adapter: {self.config.lora_adapter_path}")
                self.model = PeftModel.from_pretrained(
                    self.model, 
                    self.config.lora_adapter_path
                )
            
            # Initialize LMCache for performance
            if self.config.use_lmcache:
                self.lmcache = LMCache(
                    cache_size=self.config.cache_size_mb * 1024 * 1024,
                    ttl_seconds=self.config.cache_ttl_seconds
                )
                logger.info("🚀 LMCache initialized for faster inference")
            
            # Create pipeline
            self.pipeline = pipeline(
                "text-generation",
                model=self.model,
                tokenizer=self.tokenizer,
                max_new_tokens=self.config.max_tokens,
                temperature=self.config.temperature,
                do_sample=True,
                pad_token_id=self.tokenizer.eos_token_id
            )
            
            initialization_time = time.time() - start_time
            logger.info(f"✅ DeepSeek-Math initialized in {initialization_time:.2f}s")
            
            # Test inference
            await self._test_inference()
            
            return True
            
        except Exception as e:
            logger.error(f"❌ Failed to initialize DeepSeek-Math: {e}")
            return False
    
    async def _test_inference(self):
        """Test model inference with simple calculation"""
        try:
            test_prompt = "Calculate 15% of 8.5 SOL for position sizing:"
            result = await self._generate_response(test_prompt, max_tokens=50)
            logger.info(f"🧪 Test inference successful: {result[:100]}...")
        except Exception as e:
            logger.warning(f"⚠️ Test inference failed: {e}")
    
    async def calculate_position_size(
        self,
        capital: float,
        risk_tolerance: float,
        expected_return: float,
        volatility: float,
        strategy: str
    ) -> TradingCalculation:
        """Calculate optimal position size using Kelly Criterion"""
        start_time = time.time()
        
        try:
            prompt = self.prompts["position_sizing"].format(
                capital=capital,
                risk_tolerance=risk_tolerance,
                expected_return=expected_return,
                volatility=volatility,
                strategy=strategy
            )
            
            # Check cache first
            cache_key = f"position_size_{hash(prompt)}"
            if self.lmcache:
                cached_result = await self.lmcache.get(cache_key)
                if cached_result:
                    logger.info("📦 Position size calculation retrieved from cache")
                    return cached_result
            
            response = await self._generate_response(prompt)
            result = self._parse_json_response(response)
            
            calculation = TradingCalculation(
                calculation_type="position_sizing",
                input_params={
                    "capital": capital,
                    "risk_tolerance": risk_tolerance,
                    "expected_return": expected_return,
                    "volatility": volatility,
                    "strategy": strategy
                },
                result=result,
                confidence=min(0.95, 1.0 - volatility),  # Higher volatility = lower confidence
                reasoning=result.get("reasoning", "Kelly Criterion calculation"),
                execution_time_ms=int((time.time() - start_time) * 1000),
                model_used=self.config.model_name
            )
            
            # Cache result
            if self.lmcache:
                await self.lmcache.set(cache_key, calculation, ttl=3600)
            
            # Update metrics
            self.metrics.record_calculation("position_sizing", calculation.execution_time_ms)
            
            return calculation
            
        except Exception as e:
            logger.error(f"❌ Position size calculation failed: {e}")
            raise
    
    async def calculate_arbitrage_profit(
        self,
        token: str,
        price_a: float,
        price_b: float,
        liquidity_a: float,
        liquidity_b: float,
        gas_cost: float
    ) -> TradingCalculation:
        """Calculate arbitrage profit potential"""
        start_time = time.time()
        
        try:
            prompt = self.prompts["arbitrage_profit"].format(
                token=token,
                price_a=price_a,
                price_b=price_b,
                liquidity_a=liquidity_a,
                liquidity_b=liquidity_b,
                gas_cost=gas_cost
            )
            
            response = await self._generate_response(prompt)
            result = self._parse_json_response(response)
            
            calculation = TradingCalculation(
                calculation_type="arbitrage_profit",
                input_params={
                    "token": token,
                    "price_a": price_a,
                    "price_b": price_b,
                    "liquidity_a": liquidity_a,
                    "liquidity_b": liquidity_b,
                    "gas_cost": gas_cost
                },
                result=result,
                confidence=0.9 if result.get("feasible", False) else 0.3,
                reasoning=result.get("reasoning", "Arbitrage calculation with slippage"),
                execution_time_ms=int((time.time() - start_time) * 1000),
                model_used=self.config.model_name
            )
            
            self.metrics.record_calculation("arbitrage", calculation.execution_time_ms)
            return calculation
            
        except Exception as e:
            logger.error(f"❌ Arbitrage calculation failed: {e}")
            raise
    
    async def assess_trading_risk(
        self,
        strategy: str,
        token: str,
        position_size: float,
        market_conditions: Dict[str, Any],
        volatility: float,
        liquidity: float
    ) -> RiskAssessment:
        """Comprehensive risk assessment for trading decision"""
        start_time = time.time()
        
        try:
            prompt = self.prompts["risk_assessment"].format(
                strategy=strategy,
                token=token,
                position_size=position_size,
                market_conditions=json.dumps(market_conditions),
                volatility=volatility,
                liquidity=liquidity
            )
            
            response = await self._generate_response(prompt)
            result = self._parse_json_response(response)
            
            risk_assessment = RiskAssessment(
                risk_score=result.get("risk_score", 0.5),
                risk_factors=result.get("risk_factors", []),
                recommended_position_size=position_size * (1.0 - result.get("risk_score", 0.5)),
                max_loss_estimate=result.get("max_loss", position_size * 0.1),
                confidence=0.85,
                reasoning=result.get("recommended_action", "Risk assessment completed")
            )
            
            execution_time = int((time.time() - start_time) * 1000)
            self.metrics.record_calculation("risk_assessment", execution_time)
            
            return risk_assessment
            
        except Exception as e:
            logger.error(f"❌ Risk assessment failed: {e}")
            raise
    
    async def _generate_response(self, prompt: str, max_tokens: Optional[int] = None) -> str:
        """Generate response from model"""
        try:
            max_tokens = max_tokens or self.config.max_tokens
            
            # Add system prompt for trading context
            full_prompt = f"""You are a mathematical trading expert for Solana DeFi. 
Provide precise calculations and always return valid JSON responses.
Focus on risk management and realistic profit estimates.

{prompt}

Response:"""
            
            # Generate response
            if self.pipeline:
                outputs = self.pipeline(
                    full_prompt,
                    max_new_tokens=max_tokens,
                    temperature=self.config.temperature,
                    pad_token_id=self.tokenizer.eos_token_id,
                    return_full_text=False
                )
                response = outputs[0]["generated_text"].strip()
            else:
                # Fallback to direct model inference
                inputs = self.tokenizer.encode(full_prompt, return_tensors="pt")
                with torch.no_grad():
                    outputs = self.model.generate(
                        inputs,
                        max_new_tokens=max_tokens,
                        temperature=self.config.temperature,
                        do_sample=True,
                        pad_token_id=self.tokenizer.eos_token_id
                    )
                response = self.tokenizer.decode(outputs[0][inputs.shape[1]:], skip_special_tokens=True)
            
            return response.strip()
            
        except Exception as e:
            logger.error(f"❌ Response generation failed: {e}")
            raise
    
    def _parse_json_response(self, response: str) -> Dict[str, Any]:
        """Parse JSON response from model"""
        try:
            # Try to find JSON in response
            start_idx = response.find('{')
            end_idx = response.rfind('}') + 1
            
            if start_idx != -1 and end_idx != -1:
                json_str = response[start_idx:end_idx]
                return json.loads(json_str)
            else:
                # Fallback: create structured response
                return {
                    "result": response,
                    "reasoning": "Parsed from text response",
                    "confidence": 0.7
                }
                
        except json.JSONDecodeError as e:
            logger.warning(f"⚠️ JSON parsing failed: {e}, using fallback")
            return {
                "result": response,
                "reasoning": "Failed to parse JSON, using raw response",
                "confidence": 0.5
            }
    
    async def get_metrics(self) -> Dict[str, Any]:
        """Get performance metrics"""
        return {
            "model_name": self.config.model_name,
            "calculations_performed": self.metrics.total_calculations,
            "average_latency_ms": self.metrics.average_latency,
            "cache_hit_ratio": self.lmcache.hit_ratio if self.lmcache else 0.0,
            "memory_usage_mb": torch.cuda.memory_allocated() / 1024 / 1024 if torch.cuda.is_available() else 0
        }
    
    async def cleanup(self):
        """Cleanup resources"""
        try:
            if self.model:
                del self.model
            if self.tokenizer:
                del self.tokenizer
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            logger.info("🧹 DeepSeek-Math cleanup completed")
        except Exception as e:
            logger.error(f"❌ Cleanup failed: {e}")
