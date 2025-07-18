"""
⚙️ AI Configuration for Solana HFT Ninja
Optimized for cost-effective small portfolio trading with DeepSeek-Math
"""

from dataclasses import dataclass, field
from typing import Optional, Dict, Any
import os

@dataclass
class DeepSeekConfig:
    """DeepSeek-Math model configuration - Cost-effective AI for small portfolios"""
    model_name: str = "deepseek-ai/deepseek-math-7b-instruct"
    use_quantization: bool = True  # 4-bit quantization for memory efficiency
    use_lmcache: bool = True      # Smart caching for cost reduction
    cache_size_mb: int = 1024     # 1GB cache for frequent calculations
    cache_ttl_seconds: int = 3600 # 1 hour cache TTL
    max_tokens: int = 512         # Optimized for trading calculations
    temperature: float = 0.1      # Low temperature for consistent math
    lora_adapter_path: Optional[str] = None  # Path to fine-tuned LoRA adapter
    device_map: str = "auto"
    torch_dtype: str = "float16"
    
    # API settings
    api_host: str = "0.0.0.0"
    api_port: int = 8003
    workers: int = 1
    
    # Performance settings
    max_concurrent_requests: int = 5
    request_timeout_seconds: int = 30
    
    # Cost optimization
    enable_batching: bool = True
    batch_size: int = 4
    enable_cpu_offload: bool = True
    
    @classmethod
    def from_env(cls) -> 'DeepSeekConfig':
        """Create config from environment variables"""
        return cls(
            model_name=os.getenv("DEEPSEEK_MODEL_NAME", cls.model_name),
            use_quantization=os.getenv("USE_QUANTIZATION", "true").lower() == "true",
            use_lmcache=os.getenv("USE_LMCACHE", "true").lower() == "true",
            cache_size_mb=int(os.getenv("CACHE_SIZE_MB", "1024")),
            max_tokens=int(os.getenv("MAX_TOKENS", "512")),
            temperature=float(os.getenv("TEMPERATURE", "0.1")),
            lora_adapter_path=os.getenv("LORA_ADAPTER_PATH"),
            api_port=int(os.getenv("API_PORT", "8003"))
        )

@dataclass
class OumiConfig:
    """OUMI AI configuration"""
    enabled: bool = True
    api_url: str = "http://localhost:8001"
    model_name: str = "oumi/solana-trading-v1"
    max_tokens: int = 1000
    temperature: float = 0.3
    confidence_threshold: float = 0.7

@dataclass
class OpenSearchConfig:
    """OpenSearch AI configuration"""
    enabled: bool = True
    host: str = "localhost"
    port: int = 9200
    index_name: str = "trading_patterns"
    embedding_model: str = "sentence-transformers/all-MiniLM-L6-v2"
    max_results: int = 10

@dataclass
class LMCacheConfig:
    """LMCache configuration"""
    enabled: bool = True
    cache_type: str = "memory"  # memory, redis, disk
    max_size_mb: int = 2048
    ttl_seconds: int = 3600
    eviction_policy: str = "LRU"
    distributed_cache: bool = False
    redis_url: Optional[str] = None

