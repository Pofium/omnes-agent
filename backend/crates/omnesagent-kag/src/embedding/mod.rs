//! Провайдеры векторных представлений (Embeddings).

pub mod fake;
#[cfg(feature = "candle")]
pub mod local_bert;

use async_trait::async_trait;
use std::sync::Arc;

pub use fake::FakeEmbedding;
#[cfg(feature = "candle")]
pub use local_bert::LocalBertEmbedding;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Получить эмбеддинги для списка текстов (батч).
    async fn embed(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>>;

    /// Размерность вектора.
    fn dim(&self) -> usize;
}

/// Фабрика провайдера эмбеддингов по умолчанию.
pub fn default_embedding_provider() -> Arc<dyn EmbeddingProvider> {
    #[cfg(feature = "candle")]
    {
        match LocalBertEmbedding::new("local") {
            Ok(local) => Arc::new(local),
            Err(e) => {
                tracing::warn!("Не удалось инициализировать Candle embedder ({e}), используется FakeEmbedding");
                Arc::new(FakeEmbedding::new(384))
            }
        }
    }
    #[cfg(not(feature = "candle"))]
    {
        Arc::new(FakeEmbedding::new(384))
    }
}
