// use crate::{context::Context, daemon::Daemon, utilities};
// use nullnet_liberror::{location, Error, ErrorHandler, Location};
// use nullnet_libwallguard::{authorization_status::State, AuthorizationApproved};
// use std::sync::Arc;
// use tokio::sync::{broadcast, Mutex};

// #[derive(Debug, Clone)]
// pub struct AuthorizationTask {
//     daemon: Arc<Mutex<Daemon>>,
//     shutdown: broadcast::Sender<()>,
//     timestamp: u64,
// }

// impl AuthorizationTask {
//     pub fn new(daemon: Arc<Mutex<Daemon>>, context: Context, org_id: String) -> Self {
//         let timestamp = utilities::time::timestamp();
//         let (shutdown, mut receiver) = broadcast::channel(1);

//         tokio::spawn(task(daemon.clone(), receiver, context, org_id));

//         Self {
//             daemon,
//             shutdown,
//             timestamp: timestamp as u64,
//         }
//     }

//     pub fn timestamp(&self) -> u64 {
//         self.timestamp
//     }

//     pub fn shutdown(&self) {
//         let _ = self.shutdown.send(());
//     }
// }

// async fn task(
//     daemon: Arc<Mutex<Daemon>>,
//     mut receiver: broadcast::Receiver<()>,
//     context: Context,
//     org_id: String,
// ) {
//     tokio::select! {
//         _ = receiver.recv() => {}
//         result = wait_for_authorization(daemon.clone(), context, org_id) => {
//             match result {
//                 Ok(data) => {
//                     Daemon::on_authorized(daemon, data).await;
//                 },
//                 Err(err) => {
//                     Daemon::on_error(daemon, err.to_str()).await;
//                 }
//             };
//         }
//     }
// }

// async fn wait_for_authorization(
//     daemon: Arc<Mutex<Daemon>>,
//     context: Context,
//     org_id: String,
// ) -> Result<AuthorizationApproved, Error> {
//     let device_uuid = Daemon::get_uuid(daemon).await;

//     let mut stream = context
//         .server
//         .authorization_request(&device_uuid, &org_id)
//         .await
//         .handle_err(location!())?;

//     loop {
//         let status = stream.message().await.handle_err(location!())?;

//         if status.is_none() {
//             return Err("Stream closed unexpectedly").handle_err(location!());
//         }

//         let state = status.unwrap().state;

//         if state.is_none() {
//             return Err("Server responeded with empty state").handle_err(location!());
//         }

//         match state.unwrap() {
//             State::Pending(_) => {
//                 tokio::task::yield_now().await;
//                 continue;
//             }
//             State::Rejected(_) => return Err("Authorization rejected").handle_err(location!()),
//             State::Approved(authorization_approved) => return Ok(authorization_approved),
//         };
//     }
// }