@dataclass
class AIConfig:
    """Main AI configuration"""
    enabled: bool = True
    
    # Component toggles
    oumi_enabled: bool = True
    opensearch_enabled: bool = True
    lmcache_enabled: bool = True
    deepseek_enabled: bool = True
    
    # Default model settings
    default_model: str = "deepseek-math"  # Use cost-effective model by default
    temperature: float = 0.1
    max_tokens: int = 512
    
    # API settings
    openai_api_key: Optional[str] = None
    anthropic_api_key: Optional[str] = None
    huggingface_token: Optional[str] = None
    
    # Performance settings
    cache_ttl_seconds: int = 3600
    max_concurrent_requests: int = 10
    request_timeout_seconds: int = 30
    
    # Component configurations
    deepseek: DeepSeekConfig = field(default_factory=DeepSeekConfig)
    oumi: OumiConfig = field(default_factory=OumiConfig)
    opensearch: OpenSearchConfig = field(default_factory=OpenSearchConfig)
    lmcache: LMCacheConfig = field(default_factory=LMCacheConfig)
    
    # Cost optimization settings
    cost_optimization: Dict[str, Any] = field(default_factory=lambda: {
        "max_daily_cost_usd": 1.0,  # Maximum $1 per day
        "prefer_cached_results": True,
        "batch_requests": True,
        "use_quantized_models": True,
        "enable_early_stopping": True
    })
    
    # Trading-specific AI settings
    trading_ai: Dict[str, Any] = field(default_factory=lambda: {
        "position_sizing_model": "deepseek-math",
        "risk_assessment_model": "deepseek-math",
        "arbitrage_calculation_model": "deepseek-math",
        "sentiment_analysis_model": "oumi",
        "pattern_recognition_model": "opensearch",
        "confidence_threshold": 0.7,
        "max_calculation_time_ms": 1000
    })
    
    @classmethod
    def from_env(cls) -> 'AIConfig':
        """Create configuration from environment variables"""
        return cls(
            enabled=os.getenv("AI_ENABLED", "true").lower() == "true",
            deepseek_enabled=os.getenv("DEEPSEEK_ENABLED", "true").lower() == "true",
            oumi_enabled=os.getenv("OUMI_ENABLED", "true").lower() == "true",
            opensearch_enabled=os.getenv("OPENSEARCH_ENABLED", "true").lower() == "true",
            lmcache_enabled=os.getenv("LMCACHE_ENABLED", "true").lower() == "true",
            
            openai_api_key=os.getenv("OPENAI_API_KEY"),
            anthropic_api_key=os.getenv("ANTHROPIC_API_KEY"),
            huggingface_token=os.getenv("HUGGINGFACE_TOKEN"),
            
            deepseek=DeepSeekConfig.from_env(),
            
            cost_optimization={
                "max_daily_cost_usd": float(os.getenv("MAX_DAILY_AI_COST", "1.0")),
                "prefer_cached_results": os.getenv("PREFER_CACHE", "true").lower() == "true",
                "batch_requests": os.getenv("BATCH_REQUESTS", "true").lower() == "true",
                "use_quantized_models": os.getenv("USE_QUANTIZATION", "true").lower() == "true",
                "enable_early_stopping": os.getenv("EARLY_STOPPING", "true").lower() == "true"
            }
        )
    
    def get_model_config(self, model_name: str) -> Dict[str, Any]:
        """Get configuration for specific model"""
        if model_name == "deepseek-math":
            return {
                "config": self.deepseek,
                "cost_per_token": 0.000001,  # Very low cost
                "latency_ms": 200,
                "accuracy": 0.94
            }
        elif model_name == "oumi":
            return {
                "config": self.oumi,
                "cost_per_token": 0.000002,
                "latency_ms": 150,
                "accuracy": 0.92
            }
        else:
            return {
                "cost_per_token": 0.00001,  # Default higher cost
                "latency_ms": 500,
                "accuracy": 0.85
            }
    
    def is_cost_effective(self, daily_usage_usd: float) -> bool:
        """Check if current usage is within cost limits"""
        return daily_usage_usd <= self.cost_optimization["max_daily_cost_usd"]
    
    def get_recommended_model(self, task_type: str) -> str:
        """Get recommended model for specific task type"""
        model_mapping = {
            "position_sizing": "deepseek-math",
            "risk_assessment": "deepseek-math", 
            "arbitrage_calculation": "deepseek-math",
            "sandwich_calculation": "deepseek-math",
            "sentiment_analysis": "oumi",
            "pattern_recognition": "opensearch",
            "market_analysis": "oumi"
        }
        
        return model_mapping.get(task_type, self.default_model)

# Global configuration instance
ai_config = AIConfig.from_env()

# Export commonly used configs
deepseek_config = ai_config.deepseek
oumi_config = ai_config.oumi
opensearch_config = ai_config.opensearch
lmcache_config = ai_config.lmcache
