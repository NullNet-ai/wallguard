use crate::control_channel::command::ExecutableCommand;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::{process::Stdio, sync::Arc};
use tokio::{
    io::AsyncReadExt,
    process::Command,
    sync::{Mutex, mpsc::Sender},
};
use wallguard_common::protobuf::wallguard_commands::{
    ClientMessage, ExecuteCliCommandRequest, ExecuteCliCommandResponse, client_message::Message,
};

type OutboundStream = Arc<Mutex<Sender<ClientMessage>>>;

pub struct ExecuteCliCommand {
    stream: OutboundStream,
    request: ExecuteCliCommandRequest,
}

impl ExecuteCliCommand {
    pub fn new(stream: OutboundStream, request: ExecuteCliCommandRequest) -> Self {
        Self { stream, request }
    }

    async fn send_response(self, stdout: String, stderr: String, status: i32) -> Result<(), Error> {
        let response = ExecuteCliCommandResponse {
            stdout,
            stderr,
            status,
        };

        let message = ClientMessage {
            message: Some(Message::ExecuteCliCommandResponse(response)),
        };

        self.stream
            .lock()
            .await
            .send(message)
            .await
            .handle_err(location!())
    }
}

impl ExecutableCommand for ExecuteCliCommand {
    async fn execute(self) -> Result<(), Error> {
        let Ok(mut child) = Command::new(&self.request.command)
            .args(&self.request.arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        else {
            return self
                .send_response("".into(), "Failed to spawn the commnd".into(), -1)
                .await;
        };

        let mut stdout = String::new();
        if let Some(mut out_pipe) = child.stdout.take() {
            let _ = out_pipe.read_to_string(&mut stdout).await;
        }

        let mut stderr = String::new();
        if let Some(mut err_pipe) = child.stderr.take() {
            let _ = err_pipe.read_to_string(&mut stderr).await;
        }

        let status = child
            .wait()
            .await
            .map(|status| status.code())
            .unwrap_or(None)
            .unwrap_or(-1);

        self.send_response(stdout, stderr, status).await
    }
}
