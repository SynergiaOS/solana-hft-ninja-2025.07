#!/usr/bin/env python3
"""
Cerebro Memory Management API
FastAPI endpoints for memory operations
"""

from fastapi import APIRouter, HTTPException, Depends
from pydantic import BaseModel
from typing import List, Dict, Any, Optional
import time
import logging

from .schema import ContextEntry, SearchQuery, SearchResult, ContextType, ContextSource
from .rag_search import CerebroRAGSearch
from ..jina.client import CerebroEmbeddingClient

logger = logging.getLogger(__name__)

# Global instances
rag_search = None
embedding_client = None

# Pydantic models for API
class ContextCreateRequest(BaseModel):
    content: str
    context_type: str
    source: str
    confidence: Optional[float] = None
    tags: Optional[Dict[str, Any]] = None
    related_strategy: Optional[str] = None
    profit_impact: Optional[float] = None

class ContextResponse(BaseModel):
    context_id: str
    content: str
    context_type: str
    source: str
    timestamp: float
    confidence: Optional[float]
    tags: Optional[Dict[str, Any]]
    related_strategy: Optional[str]
    profit_impact: Optional[float]

class SearchRequest(BaseModel):
    query_text: str
    context_types: Optional[List[str]] = None
    sources: Optional[List[str]] = None
    time_range: Optional[List[float]] = None  # [start_timestamp, end_timestamp]
    strategy_filter: Optional[str] = None
    min_confidence: Optional[float] = None
    max_results: int = 10
    similarity_threshold: float = 0.7

class SearchResponse(BaseModel):
    results: List[Dict[str, Any]]
    total_found: int
    query_time_ms: int

# Router
router = APIRouter(prefix="/memory", tags=["memory"])

async def get_rag_search():
    """Dependency to get RAG search instance"""
    global rag_search
    if rag_search is None:
        rag_search = CerebroRAGSearch()
        await rag_search.connect()
    return rag_search

async def get_embedding_client():
    """Dependency to get embedding client"""
    global embedding_client
    if embedding_client is None:
        embedding_client = CerebroEmbeddingClient()
    return embedding_client

@router.post("/save", response_model=ContextResponse)
async def save_context(
    request: ContextCreateRequest,
    rag_search: CerebroRAGSearch = Depends(get_rag_search),
    embedding_client: CerebroEmbeddingClient = Depends(get_embedding_client)
):
    """Save new context to memory"""
    try:
        # Generate embedding for content
        embedding = embedding_client.embed_text(request.content)

        # Create context entry
        context = ContextEntry(
            content=request.content,
            vector=embedding.tolist(),
            context_type=ContextType(request.context_type),
            source=ContextSource(request.source),
            timestamp=time.time(),
            confidence=request.confidence,
            tags=request.tags,
            related_strategy=request.related_strategy,
            profit_impact=request.profit_impact
        )

        # Store in memory
        success = await rag_search.store_context(context)

        if not success:
            raise HTTPException(status_code=500, detail="Failed to store context")

        return ContextResponse(
            context_id=context.context_id,
            content=context.content,
            context_type=context.context_type.value,
            source=context.source.value,
            timestamp=context.timestamp,
            confidence=context.confidence,
            tags=context.tags,
            related_strategy=context.related_strategy,
            profit_impact=context.profit_impact
        )

    except ValueError as e:
        raise HTTPException(status_code=400, detail=f"Invalid enum value: {e}")
    except Exception as e:
        logger.error(f"Failed to save context: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@router.post("/search", response_model=SearchResponse)
async def search_memory(
    request: SearchRequest,
    rag_search: CerebroRAGSearch = Depends(get_rag_search)
):
    """Search memory for relevant contexts"""
    start_time = time.time()

    try:
        # Convert request to SearchQuery
        context_types = None
        if request.context_types:
            context_types = [ContextType(ct) for ct in request.context_types]

        sources = None
        if request.sources:
            sources = [ContextSource(s) for s in request.sources]

        time_range = None
        if request.time_range and len(request.time_range) == 2:
            time_range = tuple(request.time_range)

        query = SearchQuery(
            query_text=request.query_text,
            context_types=context_types,
            sources=sources,
            time_range=time_range,
            strategy_filter=request.strategy_filter,
            min_confidence=request.min_confidence,
            max_results=request.max_results,
            similarity_threshold=request.similarity_threshold
        )

        # Perform search
        results = await rag_search.search(query)

        # Convert results to response format
        result_dicts = [result.to_dict() for result in results]

        query_time_ms = int((time.time() - start_time) * 1000)

        return SearchResponse(
            results=result_dicts,
            total_found=len(results),
            query_time_ms=query_time_ms
        )

    except ValueError as e:
        raise HTTPException(status_code=400, detail=f"Invalid enum value: {e}")
    except Exception as e:
        logger.error(f"Search failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@router.get("/stats")
async def get_memory_stats(
    rag_search: CerebroRAGSearch = Depends(get_rag_search)
):
    """Get memory statistics"""
    try:
        # Count contexts by type
        type_counts = {}
        for context_type in ContextType:
            type_key = rag_search.schema.type_index_key(context_type)
            count = await rag_search.redis_client.scard(type_key)
            type_counts[context_type.value] = count

        # Count contexts by source
        source_counts = {}
        for source in ContextSource:
            source_key = rag_search.schema.source_index_key(source)
            count = await rag_search.redis_client.scard(source_key)
            source_counts[source.value] = count

        # Total contexts
        all_keys = await rag_search.redis_client.keys(f"{rag_search.schema.CONTEXT_PREFIX}*")
        total_contexts = len(all_keys)

        # Memory usage
        info = await rag_search.redis_client.info("memory")
        memory_mb = int(info.get("used_memory", 0)) / 1024 / 1024

        return {
            "total_contexts": total_contexts,
            "memory_usage_mb": round(memory_mb, 2),
            "contexts_by_type": type_counts,
            "contexts_by_source": source_counts,
            "timestamp": time.time()
        }

    except Exception as e:
        logger.error(f"Failed to get stats: {e}")
        raise HTTPException(status_code=500, detail=str(e))