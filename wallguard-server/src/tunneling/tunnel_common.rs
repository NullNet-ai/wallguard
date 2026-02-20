use async_trait::async_trait;
use nullnet_liberror::Error;
use std::fmt::Debug;
use std::pin::Pin;

use crate::{
    app_context::AppContext,
    datastore::{ServiceInfo, TunnelModel},
    reverse_tunnel::TunnelInstance,
    tunneling::{
        async_io::AsyncIo,
        tunnel_common_data::{TunnelCommonData, TunnelCreateError},
    },
};

#[async_trait]
pub trait TunnelCommon: Debug + Send + Sync {
    async fn create(context: AppContext, data: TunnelCommonData) -> Result<Self, TunnelCreateError>
    where
        Self: Sized;

    async fn request_tunnel(&self) -> Result<TunnelInstance, Error>;

    async fn request_session(&self) -> Result<Pin<Box<dyn AsyncIo + Send>>, Error>;

    async fn terminate(&self) -> Result<(), Error>;

    fn get_service_data(&self) -> &ServiceInfo;

    fn get_tunnel_data(&self) -> &TunnelModel;
}
