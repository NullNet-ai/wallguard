use crate::app_context::AppContext;
use futures_util::Stream;
use rmcp::{
    model::{ClientJsonRpcMessage, ServerJsonRpcMessage},
    transport::{
        WorkerTransport,
        common::server_side_http::ServerSseMessage,
        streamable_http_server::{
            SessionId, SessionManager,
            session::local::{LocalSessionManager, LocalSessionManagerError, LocalSessionWorker},
        },
    },
};

pub struct SessionManagerEx {
    context: AppContext,
    inner: LocalSessionManager,
}

impl SessionManagerEx {
    pub fn new(context: AppContext) -> Self {
        Self {
            context,
            inner: Default::default(),
        }
    }
}

impl SessionManager for SessionManagerEx {
    type Error = LocalSessionManagerError;
    type Transport = WorkerTransport<LocalSessionWorker>;

    async fn create_session(&self) -> Result<(SessionId, Self::Transport), Self::Error> {
        self.inner.create_session().await
    }

    async fn initialize_session(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<ServerJsonRpcMessage, Self::Error> {
        self.inner.initialize_session(id, message).await
    }

    async fn has_session(&self, id: &SessionId) -> Result<bool, Self::Error> {
        self.inner.has_session(id).await
    }

    async fn close_session(&self, id: &SessionId) -> Result<(), Self::Error> {
        self.context.mcp_sessions.lock().await.remove(id.as_ref());
        self.inner.close_session(id).await
    }

    async fn create_stream(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + 'static, Self::Error> {
        self.inner.create_stream(id, message).await
    }

    async fn accept_message(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> Result<(), Self::Error> {
        self.inner.accept_message(id, message).await
    }

    async fn create_standalone_stream(
        &self,
        id: &SessionId,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + 'static, Self::Error> {
        self.inner.create_standalone_stream(id).await
    }

    async fn resume(
        &self,
        id: &SessionId,
        last_event_id: String,
    ) -> Result<impl Stream<Item = ServerSseMessage> + Send + 'static, Self::Error> {
        self.inner.resume(id, last_event_id).await
    }
}
