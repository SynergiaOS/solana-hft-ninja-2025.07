#!/usr/bin/env python3
"""
Cerebro RAG Search Implementation
Retrieval-Augmented Generation with semantic search in DragonflyDB
"""

import asyncio
import aioredis
import numpy as np
import json
import logging
from typing import List, Dict, Any, Optional, Tuple
from datetime import datetime, timedelta

from .schema import ContextEntry, SearchQuery, SearchResult, MemorySchema, ContextType, ContextSource
from ..jina.client import CerebroEmbeddingClient

logger = logging.getLogger(__name__)


class CerebroRAGSearch:
    """RAG search engine for Cerebro memory system"""

    def __init__(
        self,
        redis_url: str = "redis://:cerebro_secure_2025@localhost:6379",
        embedding_model: str = "all-MiniLM-L6-v2"
    ):
        self.redis_url = redis_url
        self.redis_client = None
        self.embedding_client = CerebroEmbeddingClient(embedding_model)
        self.schema = MemorySchema()

    async def connect(self):
        """Connect to DragonflyDB"""
        try:
            self.redis_client = await aioredis.from_url(self.redis_url, decode_responses=True)
            await self.redis_client.ping()
            logger.info("✅ Connected to DragonflyDB for RAG search")
        except Exception as e:
            logger.error(f"❌ Failed to connect to DragonflyDB: {e}")
            raise

    async def close(self):
        """Close connections"""
        if self.redis_client:
            await self.redis_client.close()

    async def store_context(self, context: ContextEntry) -> bool:
        """Store context entry in memory"""
        try:
            # Store main context entry
            context_key = self.schema.context_key(context.context_id)
            context_data = context.to_json()
            await self.redis_client.set(context_key, context_data)

            # Update indexes
            await self._update_indexes(context)

            logger.info(f"✅ Stored context: {context.context_id}")
            return True

        except Exception as e:
            logger.error(f"❌ Failed to store context: {e}")
            return False

    async def _update_indexes(self, context: ContextEntry):
        """Update various indexes for efficient filtering"""
        try:
            # Type index
            type_key = self.schema.type_index_key(context.context_type)
            await self.redis_client.sadd(type_key, context.context_id)

            # Source index
            source_key = self.schema.source_index_key(context.source)
            await self.redis_client.sadd(source_key, context.context_id)

            # Strategy index (if applicable)
            if context.related_strategy:
                strategy_key = self.schema.strategy_index_key(context.related_strategy)
                await self.redis_client.sadd(strategy_key, context.context_id)

            # Time index (daily buckets)
            date_str = datetime.fromtimestamp(context.timestamp).strftime('%Y-%m-%d')
            time_key = self.schema.time_index_key(date_str)
            await self.redis_client.sadd(time_key, context.context_id)

        except Exception as e:
            logger.error(f"❌ Failed to update indexes: {e}")

    async def search(self, query: SearchQuery) -> List[SearchResult]:
        """Perform semantic search in memory"""
        try:
            # Get candidate context IDs based on filters
            candidate_ids = await self._get_filtered_candidates(query)

            if not candidate_ids:
                logger.info("No candidates found matching filters")
                return []

            # Load candidate contexts
            candidates = await self._load_contexts(candidate_ids)

            if not candidates:
                logger.info("No valid contexts loaded")
                return []

            # Perform semantic similarity search
            results = await self._semantic_search(query, candidates)

            logger.info(f"✅ Found {len(results)} relevant contexts")
            return results

        except Exception as e:
            logger.error(f"❌ Search failed: {e}")
            return []

    async def _get_filtered_candidates(self, query: SearchQuery) -> List[str]:
        """Get candidate context IDs based on filters"""
        try:
            candidate_sets = []

            # Filter by context types
            if query.context_types:
                for context_type in query.context_types:
                    type_key = self.schema.type_index_key(context_type)
                    candidate_sets.append(type_key)

            # Filter by sources
            if query.sources:
                for source in query.sources:
                    source_key = self.schema.source_index_key(source)
                    candidate_sets.append(source_key)

            # Filter by strategy
            if query.strategy_filter:
                strategy_key = self.schema.strategy_index_key(query.strategy_filter)
                candidate_sets.append(strategy_key)

            # Filter by time range
            if query.time_range:
                start_time, end_time = query.time_range
                start_date = datetime.fromtimestamp(start_time)
                end_date = datetime.fromtimestamp(end_time)

                current_date = start_date
                while current_date <= end_date:
                    date_str = current_date.strftime('%Y-%m-%d')
                    time_key = self.schema.time_index_key(date_str)
                    candidate_sets.append(time_key)
                    current_date += timedelta(days=1)

            # If no filters, get all contexts
            if not candidate_sets:
                all_keys = await self.redis_client.keys(f"{self.schema.CONTEXT_PREFIX}*")
                return [key.replace(self.schema.CONTEXT_PREFIX, "") for key in all_keys]

            # Intersect all filter sets
            if len(candidate_sets) == 1:
                candidates = await self.redis_client.smembers(candidate_sets[0])
            else:
                candidates = await self.redis_client.sinter(*candidate_sets)

            return list(candidates) if candidates else []

        except Exception as e:
            logger.error(f"❌ Failed to get filtered candidates: {e}")
            return []

    async def _load_contexts(self, context_ids: List[str]) -> List[ContextEntry]:
        """Load context entries from storage"""
        try:
            contexts = []

            for context_id in context_ids:
                context_key = self.schema.context_key(context_id)
                context_data = await self.redis_client.get(context_key)

                if context_data:
                    try:
                        context = ContextEntry.from_json(context_data)
                        contexts.append(context)
                    except Exception as e:
                        logger.warning(f"Failed to parse context {context_id}: {e}")

            return contexts

        except Exception as e:
            logger.error(f"❌ Failed to load contexts: {e}")
            return []

    async def _semantic_search(self, query: SearchQuery, candidates: List[ContextEntry]) -> List[SearchResult]:
        """Perform semantic similarity search"""
        try:
            # Generate query embedding
            query_embedding = self.embedding_client.embed_text(query.query_text)

            # Calculate similarities
            similarities = []
            for candidate in candidates:
                try:
                    candidate_embedding = np.array(candidate.vector)

                    # Cosine similarity
                    dot_product = np.dot(query_embedding, candidate_embedding)
                    norm1 = np.linalg.norm(query_embedding)
                    norm2 = np.linalg.norm(candidate_embedding)

                    if norm1 == 0 or norm2 == 0:
                        similarity = 0.0
                    else:
                        similarity = dot_product / (norm1 * norm2)

                    # Apply confidence filter
                    if query.min_confidence and candidate.confidence:
                        if candidate.confidence < query.min_confidence:
                            continue

                    # Apply similarity threshold
                    if similarity >= query.similarity_threshold:
                        similarities.append((candidate, float(similarity)))

                except Exception as e:
                    logger.warning(f"Failed to calculate similarity for context {candidate.context_id}: {e}")

            # Sort by similarity and limit results
            similarities.sort(key=lambda x: x[1], reverse=True)
            similarities = similarities[:query.max_results]

            # Create search results
            results = []
            for rank, (context, similarity) in enumerate(similarities):
                result = SearchResult(
                    context_entry=context,
                    similarity_score=similarity,
                    rank=rank + 1
                )
                results.append(result)

            return results

        except Exception as e:
            logger.error(f"❌ Semantic search failed: {e}")
            return []