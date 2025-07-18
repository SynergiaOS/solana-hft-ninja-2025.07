#!/usr/bin/env python3
"""
Unit Tests for Cerebro Memory System
Tests embedding, storage, and retrieval functionality
"""

import pytest
import asyncio
import json
import numpy as np
from unittest.mock import Mock, AsyncMock, patch
from datetime import datetime, timedelta

# Import the modules to test
import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from memory.memory_manager import MemoryManager
from memory.embedding_service import EmbeddingService
from core.config import CerebroConfig

class TestEmbeddingService:
    """Test the embedding service functionality"""
    
    @pytest.fixture
    def embedding_service(self):
        """Create a mock embedding service for testing"""
        config = CerebroConfig()
        config.jina.model_name = "jina-embeddings-v2-base-en"
        return EmbeddingService(config)
    
    def test_embedding_service_initialization(self, embedding_service):
        """Test that embedding service initializes correctly"""
        assert embedding_service.config is not None
        assert embedding_service.model_name == "jina-embeddings-v2-base-en"
    
    @pytest.mark.asyncio
    async def test_create_embedding(self, embedding_service):
        """Test embedding creation"""
        # Mock the actual embedding call
        with patch.object(embedding_service, '_call_jina_api') as mock_api:
            mock_api.return_value = [0.1, 0.2, 0.3, 0.4, 0.5] * 154  # 770 dimensions
            
            text = "Test trading strategy analysis"
            embedding = await embedding_service.create_embedding(text)
            
            assert isinstance(embedding, list)
            assert len(embedding) == 770  # Jina v2 base model dimension
            assert all(isinstance(x, float) for x in embedding)
            mock_api.assert_called_once_with(text)
    
    @pytest.mark.asyncio
    async def test_batch_embeddings(self, embedding_service):
        """Test batch embedding creation"""
        with patch.object(embedding_service, '_call_jina_api') as mock_api:
            mock_api.return_value = [[0.1] * 770, [0.2] * 770, [0.3] * 770]
            
            texts = [
                "Sandwich strategy performance",
                "Arbitrage opportunity detected", 
                "Market sentiment analysis"
            ]
            
            embeddings = await embedding_service.create_embeddings_batch(texts)
            
            assert len(embeddings) == 3
            assert all(len(emb) == 770 for emb in embeddings)
            mock_api.assert_called_once_with(texts)
    
    def test_cosine_similarity(self, embedding_service):
        """Test cosine similarity calculation"""
        vec1 = [1.0, 0.0, 0.0]
        vec2 = [0.0, 1.0, 0.0]
        vec3 = [1.0, 0.0, 0.0]
        
        # Orthogonal vectors should have similarity 0
        similarity_orthogonal = embedding_service.cosine_similarity(vec1, vec2)
        assert abs(similarity_orthogonal) < 1e-10
        
        # Identical vectors should have similarity 1
        similarity_identical = embedding_service.cosine_similarity(vec1, vec3)
        assert abs(similarity_identical - 1.0) < 1e-10
    
    @pytest.mark.asyncio
    async def test_embedding_error_handling(self, embedding_service):
        """Test error handling in embedding creation"""
        with patch.object(embedding_service, '_call_jina_api') as mock_api:
            mock_api.side_effect = Exception("API Error")
            
            with pytest.raises(Exception):
                await embedding_service.create_embedding("test text")

