#!/usr/bin/env python3
"""
Cerebro Jina Client
Simple client for text embedding without full Jina framework
"""

import numpy as np
import httpx
import json
import logging
from typing import List, Dict, Any, Optional
from sentence_transformers import SentenceTransformer

logger = logging.getLogger(__name__)


class CerebroEmbeddingClient:
    """Simple embedding client using sentence-transformers"""

    def __init__(self, model_name: str = "jinaai/jina-embeddings-v2-base-en"):
        self.model_name = model_name
        try:
            self.model = SentenceTransformer(model_name)
            logger.info(f"✅ Loaded embedding model: {model_name}")
        except Exception as e:
            logger.error(f"❌ Failed to load model {model_name}: {e}")
            # Fallback to a smaller model
            self.model = SentenceTransformer('all-MiniLM-L6-v2')
            logger.info("✅ Using fallback model: all-MiniLM-L6-v2")

    def embed_text(self, text: str) -> np.ndarray:
        """Generate embedding for a single text"""
        try:
            embedding = self.model.encode(text, convert_to_numpy=True)
            return embedding
        except Exception as e:
            logger.error(f"❌ Failed to embed text: {e}")
            return np.zeros(384)  # Return zero vector as fallback

    def embed_texts(self, texts: List[str]) -> List[np.ndarray]:
        """Generate embeddings for multiple texts"""
        try:
            embeddings = self.model.encode(texts, convert_to_numpy=True)
            return [emb for emb in embeddings]
        except Exception as e:
            logger.error(f"❌ Failed to embed texts: {e}")
            return [np.zeros(384) for _ in texts]

    def similarity(self, text1: str, text2: str) -> float:
        """Calculate cosine similarity between two texts"""
        try:
            emb1 = self.embed_text(text1)
            emb2 = self.embed_text(text2)

            # Cosine similarity
            dot_product = np.dot(emb1, emb2)
            norm1 = np.linalg.norm(emb1)
            norm2 = np.linalg.norm(emb2)

            if norm1 == 0 or norm2 == 0:
                return 0.0

            return dot_product / (norm1 * norm2)
        except Exception as e:
            logger.error(f"❌ Failed to calculate similarity: {e}")
            return 0.0

    def find_similar(self, query_text: str, candidate_texts: List[str], top_k: int = 5) -> List[Dict[str, Any]]:
        """Find most similar texts to query"""
        try:
            query_emb = self.embed_text(query_text)
            candidate_embs = self.embed_texts(candidate_texts)

            similarities = []
            for i, candidate_emb in enumerate(candidate_embs):
                # Cosine similarity
                dot_product = np.dot(query_emb, candidate_emb)
                norm1 = np.linalg.norm(query_emb)
                norm2 = np.linalg.norm(candidate_emb)

                if norm1 == 0 or norm2 == 0:
                    similarity = 0.0
                else:
                    similarity = dot_product / (norm1 * norm2)

                similarities.append({
                    'text': candidate_texts[i],
                    'similarity': float(similarity),
                    'index': i
                })

            # Sort by similarity and return top_k
            similarities.sort(key=lambda x: x['similarity'], reverse=True)
            return similarities[:top_k]

        except Exception as e:
            logger.error(f"❌ Failed to find similar texts: {e}")
            return []


# Test function
def test_embedding_client():
    """Test the embedding client"""
    print("🧪 Testing Cerebro Embedding Client...")

    client = CerebroEmbeddingClient()

    # Test single embedding
    text = "Trading strategy performance analysis"
    embedding = client.embed_text(text)
    print(f"✅ Single embedding shape: {embedding.shape}")

    # Test multiple embeddings
    texts = [
        "Sandwich attack detected in mempool",
        "Arbitrage opportunity found between DEXes",
        "Risk management triggered stop loss",
        "Market making strategy optimization"
    ]
    embeddings = client.embed_texts(texts)
    print(f"✅ Multiple embeddings: {len(embeddings)} vectors")

    # Test similarity
    query = "MEV strategy analysis"
    similar = client.find_similar(query, texts, top_k=2)
    print(f"✅ Similar texts to '{query}':")
    for item in similar:
        print(f"  - {item['text']} (similarity: {item['similarity']:.3f})")

    print("🎉 Embedding client test completed!")


if __name__ == "__main__":
    test_embedding_client()