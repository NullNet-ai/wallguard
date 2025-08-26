use std::{pin::Pin, task::Poll};

use crate::reverse_tunnel::TunnelInstance;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tokio::io::{AsyncRead, AsyncWrite};
use wallguard_common::protobuf::wallguard_tunnel::client_frame::Message as ClientMessage;
use wallguard_common::protobuf::wallguard_tunnel::server_frame::Message as ServerMessage;
use wallguard_common::protobuf::wallguard_tunnel::{ClientFrame, DataFrame, ServerFrame};

/// Adapter that implements `AsyncRead` and `AsyncWrite` for a `TunnelInstance`.
///
/// This adapter bridges the gap between the async gRPC-based tunnel interface
/// and the standard Tokio `AsyncRead`/`AsyncWrite` traits, allowing the tunnel
/// to be used with any code that expects standard async I/O streams.
///
/// The adapter handles:
/// - Converting between byte streams and protobuf frames
/// - Buffering partial frame data to prevent data loss
/// - Managing async tasks for non-blocking I/O operations
#[derive(Debug)]
pub struct TunnelAdapter {
    tunnel: TunnelInstance,

    read_task: Option<tokio::task::JoinHandle<Result<ClientFrame, Error>>>,
    write_task: Option<tokio::task::JoinHandle<Result<usize, Error>>>,

    frame_buffer: Option<Vec<u8>>,
    buffer_offset: usize,
}

impl TryFrom<TunnelInstance> for TunnelAdapter {
    fn try_from(tunnel: TunnelInstance) -> Result<Self, Error> {
        if tunnel.authenticated {
            Ok(Self {
                tunnel,
                read_task: None,
                write_task: None,
                frame_buffer: None,
                buffer_offset: 0,
            })
        } else {
            Err("TunnelAdapter: Expected an authenticated client").handle_err(location!())
        }
    }

    type Error = Error;
}

impl AsyncRead for TunnelAdapter {
    /// Polls for read readiness, attempting to read data from the tunnel into the provided buffer.
    ///
    /// This implementation:
    /// 1. First serves any buffered data from previous partial reads
    /// 2. Spawns a new read task if none is active
    /// 3. Polls the read task for completion
    /// 4. Converts received `ClientFrame` to bytes and copies to the buffer
    /// 5. Buffers any remaining data that doesn't fit in the current read
    ///
    /// # Arguments
    /// * `cx` - Task context for waking
    /// * `buf` - Buffer to fill with read data
    ///
    /// # Returns
    /// * `Poll::Ready(Ok(()))` - Data was successfully read
    /// * `Poll::Ready(Err(_))` - An error occurred during reading
    /// * `Poll::Pending` - No data available yet, task will be woken when ready
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if let Some(frame_data) = self.frame_buffer.clone() {
            let remaining_in_frame = frame_data.len() - self.buffer_offset;

            if remaining_in_frame > 0 {
                let bytes_to_copy = std::cmp::min(remaining_in_frame, buf.remaining());
                let start = self.buffer_offset;
                let end = start + bytes_to_copy;
                buf.put_slice(&frame_data[start..end]);
                self.buffer_offset += bytes_to_copy;

                if self.buffer_offset >= frame_data.len() {
                    self.frame_buffer = None;
                    self.buffer_offset = 0;
                }

                return Poll::Ready(Ok(()));
            }
        }

        if self.read_task.is_none() {
            let tunnel = self.tunnel.clone();
            self.read_task = Some(tokio::spawn(async move { tunnel.read().await }));
        }