class TestMemoryManager:
    """Test the memory manager functionality"""
    
    @pytest.fixture
    def mock_redis(self):
        """Create a mock Redis client"""
        redis_mock = Mock()
        redis_mock.ping.return_value = True
        redis_mock.set.return_value = True
        redis_mock.get.return_value = None
        redis_mock.keys.return_value = []
        redis_mock.hset.return_value = True
        redis_mock.hgetall.return_value = {}
        redis_mock.zadd.return_value = True
        redis_mock.zrange.return_value = []
        return redis_mock
    
    @pytest.fixture
    def mock_embedding_service(self):
        """Create a mock embedding service"""
        service = Mock()
        service.create_embedding = AsyncMock(return_value=[0.1] * 770)
        service.cosine_similarity = Mock(return_value=0.8)
        return service
    
    @pytest.fixture
    def memory_manager(self, mock_redis, mock_embedding_service):
        """Create a memory manager with mocked dependencies"""
        config = CerebroConfig()
        manager = MemoryManager(config)
        manager.redis_client = mock_redis
        manager.embedding_service = mock_embedding_service
        return manager
    
    @pytest.mark.asyncio
    async def test_store_context(self, memory_manager, mock_redis, mock_embedding_service):
        """Test storing context in memory"""
        content = "Successful arbitrage trade executed with 0.05 SOL profit"
        context_type = "trading_success"
        metadata = {"strategy": "arbitrage", "profit": 0.05}
        
        result = await memory_manager.store_context(content, context_type, metadata)
        
        # Verify embedding was created
        mock_embedding_service.create_embedding.assert_called_once_with(content)
        
        # Verify Redis operations
        assert mock_redis.set.called
        assert mock_redis.hset.called
        assert mock_redis.zadd.called
        
        # Verify result
        assert result["success"] is True
        assert "memory_id" in result
        assert result["content"] == content
    
    @pytest.mark.asyncio
    async def test_search_relevant_context(self, memory_manager, mock_redis, mock_embedding_service):
        """Test searching for relevant context"""
        # Mock stored memories
        mock_redis.keys.return_value = ["cerebro:memory:1", "cerebro:memory:2"]
        mock_redis.hgetall.side_effect = [
            {
                "content": "Arbitrage strategy working well",
                "context_type": "strategy_analysis",
                "embedding": json.dumps([0.1] * 770),
                "timestamp": datetime.now().isoformat(),
                "metadata": json.dumps({"strategy": "arbitrage"})
            },
            {
                "content": "Market sentiment is bullish",
                "context_type": "market_analysis", 
                "embedding": json.dumps([0.2] * 770),
                "timestamp": datetime.now().isoformat(),
                "metadata": json.dumps({"sentiment": "bullish"})
            }
        ]
        
        # Mock similarity scores
        mock_embedding_service.cosine_similarity.side_effect = [0.9, 0.3]
        
        query = "How is arbitrage performing?"
        results = await memory_manager.search_relevant_context(query, limit=5, threshold=0.5)
        
        # Verify embedding was created for query
        mock_embedding_service.create_embedding.assert_called_with(query)
        
        # Verify results
        assert len(results) == 1  # Only one above threshold
        assert results[0]["content"] == "Arbitrage strategy working well"
        assert results[0]["similarity_score"] == 0.9
    
    @pytest.mark.asyncio
    async def test_get_context_by_type(self, memory_manager, mock_redis):
        """Test retrieving context by type"""
        mock_redis.keys.return_value = ["cerebro:memory:1", "cerebro:memory:2"]
        mock_redis.hgetall.side_effect = [
            {
                "content": "Strategy analysis result",
                "context_type": "strategy_analysis",
                "timestamp": datetime.now().isoformat()
            },
            {
                "content": "Market analysis result",
                "context_type": "market_analysis",
                "timestamp": datetime.now().isoformat()
            }
        ]
        
        results = await memory_manager.get_context_by_type("strategy_analysis", limit=10)
        
        assert len(results) == 1
        assert results[0]["context_type"] == "strategy_analysis"
    
    @pytest.mark.asyncio
    async def test_cleanup_old_memories(self, memory_manager, mock_redis):
        """Test cleanup of old memories"""
        # Mock old memories
        old_timestamp = (datetime.now() - timedelta(days=31)).isoformat()
        recent_timestamp = datetime.now().isoformat()
        
        mock_redis.keys.return_value = ["cerebro:memory:1", "cerebro:memory:2"]
        mock_redis.hgetall.side_effect = [
            {
                "content": "Old memory",
                "timestamp": old_timestamp,
                "context_type": "old_analysis"
            },
            {
                "content": "Recent memory", 
                "timestamp": recent_timestamp,
                "context_type": "recent_analysis"
            }
        ]
        
        result = await memory_manager.cleanup_old_memories(days=30)
        
        # Should delete the old memory
        assert result["deleted_count"] == 1
        mock_redis.delete.assert_called_once_with("cerebro:memory:1")
    
    @pytest.mark.asyncio
    async def test_get_memory_stats(self, memory_manager, mock_redis):
        """Test getting memory statistics"""
        mock_redis.keys.return_value = ["cerebro:memory:1", "cerebro:memory:2", "cerebro:memory:3"]
        mock_redis.info.return_value = {"used_memory": 1024000}
        
        stats = await memory_manager.get_memory_stats()
        
        assert stats["total_memories"] == 3
        assert stats["memory_usage_mb"] == 1.0
        assert "timestamp" in stats
    
    @pytest.mark.asyncio
    async def test_error_handling_store_context(self, memory_manager, mock_redis, mock_embedding_service):
        """Test error handling when storing context fails"""
        mock_embedding_service.create_embedding.side_effect = Exception("Embedding failed")
        
        result = await memory_manager.store_context("test content", "test_type")
        
        assert result["success"] is False
        assert "error" in result
    
    @pytest.mark.asyncio
    async def test_error_handling_search_context(self, memory_manager, mock_redis, mock_embedding_service):
        """Test error handling when search fails"""
        mock_embedding_service.create_embedding.side_effect = Exception("Search failed")
        
        results = await memory_manager.search_relevant_context("test query")
        
        assert results == []

class TestMemoryIntegration:
    """Integration tests for the complete memory system"""
    
    @pytest.mark.asyncio
    async def test_full_memory_workflow(self):
        """Test complete workflow: store -> search -> retrieve"""
        # This would require actual Redis and embedding service
        # For now, we'll test the workflow with mocks
        
        config = CerebroConfig()
        memory_manager = MemoryManager(config)
        
        # Mock the dependencies
        memory_manager.redis_client = Mock()
        memory_manager.embedding_service = Mock()
        
        # Mock successful operations
        memory_manager.redis_client.ping.return_value = True
        memory_manager.embedding_service.create_embedding = AsyncMock(return_value=[0.1] * 770)
        memory_manager.redis_client.set.return_value = True
        memory_manager.redis_client.hset.return_value = True
        memory_manager.redis_client.zadd.return_value = True
        
        # Store context
        store_result = await memory_manager.store_context(
            "Test trading analysis",
            "analysis",
            {"test": True}
        )
        
        assert store_result["success"] is True
        
        # Mock search results
        memory_manager.redis_client.keys.return_value = ["cerebro:memory:1"]
        memory_manager.redis_client.hgetall.return_value = {
            "content": "Test trading analysis",
            "context_type": "analysis",
            "embedding": json.dumps([0.1] * 770),
            "timestamp": datetime.now().isoformat(),
            "metadata": json.dumps({"test": True})
        }
        memory_manager.embedding_service.cosine_similarity.return_value = 0.95
        
        # Search for context
        search_results = await memory_manager.search_relevant_context("trading analysis")
        
        assert len(search_results) == 1
        assert search_results[0]["content"] == "Test trading analysis"
        assert search_results[0]["similarity_score"] == 0.95

# Test configuration and fixtures
@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()

if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
