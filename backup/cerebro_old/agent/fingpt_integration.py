#!/usr/bin/env python3
"""
FinGPT Integration for Project Cerebro
Integrates AI4Finance-Foundation/FinGPT models for specialized financial analysis
"""

import torch
import asyncio
import json
import logging
from typing import Dict, Any, List, Optional, Union
from datetime import datetime
from transformers import (
    AutoTokenizer, 
    AutoModelForCausalLM, 
    BitsAndBytesConfig,
    pipeline
)
from peft import PeftModel, PeftConfig
import numpy as np

logger = logging.getLogger(__name__)

class FinGPTModel:
    """
    FinGPT model wrapper for financial analysis tasks
    Supports multiple FinGPT variants: sentiment analysis, forecasting, multi-task
    """
    
    def __init__(self, model_name: str = "FinGPT/fingpt-sentiment_llama2-13b_lora", device: str = "auto"):
        self.model_name = model_name
        self.device = self._get_device(device)
        self.tokenizer = None
        self.model = None
        self.pipeline = None
        self.model_type = self._determine_model_type(model_name)
        
        # Model configuration
        self.max_length = 2048
        self.temperature = 0.7
        self.top_p = 0.9
        self.do_sample = True
        
        logger.info(f"Initializing FinGPT model: {model_name}")
        logger.info(f"Model type: {self.model_type}")
        logger.info(f"Device: {self.device}")
    
    def _get_device(self, device: str) -> str:
        """Determine the best device for model inference"""
        if device == "auto":
            if torch.cuda.is_available():
                return "cuda"
            elif torch.backends.mps.is_available():
                return "mps"
            else:
                return "cpu"
        return device
    
    def _determine_model_type(self, model_name: str) -> str:
        """Determine the type of FinGPT model based on name"""
        if "sentiment" in model_name.lower():
            return "sentiment_analysis"
        elif "forecaster" in model_name.lower():
            return "forecasting"
        elif "mt" in model_name.lower():
            return "multi_task"
        else:
            return "general"
    
    async def initialize(self):
        """Initialize the FinGPT model and tokenizer"""
        try:
            logger.info("Loading FinGPT model and tokenizer...")
            
            # Configure quantization for memory efficiency
            quantization_config = BitsAndBytesConfig(
                load_in_4bit=True,
                bnb_4bit_compute_dtype=torch.float16,
                bnb_4bit_use_double_quant=True,
                bnb_4bit_quant_type="nf4"
            )
            
            # Load tokenizer
            self.tokenizer = AutoTokenizer.from_pretrained(
                self.model_name,
                trust_remote_code=True,
                padding_side="left"
            )
            
            # Add pad token if not present
            if self.tokenizer.pad_token is None:
                self.tokenizer.pad_token = self.tokenizer.eos_token
            
            # Load base model with quantization
            base_model_name = self._get_base_model_name()
            self.model = AutoModelForCausalLM.from_pretrained(
                base_model_name,
                quantization_config=quantization_config,
                device_map="auto",
                trust_remote_code=True,
                torch_dtype=torch.float16
            )
            
            # Load LoRA adapter if it's a FinGPT fine-tuned model
            if "FinGPT/" in self.model_name:
                try:
                    self.model = PeftModel.from_pretrained(
                        self.model,
                        self.model_name,
                        torch_dtype=torch.float16
                    )
                    logger.info("✅ LoRA adapter loaded successfully")
                except Exception as e:
                    logger.warning(f"Could not load LoRA adapter: {e}")
            
            # Create text generation pipeline
            self.pipeline = pipeline(
                "text-generation",
                model=self.model,
                tokenizer=self.tokenizer,
                device_map="auto",
                torch_dtype=torch.float16,
                trust_remote_code=True
            )
            
            logger.info("✅ FinGPT model initialized successfully")
            
        except Exception as e:
            logger.error(f"Failed to initialize FinGPT model: {e}")
            raise
    
    def _get_base_model_name(self) -> str:
        """Get the base model name for FinGPT variants"""
        if "llama2-13b" in self.model_name:
            return "meta-llama/Llama-2-13b-hf"
        elif "llama2-7b" in self.model_name:
            return "meta-llama/Llama-2-7b-hf"
        elif "chatglm2-6b" in self.model_name:
            return "THUDM/chatglm2-6b"
        elif "falcon-7b" in self.model_name:
            return "tiiuae/falcon-7b"
        elif "bloom-7b1" in self.model_name:
            return "bigscience/bloom-7b1"
        elif "mpt-7b" in self.model_name:
            return "mosaicml/mpt-7b"
        elif "qwen-7b" in self.model_name:
            return "Qwen/Qwen-7B-Chat"
        else:
            # Default to Llama2-7b for unknown variants
            return "meta-llama/Llama-2-7b-hf"
    
    async def analyze_sentiment(self, text: str) -> Dict[str, Any]:
        """
        Analyze financial sentiment using FinGPT
        
        Args:
            text: Financial text to analyze
            
        Returns:
            Dict containing sentiment analysis results
        """
        try:
            # Create sentiment analysis prompt
            prompt = self._create_sentiment_prompt(text)
            
            # Generate response
            response = await self._generate_response(prompt)
            
            # Parse sentiment from response
            sentiment_result = self._parse_sentiment_response(response)
            
            return {
                "text": text,
                "sentiment": sentiment_result["sentiment"],
                "confidence": sentiment_result["confidence"],
                "reasoning": sentiment_result["reasoning"],
                "model_used": self.model_name,
                "timestamp": datetime.now().isoformat()
            }
            
        except Exception as e:
            logger.error(f"Sentiment analysis failed: {e}")
            return {
                "text": text,
                "sentiment": "neutral",
                "confidence": 0.0,
                "reasoning": f"Analysis failed: {str(e)}",
                "model_used": self.model_name,
                "timestamp": datetime.now().isoformat()
            }
    
    async def forecast_price(self, ticker: str, context: Dict[str, Any]) -> Dict[str, Any]:
        """
        Generate price forecast using FinGPT-Forecaster
        
        Args:
            ticker: Stock ticker symbol
            context: Market context and historical data
            
        Returns:
            Dict containing price forecast
        """
        try:
            # Create forecasting prompt
            prompt = self._create_forecasting_prompt(ticker, context)
            
            # Generate response
            response = await self._generate_response(prompt)
            
            # Parse forecast from response
            forecast_result = self._parse_forecast_response(response)
            
            return {
                "ticker": ticker,
                "forecast": forecast_result["direction"],
                "confidence": forecast_result["confidence"],
                "reasoning": forecast_result["reasoning"],
                "timeframe": "1_week",
                "model_used": self.model_name,
                "timestamp": datetime.now().isoformat()
            }
            
        except Exception as e:
            logger.error(f"Price forecasting failed: {e}")
            return {
                "ticker": ticker,
                "forecast": "neutral",
                "confidence": 0.0,
                "reasoning": f"Forecast failed: {str(e)}",
                "model_used": self.model_name,
                "timestamp": datetime.now().isoformat()
            }
    
    async def analyze_financial_text(self, text: str, task: str = "general") -> Dict[str, Any]:
        """
        General financial text analysis using multi-task FinGPT
        
        Args:
            text: Financial text to analyze
            task: Specific task (sentiment, ner, relation_extraction, etc.)
            
        Returns:
            Dict containing analysis results
        """
        try:
            # Create task-specific prompt
            prompt = self._create_multitask_prompt(text, task)
            
            # Generate response
            response = await self._generate_response(prompt)
            
            # Parse response based on task
            analysis_result = self._parse_multitask_response(response, task)
            
            return {
                "text": text,
                "task": task,
                "result": analysis_result,
                "model_used": self.model_name,
                "timestamp": datetime.now().isoformat()
            }
            
        except Exception as e:
            logger.error(f"Financial text analysis failed: {e}")
            return {
                "text": text,
                "task": task,
                "result": f"Analysis failed: {str(e)}",
                "model_used": self.model_name,
                "timestamp": datetime.now().isoformat()
            }
    
    def _create_sentiment_prompt(self, text: str) -> str:
        """Create prompt for sentiment analysis"""
        return f"""Instruction: What is the sentiment of this news? Please choose an answer from {{negative/neutral/positive}}.

Input: {text}

Output:"""
    
    def _create_forecasting_prompt(self, ticker: str, context: Dict[str, Any]) -> str:
        """Create prompt for price forecasting"""
        market_data = context.get("market_data", {})
        news_summary = context.get("news_summary", "No recent news available")
        
        return f"""Instruction: Based on the following information, predict the stock price movement for {ticker} in the next week. Choose from {{up/down/stable}} and provide reasoning.

Market Data:
- Current Price: ${market_data.get('current_price', 'N/A')}
- 24h Change: {market_data.get('price_change_24h', 'N/A')}%
- Volume: {market_data.get('volume', 'N/A')}

Recent News Summary:
{news_summary}

Prediction:"""
    
    def _create_multitask_prompt(self, text: str, task: str) -> str:
        """Create prompt for multi-task analysis"""
        task_instructions = {
            "sentiment": "What is the sentiment of this news? Please choose an answer from {negative/neutral/positive}.",
            "ner": "Please extract entities and their types from the input sentence, entity types should be chosen from {person/organization/location}.",
            "relation_extraction": "Extract the word/phrase pair and the corresponding lexical relationship between them from the input text.",
            "headline_classification": "Does the news headline talk about price going up? Please choose an answer from {Yes/No}.",
            "general": "Analyze this financial text and provide insights."
        }
        
        instruction = task_instructions.get(task, task_instructions["general"])
        
        return f"""Instruction: {instruction}

Input: {text}

Output:"""
    
    async def _generate_response(self, prompt: str) -> str:
        """Generate response using the FinGPT model"""
        try:
            # Tokenize input
            inputs = self.tokenizer(
                prompt,
                return_tensors="pt",
                truncation=True,
                max_length=self.max_length - 512,  # Leave room for generation
                padding=True
            )
            
            # Move to device
            inputs = {k: v.to(self.device) for k, v in inputs.items()}
            
            # Generate response
            with torch.no_grad():
                outputs = self.model.generate(
                    **inputs,
                    max_new_tokens=512,
                    temperature=self.temperature,
                    top_p=self.top_p,
                    do_sample=self.do_sample,
                    pad_token_id=self.tokenizer.eos_token_id,
                    eos_token_id=self.tokenizer.eos_token_id
                )
            
            # Decode response
            response = self.tokenizer.decode(
                outputs[0][inputs["input_ids"].shape[1]:],
                skip_special_tokens=True
            ).strip()
            
            return response
            
        except Exception as e:
            logger.error(f"Response generation failed: {e}")
            return f"Generation failed: {str(e)}"
    
    def _parse_sentiment_response(self, response: str) -> Dict[str, Any]:
        """Parse sentiment analysis response"""
        response_lower = response.lower().strip()
        
        # Extract sentiment
        if "positive" in response_lower:
            sentiment = "positive"
            confidence = 0.8
        elif "negative" in response_lower:
            sentiment = "negative"
            confidence = 0.8
        elif "neutral" in response_lower:
            sentiment = "neutral"
            confidence = 0.7
        else:
            sentiment = "neutral"
            confidence = 0.5
        
        return {
            "sentiment": sentiment,
            "confidence": confidence,
            "reasoning": response
        }
    
    def _parse_forecast_response(self, response: str) -> Dict[str, Any]:
        """Parse price forecast response"""
        response_lower = response.lower().strip()
        
        # Extract direction
        if "up" in response_lower or "increase" in response_lower or "bullish" in response_lower:
            direction = "up"
            confidence = 0.7
        elif "down" in response_lower or "decrease" in response_lower or "bearish" in response_lower:
            direction = "down"
            confidence = 0.7
        elif "stable" in response_lower or "sideways" in response_lower:
            direction = "stable"
            confidence = 0.6
        else:
            direction = "neutral"
            confidence = 0.5
        
        return {
            "direction": direction,
            "confidence": confidence,
            "reasoning": response
        }
    
    def _parse_multitask_response(self, response: str, task: str) -> str:
        """Parse multi-task analysis response"""
        # For now, return the raw response
        # In production, implement task-specific parsing
        return response.strip()
    
    async def close(self):
        """Cleanup model resources"""
        try:
            if self.model is not None:
                del self.model
            if self.tokenizer is not None:
                del self.tokenizer
            if self.pipeline is not None:
                del self.pipeline
            
            # Clear CUDA cache if using GPU
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            
            logger.info("✅ FinGPT model resources cleaned up")
            
        except Exception as e:
            logger.error(f"Error during cleanup: {e}")

