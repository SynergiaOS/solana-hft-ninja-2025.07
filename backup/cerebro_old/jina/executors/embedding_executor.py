#!/usr/bin/env python3
"""
Cerebro Embedding Executor
Jina executor for text embedding using jina-embeddings-v2
"""

import numpy as np
from typing import Dict, List, Any, Optional
from jina import Executor, requests, DocumentArray, Document
import torch
from transformers import AutoModel, AutoTokenizer
import logging

logger = logging.getLogger(__name__)


class CerebroEmbeddingExecutor(Executor):
    """Executor for generating text embeddings using Jina embeddings model"""

    def __init__(
        self,
        model_name: str = "jinaai/jina-embeddings-v2-base-en",
        device: str = "cpu",
        batch_size: int = 32,
        max_length: int = 512,
        **kwargs
    ):
        super().__init__(**kwargs)
        self.model_name = model_name
        self.device = device
        self.batch_size = batch_size
        self.max_length = max_length

        # Load model and tokenizer
        try:
            self.tokenizer = AutoTokenizer.from_pretrained(model_name, trust_remote_code=True)
            self.model = AutoModel.from_pretrained(model_name, trust_remote_code=True)
            self.model.to(device)
            self.model.eval()
            logger.info(f"✅ Loaded embedding model: {model_name}")
        except Exception as e:
            logger.error(f"❌ Failed to load model {model_name}: {e}")
            raise

    @requests(on='/embed')
    def embed_text(self, docs: DocumentArray, **kwargs) -> DocumentArray:
        """Generate embeddings for input texts"""
        try:
            texts = [doc.text for doc in docs if doc.text]

            if not texts:
                logger.warning("No texts to embed")
                return docs

            # Process in batches
            all_embeddings = []
            for i in range(0, len(texts), self.batch_size):
                batch_texts = texts[i:i + self.batch_size]
                batch_embeddings = self._embed_batch(batch_texts)
                all_embeddings.extend(batch_embeddings)

            # Assign embeddings to documents
            for doc, embedding in zip(docs, all_embeddings):
                if embedding is not None:
                    doc.embedding = embedding
                    doc.tags['embedding_model'] = self.model_name
                    doc.tags['embedding_dim'] = len(embedding)

            logger.info(f"✅ Generated embeddings for {len(texts)} texts")
            return docs

        except Exception as e:
            logger.error(f"❌ Embedding generation failed: {e}")
            return docs

    def _embed_batch(self, texts: List[str]) -> List[Optional[np.ndarray]]:
        """Generate embeddings for a batch of texts"""
        try:
            # Tokenize
            inputs = self.tokenizer(
                texts,
                max_length=self.max_length,
                padding=True,
                truncation=True,
                return_tensors='pt'
            ).to(self.device)

            # Generate embeddings
            with torch.no_grad():
                outputs = self.model(**inputs)
                # Use mean pooling
                embeddings = outputs.last_hidden_state.mean(dim=1)
                embeddings = embeddings.cpu().numpy()

            return [emb for emb in embeddings]

        except Exception as e:
            logger.error(f"❌ Batch embedding failed: {e}")
            return [None] * len(texts)

    @requests(on='/search')
    def search_similar(self, docs: DocumentArray, **kwargs) -> DocumentArray:
        """Search for similar embeddings (placeholder for vector search)"""
        # This would typically interface with DragonflyDB
        # For now, just return the input docs
        logger.info("🔍 Similarity search requested")
        return docs