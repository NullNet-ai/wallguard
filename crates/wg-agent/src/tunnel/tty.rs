use std::thread;

use bytes::Bytes;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::sync::mpsc;

use super::TunnelStream;

/// Spawn a PTY shell and relay stdin/stdout through the tunnel stream.
pub async fn run_tty_tunnel(stream: TunnelStream, shell: &str) -> anyhow::Result<()> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows:         24,
        cols:         80,
        pixel_width:  0,
        pixel_height: 0,
    })?;

    let mut cmd = CommandBuilder::new(shell);
    cmd.env("TERM", "xterm-256color");
    let _child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);

    let mut pty_reader = pair.master.try_clone_reader()?;
    let mut pty_writer = pair.master.take_writer()?;

    // stdin channel: async relay loop → PTY writer thread
    let (stdin_tx, mut stdin_rx) = mpsc::channel::<Bytes>(16);
    // stdout channel: PTY reader thread → async relay loop
    let (stdout_tx, mut stdout_rx) = mpsc::channel::<Bytes>(16);

    // PTY reader thread: reads PTY output, sends to async side.
    thread::spawn(move || {
        let mut buf = vec![0u8; 4096];
        loop {
            match std::io::Read::read(&mut pty_reader, &mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if stdout_tx.blocking_send(Bytes::copy_from_slice(&buf[..n])).is_err() {
                        break;
                    }
                }
            }
        }
    });

    // PTY writer thread: receives from async side, writes to PTY.
    thread::spawn(move || {
        while let Some(data) = stdin_rx.blocking_recv() {
            if std::io::Write::write_all(&mut pty_writer, &data).is_err() {
                break;
            }
        }
    });

    let mut stream_r = stream.read;
    let mut stream_w = stream.write;
    let mut buf = vec![0u8; 4096];

    loop {
        tokio::select! {
            result = stream_r.read(&mut buf) => {
                match result {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if stdin_tx.send(Bytes::copy_from_slice(&buf[..n])).await.is_err() {
                            break;
                        }
                    }
                }
            }
            data = stdout_rx.recv() => {
                let Some(data) = data else { break };
                if stream_w.write_all(&data).await.is_err() {
                    break;
                }
            }
        }
    }

    // Dropping pair.master here closes the PTY master, sending SIGHUP to the shell.
    drop(pair.master);
    Ok(())
}
