use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;

#[derive(Debug)]
pub enum RetrievalStrategy {
    Immediate,
    Await(Duration),
}

#[derive(Debug, Clone, Default)]
pub struct TokenProvider {
    token: Arc<RwLock<Option<String>>>,
}

impl TokenProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn update(&self, token: impl Into<String>) {
        let mut lock = self.token.write().await;
        *lock = Some(token.into());
    }

    pub async fn get(&self) -> Option<String> {
        self.token.read().await.clone()
    }

    pub async fn obtain(&self, strategy: RetrievalStrategy) -> Option<String> {
        match strategy {
            RetrievalStrategy::Immediate => self.get().await,
            RetrievalStrategy::Await(timeout) => {
                let deadline = Instant::now() + timeout;
                loop {
                    if let Some(token) = self.get().await {
                        return Some(token);
                    }

                    if Instant::now() >= deadline {
                        return None;
                    }

                    tokio::task::yield_now().await;
                }
            }
        }
    }
}