class FinGPTManager:
    """
    Manager for multiple FinGPT models
    Handles model selection and routing based on task requirements
    """
    
    def __init__(self):
        self.models = {}
        self.default_models = {
            "sentiment_analysis": "FinGPT/fingpt-sentiment_llama2-13b_lora",
            "forecasting": "FinGPT/fingpt-forecaster_dow30_llama2-7b_lora",
            "multi_task": "FinGPT/fingpt-mt_llama2-7b_lora"
        }
        
    async def initialize(self, models_to_load: List[str] = None):
        """Initialize specified FinGPT models"""
        if models_to_load is None:
            models_to_load = ["sentiment_analysis"]  # Load only sentiment by default
        
        for model_type in models_to_load:
            if model_type in self.default_models:
                try:
                    model = FinGPTModel(self.default_models[model_type])
                    await model.initialize()
                    self.models[model_type] = model
                    logger.info(f"✅ Loaded FinGPT model: {model_type}")
                except Exception as e:
                    logger.error(f"Failed to load FinGPT model {model_type}: {e}")
    
    async def analyze_sentiment(self, text: str) -> Dict[str, Any]:
        """Analyze sentiment using FinGPT sentiment model"""
        if "sentiment_analysis" in self.models:
            return await self.models["sentiment_analysis"].analyze_sentiment(text)
        else:
            logger.warning("Sentiment analysis model not loaded")
            return {
                "text": text,
                "sentiment": "neutral",
                "confidence": 0.0,
                "reasoning": "Sentiment model not available",
                "model_used": "none",
                "timestamp": datetime.now().isoformat()
            }
    
    async def forecast_price(self, ticker: str, context: Dict[str, Any]) -> Dict[str, Any]:
        """Generate price forecast using FinGPT forecaster"""
        if "forecasting" in self.models:
            return await self.models["forecasting"].forecast_price(ticker, context)
        else:
            logger.warning("Forecasting model not loaded")
            return {
                "ticker": ticker,
                "forecast": "neutral",
                "confidence": 0.0,
                "reasoning": "Forecasting model not available",
                "model_used": "none",
                "timestamp": datetime.now().isoformat()
            }
    
    async def analyze_financial_text(self, text: str, task: str = "general") -> Dict[str, Any]:
        """Analyze financial text using multi-task FinGPT"""
        if "multi_task" in self.models:
            return await self.models["multi_task"].analyze_financial_text(text, task)
        else:
            logger.warning("Multi-task model not loaded")
            return {
                "text": text,
                "task": task,
                "result": "Multi-task model not available",
                "model_used": "none",
                "timestamp": datetime.now().isoformat()
            }
    
    async def close(self):
        """Close all loaded models"""
        for model in self.models.values():
            await model.close()
        self.models.clear()
        logger.info("✅ All FinGPT models closed")

# Factory function for easy integration
async def create_fingpt_manager(models_to_load: List[str] = None) -> FinGPTManager:
    """Factory function to create and initialize FinGPT manager"""
    manager = FinGPTManager()
    await manager.initialize(models_to_load)
    return manager
