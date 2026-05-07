//! Remote desktop tunnel: capture → H.264 encode → QUIC stream → browser.
//!
//! Wire framing (both directions):
//!   `[4 bytes LE: payload_len][payload_len bytes: payload]`
//! Outbound payload: raw H.264 NAL unit bytes.
//! Inbound payload:  UTF-8 JSON `InputEvent`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anyhow::Context as _;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use wg_shared::rdp::InputEvent;

use super::TunnelStream;

pub async fn run_remote_desktop_tunnel(
    stream:      TunnelStream,
    width:       u32,
    height:      u32,
    target_fps:  u32,
    target_kbps: u32,
) -> anyhow::Result<()> {
    let backend = crate::capture::open_capture_backend()
        .context("RDP: capture backend unavailable")?;
    let encoder = crate::encode::H264Encoder::new(width, height, target_fps, target_kbps)
        .context("RDP: encoder init failed")?;

    let cancelled = Arc::new(AtomicBool::new(false));
    let pli_flag  = Arc::new(AtomicBool::new(false));

    // frame channel: capture thread → write task (bounded to 2 to minimise lag)
    let (frame_tx, mut frame_rx) = tokio::sync::mpsc::channel::<Vec<Vec<u8>>>(2);
    // event channel: read loop → inject thread
    let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<InputEvent>(16);

    let (mut stream_read, stream_write) = (stream.read, stream.write);

    // --- Capture + encode thread (blocking) ---
    let cancelled_cap = cancelled.clone();
    let pli_cap       = pli_flag.clone();
    let cap_handle = tokio::task::spawn_blocking(move || {
        let mut backend   = backend;
        let mut encoder   = encoder;
        let frame_dur     = Duration::from_millis(1000 / target_fps.max(1) as u64);

        loop {
            if cancelled_cap.load(Ordering::Relaxed) { break; }

            if pli_cap.swap(false, Ordering::Relaxed) {
                encoder.force_intra_frame();
            }

            let t0 = Instant::now();
            match backend.capture().and_then(|f| encoder.encode_frame(&f)) {
                Ok(nals) if !nals.is_empty() => {
                    if frame_tx.blocking_send(nals).is_err() { break; }
                }
                Ok(_) => {} // encoder chose to skip this frame
                Err(e) => {
                    tracing::warn!("RDP capture/encode error: {e:#}");
                    break;
                }
            }

            let elapsed = t0.elapsed();
            if elapsed < frame_dur {
                std::thread::sleep(frame_dur - elapsed);
            }
        }
    });

    // --- Write task (async): relay NAL chunks to the tunnel stream ---
    let cancelled_w = cancelled.clone();
    let write_handle = tokio::spawn(async move {
        let mut w = stream_write;
        'write: loop {
            match frame_rx.recv().await {
                None => break, // channel closed; capture thread exited
                Some(nals) => {
                    for nal in &nals {
                        let len = (nal.len() as u32).to_le_bytes();
                        if w.write_all(&len).await.is_err() { break 'write; }
                        if w.write_all(nal).await.is_err()  { break 'write; }
                    }
                    if w.flush().await.is_err() { break; }
                }
            }
        }
        // Write side closed — signal all other tasks to stop.
        cancelled_w.store(true, Ordering::Relaxed);
    });

    // --- Input inject thread (blocking) ---
    let pli_inj = pli_flag.clone();
    let inject_handle = tokio::task::spawn_blocking(move || {
        let mut injector = crate::input::open_input_injector().ok();
        if injector.is_none() {
            tracing::warn!("RDP: input injection unavailable; video-only mode");
        }

        while let Some(event) = event_rx.blocking_recv() {
            match &event {
                InputEvent::Pli => { pli_inj.store(true, Ordering::Relaxed); }
                _ => {
                    if let Some(inj) = &mut injector {
                        if let Err(e) = inj.inject(event) {
                            tracing::warn!("RDP: input inject error: {e:#}");
                        }
                    }
                }
            }
        }
    });

    // --- Read loop: receive framed input events from the browser ---
    const MAX_EVENT_BYTES: usize = 16_384;
    let mut len_buf = [0u8; 4];
    loop {
        if stream_read.read_exact(&mut len_buf).await.is_err() {
            break; // EOF or connection reset
        }
        let len = u32::from_le_bytes(len_buf) as usize;
        if len > MAX_EVENT_BYTES {
            tracing::warn!("RDP: oversized input event ({len} B) — closing session");
            break;
        }
        let mut data = vec![0u8; len];
        if stream_read.read_exact(&mut data).await.is_err() {
            break;
        }
        match serde_json::from_slice::<InputEvent>(&data) {
            Ok(event) => { let _ = event_tx.send(event).await; }
            Err(e)    => tracing::debug!("RDP: invalid input event: {e}"),
        }
    }

    // Read side closed — signal capture thread to stop and drop the event
    // channel so the inject thread unblocks from blocking_recv.
    cancelled.store(true, Ordering::Relaxed);
    drop(event_tx);

    // Wait for all tasks before returning so the session is fully torn down.
    let _ = write_handle.await;
    let _ = inject_handle.await;
    let _ = cap_handle.await;

    Ok(())
}
