use base64::Engine as _;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;

#[derive(Debug)]
pub enum RetrievalStrategy {
    // Immediate,
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

    /// Extracts the device ID from the JWT's `account.device.id` claim.
    pub async fn device_id(&self) -> Option<String> {
        let jwt = self.get().await?;
        Self::decode_device_id(&jwt)
    }

    fn decode_device_id(jwt: &str) -> Option<String> {
        let payload = jwt.split('.').nth(1)?;
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .ok()?;
        let claims: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
        claims
            .get("account")?
            .get("device")?
            .get("id")?
            .as_str()
            .map(String::from)
    }

    pub async fn obtain(&self, strategy: RetrievalStrategy) -> Option<String> {
        match strategy {
            // RetrievalStrategy::Immediate => self.get().await,
            RetrievalStrategy::Await(timeout) => {
                let deadline = Instant::now() + timeout;
                loop {
                    if let Some(token) = self.get().await {
                        return Some(token);
                    }

                    if Instant::now() >= deadline {
                        return None;
                    }

                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }
}
