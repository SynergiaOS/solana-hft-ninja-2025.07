#!/usr/bin/env python3
"""
Cerebro Memory Schema
Data structures for context storage in DragonflyDB
"""

import json
import time
import uuid
from typing import Dict, List, Any, Optional, Union
from dataclasses import dataclass, asdict
from datetime import datetime
from enum import Enum
import numpy as np


class ContextType(Enum):
    """Types of context stored in memory"""
    PERFORMANCE_INSIGHT = "performance_insight"
    USER_DIRECTIVE = "user_directive"
    MARKET_EVENT = "market_event"
    STRATEGY_ANALYSIS = "strategy_analysis"
    ERROR_ANALYSIS = "error_analysis"
    OPTIMIZATION_SUGGESTION = "optimization_suggestion"
    TRADING_DECISION = "trading_decision"
    RISK_ASSESSMENT = "risk_assessment"


class ContextSource(Enum):
    """Sources of context information"""
    USER_CHAT = "user_chat"
    PROMETHEUS_ANALYSIS = "prometheus_analysis"
    HFT_LOGS = "hft_logs"
    MARKET_DATA = "market_data"
    STRATEGY_ENGINE = "strategy_engine"
    RISK_MANAGER = "risk_manager"
    EXTERNAL_API = "external_api"
    CEREBRO_ANALYSIS = "cerebro_analysis"


@dataclass
class ContextEntry:
    """Single context entry for memory storage"""

    # Core content
    content: str
    vector: List[float]

    # Metadata
    context_type: ContextType
    source: ContextSource
    timestamp: float

    # Optional fields
    context_id: Optional[str] = None
    confidence: Optional[float] = None
    tags: Optional[Dict[str, Any]] = None
    related_strategy: Optional[str] = None
    profit_impact: Optional[float] = None

    def __post_init__(self):
        """Initialize computed fields"""
        if self.context_id is None:
            self.context_id = str(uuid.uuid4())

        if self.tags is None:
            self.tags = {}

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for storage"""
        data = asdict(self)
        # Convert enums to strings
        data['context_type'] = self.context_type.value
        data['source'] = self.source.value
        return data

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'ContextEntry':
        """Create from dictionary"""
        # Convert string enums back
        data['context_type'] = ContextType(data['context_type'])
        data['source'] = ContextSource(data['source'])
        return cls(**data)

    def to_json(self) -> str:
        """Convert to JSON string"""
        return json.dumps(self.to_dict())

    @classmethod
    def from_json(cls, json_str: str) -> 'ContextEntry':
        """Create from JSON string"""
        data = json.loads(json_str)
        return cls.from_dict(data)


class MemorySchema:
    """Schema manager for DragonflyDB storage"""

    # Key prefixes
    CONTEXT_PREFIX = "cerebro:context:"
    INDEX_PREFIX = "cerebro:index:"
    METADATA_PREFIX = "cerebro:meta:"

    @staticmethod
    def context_key(context_id: str) -> str:
        """Generate key for context entry"""
        return f"{MemorySchema.CONTEXT_PREFIX}{context_id}"

    @staticmethod
    def type_index_key(context_type: ContextType) -> str:
        """Generate key for type index"""
        return f"{MemorySchema.INDEX_PREFIX}type:{context_type.value}"

    @staticmethod
    def source_index_key(source: ContextSource) -> str:
        """Generate key for source index"""
        return f"{MemorySchema.INDEX_PREFIX}source:{source.value}"

    @staticmethod
    def strategy_index_key(strategy_name: str) -> str:
        """Generate key for strategy index"""
        return f"{MemorySchema.INDEX_PREFIX}strategy:{strategy_name}"

    @staticmethod
    def time_index_key(date_str: str) -> str:
        """Generate key for time-based index (YYYY-MM-DD)"""
        return f"{MemorySchema.INDEX_PREFIX}time:{date_str}"

    @staticmethod
    def metadata_key(key: str) -> str:
        """Generate key for metadata"""
        return f"{MemorySchema.METADATA_PREFIX}{key}"


@dataclass
class SearchQuery:
    """Query structure for memory search"""

    query_text: str
    context_types: Optional[List[ContextType]] = None
    sources: Optional[List[ContextSource]] = None
    time_range: Optional[tuple] = None  # (start_timestamp, end_timestamp)
    strategy_filter: Optional[str] = None
    min_confidence: Optional[float] = None
    max_results: int = 10
    similarity_threshold: float = 0.7

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary"""
        data = {
            'query_text': self.query_text,
            'max_results': self.max_results,
            'similarity_threshold': self.similarity_threshold
        }

        if self.context_types:
            data['context_types'] = [ct.value for ct in self.context_types]

        if self.sources:
            data['sources'] = [s.value for s in self.sources]

        if self.time_range:
            data['time_range'] = self.time_range

        if self.strategy_filter:
            data['strategy_filter'] = self.strategy_filter

        if self.min_confidence:
            data['min_confidence'] = self.min_confidence

        return data


@dataclass
class SearchResult:
    """Result from memory search"""

    context_entry: ContextEntry
    similarity_score: float
    rank: int

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary"""
        return {
            'context_entry': self.context_entry.to_dict(),
            'similarity_score': self.similarity_score,
            'rank': self.rank
        }