        if let Some(task) = &mut self.read_task {
            match Pin::new(task).poll(cx) {
                Poll::Ready(Ok(Ok(frame))) => {
                    self.read_task = None;

                    let Some(message) = frame.message else {
                        return Poll::Ready(Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "TunnelAdapter: Client send an empty message",
                        )));
                    };

                    let ClientMessage::Data(data_frame) = message else {
                        return Poll::Ready(Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "TunnelAdapter: Client send an unepected message",
                        )));
                    };

                    let frame_data = data_frame.data;
                    let bytes_to_copy = std::cmp::min(frame_data.len(), buf.remaining());
                    buf.put_slice(&frame_data[..bytes_to_copy]);

                    if bytes_to_copy < frame_data.len() {
                        self.frame_buffer = Some(frame_data);
                        self.buffer_offset = bytes_to_copy;
                    }

                    Poll::Ready(Ok(()))
                }
                Poll::Ready(Ok(Err(err))) => {
                    self.read_task = None;
                    Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("TunnelAdapter: Tunnel read error: {}", err.to_str()),
                    )))
                }
                Poll::Ready(Err(join_error)) => {
                    self.read_task = None;
                    Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("TunnelAdapter: Read task error: {}", join_error),
                    )))
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}

impl AsyncWrite for TunnelAdapter {
    /// Polls for write readiness, attempting to write data from the buffer to the tunnel.
    ///
    /// This implementation:
    /// 1. Spawns a write task if none is active, wrapping the data in a `ServerFrame`
    /// 2. Polls the write task for completion
    /// 3. Returns the number of bytes written on success
    ///
    /// # Arguments
    /// * `cx` - Task context for waking
    /// * `buf` - Buffer containing data to write
    ///
    /// # Returns
    /// * `Poll::Ready(Ok(n))` - `n` bytes were successfully written
    /// * `Poll::Ready(Err(_))` - An error occurred during writing
    /// * `Poll::Pending` - Write not ready yet, task will be woken when ready
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        if self.write_task.is_none() {
            let tunnel = self.tunnel.clone();
            let data = buf.to_vec();
            let bytes_to_write = buf.len();

            let server_frame = ServerFrame {
                message: Some(ServerMessage::Data(DataFrame { data: data })),
            };

            self.write_task = Some(tokio::spawn(async move {
                tunnel.write(server_frame).await.map(|_| bytes_to_write)
            }));
        }

        if let Some(task) = &mut self.write_task {
            match Pin::new(task).poll(cx) {
                Poll::Ready(Ok(Ok(bytes_written))) => {
                    self.write_task = None;
                    Poll::Ready(Ok(bytes_written))
                }
                Poll::Ready(Ok(Err(err))) => {
                    self.write_task = None;
                    Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("TunnelAdapter: Tunnel write error: {}", err.to_str()),
                    )))
                }
                Poll::Ready(Err(join_error)) => {
                    self.write_task = None;
                    Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("TunnelAdapter: Write task error: {}", join_error),
                    )))
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }

    /// Polls for flush completion, ensuring all pending writes are transmitted.
    ///
    /// This implementation waits for any active write task to complete,
    /// ensuring data is fully transmitted before considering the flush complete.
    ///
    /// # Arguments
    /// * `cx` - Task context for waking
    ///
    /// # Returns
    /// * `Poll::Ready(Ok(()))` - All data has been flushed
    /// * `Poll::Ready(Err(_))` - An error occurred during flushing
    /// * `Poll::Pending` - Flush not complete yet, task will be woken when ready
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        if let Some(task) = &mut self.write_task {
            match Pin::new(task).poll(cx) {
                Poll::Ready(Ok(Ok(_))) => {
                    self.write_task = None;
                    Poll::Ready(Ok(()))
                }
                Poll::Ready(Ok(Err(err))) => {
                    self.write_task = None;
                    Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("TunnelAdapter: Flush error: {}", err.to_str()),
                    )))
                }
                Poll::Ready(Err(join_error)) => {
                    self.write_task = None;
                    Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("TunnelAdapter: Flush task error: {}", join_error),
                    )))
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }

    /// Polls for shutdown completion, ensuring all data is flushed before closing.
    ///
    /// This implementation first ensures all pending writes are flushed,
    /// then considers the shutdown complete. For this tunnel implementation,
    /// no additional close frames are sent.
    ///
    /// # Arguments
    /// * `cx` - Task context for waking
    ///
    /// # Returns
    /// * `Poll::Ready(Ok(()))` - Shutdown completed successfully
    /// * `Poll::Ready(Err(_))` - An error occurred during shutdown
    /// * `Poll::Pending` - Shutdown not complete yet, task will be woken when ready
    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.as_mut().poll_flush(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}
