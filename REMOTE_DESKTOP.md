# WallGuard Remote Desktop — Implementation Plan

## Overview

This document specifies how to replace the `captis`-stub with a fully working, cross-platform remote desktop feature. The design follows the same relay pattern used by SSH and TTY tunnels (agent → QUIC → server → WebSocket → browser) and adds a screen capture backend, an H.264 encoding pipeline, and a WebCodecs-based browser renderer.

`captis` is **dropped**. The capture layer is written in-house using platform-stable crates (`x11rb`, `ashpd`/PipeWire, `windows-rs`) whose licenses and FreeBSD support are fully understood.

---

## Goals

- **Cross-platform**: Linux (X11 + Wayland), FreeBSD (X11), Windows. All three must compile to the same binary behind a `remote-desktop` feature flag.
- **Runtime probe**: If no display is reachable (headless appliance), `Feature::RemoteDesktop` is simply not advertised — no compile-time exclusion.
- **Low-latency**: Target ≤ 150 ms glass-to-glass over a LAN.
- **Bandwidth-adaptive**: Agent drops frames under congestion rather than buffering them.
- **Secure**: No plain-text pixel data leaves the agent; session tokens authorize each stream; mouse/keyboard input is restricted to authenticated users.
- **Minimal extra dependencies**: Prefer crates that are already in the workspace or that compile cleanly on all three platforms.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Browser (wg-ui / WebAssembly)                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  RemoteDesktop component                                         │   │
│  │  ┌────────────────┐   ┌─────────────────────────────────────┐  │   │
│  │  │  WebCodecs      │   │  Input capture (keyboard, mouse,    │  │   │
│  │  │  VideoDecoder   │   │  clipboard) → JSON over WebSocket   │  │   │
│  │  │  → <canvas>    │   └─────────────────────────────────────┘  │   │
│  │  └────────────────┘                                              │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│             ▲  binary H.264 NAL chunks          │  JSON input events     │
│             │  (WebSocket, binary frames)        │                        │
└─────────────┼──────────────────────────────────-┼────────────────────────┘
              │                                    │
┌─────────────┼────────────────────────────────────┼────────────────────────┐
│  wg-server  │                                    │                        │
│  ┌──────────┴──────────────────────────────────┴──────────────────┐     │
│  │  WS handler: /api/v1/devices/{id}/tunnels/rdp/{session_id}     │     │
│  │  • video frames: WS binary → QUIC stream write                 │     │
│  │  • input events: WS text  → QUIC stream write (reverse dir)    │     │
│  │  • same bidirectional relay pattern as SSH/TTY ws.rs           │     │
│  └────────────────────────────────────────────────────────────────┘     │
└───────────────────────────────────────────────────────────────────────────┘
              │ QUIC stream (TLS, mTLS)
┌─────────────┼───────────────────────────────────────────────────────────┐
│  wg-agent   │                                                           │
│  ┌──────────┴──────────────────────────────────────────────────────┐   │
│  │  tunnel/remote_desktop.rs                                       │   │
│  │  ┌────────────────┐  frames  ┌───────────┐  NAL chunks  ┌────┐ │   │
│  │  │ CaptureBackend ├─────────►│ H264Enc   ├─────────────►│    │ │   │
│  │  │ (platform)     │          │ (openh264)│              │ QU │ │   │
│  │  └────────────────┘          └───────────┘              │ IC │ │   │
│  │  ┌────────────────┐  events                             │    │ │   │
│  │  │ InputInjector  │◄────────────────────────────────────│    │ │   │
│  │  │ (enigo/uinput) │                                     └────┘ │   │
│  │  └────────────────┘                                            │   │
│  └────────────────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────────────────┘
```

The server is a **dumb relay** — it never decodes video or interprets input. All media logic lives in the agent and the browser.

---

## Component Design

### 1. Screen Capture Backend (`wg-agent/src/capture/`)

`captis` is replaced by a `CaptureBackend` trait with three platform implementations selected at runtime:

```rust
pub trait CaptureBackend: Send {
    // Returns an BGRA/RGBA frame + dimensions.  Called at the negotiated fps.
    fn capture(&mut self) -> Result<Frame>;
    // Probe: returns Ok(backend) or Err(reason) for capability advertisement.
    fn probe() -> Result<Self> where Self: Sized;
}
```

#### X11 backend (`capture/x11.rs`) — Linux + FreeBSD

- Crate: `x11rb` (pure Rust, no C bindings beyond the X11 wire protocol)
- Use `MIT-SHM` shared memory extension when available (zero-copy path), fall back to `GetImage`
- Target: `DefaultRootWindow` or per-window capture if a window ID is supplied
- Probe: attempt `xcb_connect()` against `$DISPLAY`; fail cleanly if no socket

#### Wayland backend (`capture/wayland.rs`) — Linux only

- Crate: `ashpd` (XDG Desktop Portal) + `pipewire` for the PipeWire screenshare stream
- The portal presents a permissions dialog to the local user once; subsequent captures use the saved token
- This is the only backend that requires user interaction at the display seat — appropriate for non-headless machines
- Probe: check that `$WAYLAND_DISPLAY` is set and the portal is reachable

#### Windows backend (`capture/windows.rs`) — Windows only

- Crate: `windows-rs` with `windows::Win32::Graphics::Dxgi` (Desktop Duplication API)
- DXGI Desktop Duplication gives a GPU-backed texture at monitor refresh rate with zero extra copies
- Probe: attempt `IDXGIOutputDuplication::AcquireNextFrame`

#### Runtime selection

```rust
pub fn open_capture_backend() -> Result<Box<dyn CaptureBackend>> {
    #[cfg(target_os = "windows")]
    return DxgiCapture::probe().map(|b| Box::new(b) as _);

    #[cfg(unix)]
    {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            if let Ok(b) = WaylandCapture::probe() { return Ok(Box::new(b)); }
        }
        X11Capture::probe().map(|b| Box::new(b) as _)
    }
}
```

This is the function called by `capabilities.rs` — `Err(...)` means `Feature::RemoteDesktop` is not advertised.

---

### 2. Encoding Pipeline (`wg-agent/src/encode/`)

#### Encoder: `openh264` (already a declared dependency)

- Baseline profile, constant-bitrate mode, configurable fps/kbps from `OpenRemoteDesktopTunnel`
- Keyframe every 2 s (or on PLI — see below)
- Frame drop policy: if the encode queue has > 1 unprocessed frame, drop the oldest; never block the capture loop

#### Frame path

```
CaptureBackend::capture()          → BGRA frame
→ scale_if_needed()                → rescale to negotiated resolution (fast_image_resize crate)
→ bgra_to_yuv420()                 → color space conversion (SIMD via wide or manual)
→ openh264::Encoder::encode()      → SvcBitstreamData (one or more NAL units)
→ frame_channel (bounded, cap=2)   → QUIC stream writer
```

#### Codec negotiation (future)

The initial implementation is H.264-only. The proto message already carries a `codec` field (stub). A second codec (VP9 or AV1) can be added later without breaking the wire format.

---

### 3. Transport & Protocol

#### QUIC stream (not datagrams)

The original design proposed QUIC datagrams for video. After consideration, the implementation uses a **QUIC stream with a small send window** instead:

- Datagrams require a QUIC implementation detail (`max_datagram_frame_size`) that is not yet negotiated in our Quinn endpoint config and may be dropped silently by intermediate paths.
- A QUIC stream provides backpressure; when the browser is slow the agent's send buffer fills and the frame-drop policy in the encode pipeline kicks in — same outcome as datagram loss but explicit.
- The existing tunnel transport code (`tunnel/transport.rs`) already handles QUIC streams. No new framing is needed.

If latency measurements later show that stream head-of-line blocking is a real problem, upgrading to datagrams is a one-file change in `transport.rs`.

#### Wire framing (over the QUIC stream)

Outbound (agent → server → browser):
```
[4 bytes: payload_len as u32 LE] [payload_len bytes: NAL chunk]
```

Inbound (browser → server → agent — input events):
```
[4 bytes: payload_len as u32 LE] [payload_len bytes: JSON input event]
```

JSON input event schema (same direction as SSH input, keeps the server relay trivial):
```json
{ "t": "mouse_move",  "x": 1234, "y": 567 }
{ "t": "mouse_down",  "btn": 0 }
{ "t": "mouse_up",    "btn": 0 }
{ "t": "mouse_scroll","dx": 0, "dy": -3 }
{ "t": "key_down",    "code": "KeyA" }
{ "t": "key_up",      "code": "KeyA" }
{ "t": "clipboard",   "text": "..." }
```

A Rust enum in `wg-shared/src/rdp.rs` mirrors this for the agent to deserialize.

#### PLI (Picture Loss Indication)

The browser sends `{ "t": "pli" }` when WebCodecs reports a decode error. The agent responds with the next IDR frame immediately (call `openh264::Encoder::force_intra_frame()`).

#### Proto changes

`control.proto` already has `OpenRemoteDesktopTunnel`; add:
```proto
message CloseRemoteDesktopTunnel { string session_id = 1; }
```
to `ServerMessage` so the server can tear down a session without closing the QUIC connection.

---

### 4. Server Relay

File: `wg-server/src/api/ws.rs` — add `rdp()` handler alongside existing `ssh()` and `tty()`.

The handler is structurally identical to `tty()`:
1. Upgrade the HTTP connection to WebSocket.
2. Look up `session_id` in `TunnelRegistry` (populated when the agent opens the QUIC stream).
3. Spawn two tasks:
   - **ws_to_quic**: receive WebSocket binary frames (video from agent), forward as WS binary to browser.
   - **quic_to_ws**: receive WebSocket text frames (input from browser), forward over QUIC stream to agent.

No video parsing, no buffering beyond a `BytesMut` staging buffer for the 4-byte length prefix.

API endpoint:
```
GET /api/v1/devices/{id}/tunnels/rdp/{session_id}
POST /api/v1/devices/{id}/tunnels/rdp           → OpenRemoteDesktopTunnel → session_id
```

`api/tunnels.rs` gains a `post_rdp()` handler that sends `OpenRemoteDesktopTunnel` and returns the session ID — same pattern as SSH/TTY.

---

### 5. Input Injection (`wg-agent/src/input/`)

#### Primary: `enigo`

Already a declared dependency. Handles X11 (Linux + FreeBSD) and Windows.

Verify FreeBSD X11 path before Phase 8e: `enigo` uses `xdotool`-style Xlib calls which work on FreeBSD if `libX11` is installed. If the crate doesn't compile on FreeBSD, pin to the `xdo` feature or contribute the fix upstream — the surface area is small (mouse move + click + key events).

#### Wayland: `uinput`

On Wayland, `enigo` cannot inject input without root privileges. Use the Linux `uinput` kernel module instead: write a `UinputInjector` that opens `/dev/uinput` and synthesizes evdev events. This requires the agent to run as root or have `CAP_INPUT_RAW` — acceptable for an appliance daemon.

Gate at compile time:
```rust
#[cfg(target_os = "linux")]
mod uinput;
```

At runtime, try `enigo` first; if it fails (Wayland session without Xwayland), fall back to `uinput`.

#### Input dispatcher

```rust
pub fn open_input_injector() -> Box<dyn InputInjector> { ... }
pub trait InputInjector: Send {
    fn inject(&mut self, event: InputEvent) -> Result<()>;
}
```

---

### 6. Browser Component (`wg-ui/src/components/remote_desktop.rs`)

Replace the current stub with a real component. The browser side is the most constrained layer because it must be pure WASM + `web-sys`.

#### Rendering pipeline

```
WebSocket (binary) → ArrayBuffer
→ VideoDecoder (WebCodecs API)   ← configured on first keyframe (SPS/PPS NALs)
→ VideoFrame
→ ImageBitmap
→ CanvasRenderingContext2d::drawImage()   (or OffscreenCanvas for lower latency)
```

WebCodecs is available in Chrome 94+, Firefox 130+, Safari 16.4+. A `video/mp4` MediaSource fallback can be added later if we need broader support, but it adds ~2 s latency.

#### Input capture

```
keydown / keyup / mousemove / mousedown / mouseup / wheel events on <canvas>
→ serialize to JSON InputEvent
→ WebSocket.send() (text frame)
```

The canvas is put in `tabindex=0` and `pointer-lock` is requested on click to suppress the browser's own mouse handling.

#### Clipboard

Browser Clipboard API (`navigator.clipboard.readText/writeText`) is gated on page focus. A clipboard button outside the canvas lets the user push text to the remote session explicitly — simpler and more secure than transparent paste.

#### Connection lifecycle

```
mount → POST /api/v1/devices/{id}/tunnels/rdp → session_id
      → WS /api/v1/devices/{id}/tunnels/rdp/{session_id}?token=…
      → wait for first IDR frame
      → configure VideoDecoder (SPS/PPS)
      → start rendering loop
unmount → close WebSocket → server sends CloseRemoteDesktopTunnel to agent
```

---

## Platform Support Matrix

| Platform | Capture backend | Input injection | Notes |
|---|---|---|---|
| Linux (X11) | `x11rb` + MIT-SHM | `enigo` | Preferred, zero-copy path |
| Linux (Wayland) | `ashpd` + PipeWire | `uinput` (root) | Needs `CAP_INPUT_RAW` |
| FreeBSD (X11) | `x11rb` | `enigo` / Xlib | pfSense/OPNsense with display |
| Windows | `windows-rs` DXGI | `enigo` | DDA requires WDDM 1.2+ |
| Headless (any) | probe fails → `Err` | — | `Feature::RemoteDesktop` not advertised |

---

## Dependency Changes

Add to `wg-agent/Cargo.toml` under `[features] remote-desktop`:
```toml
[features]
remote-desktop = [
    "dep:openh264",
    "dep:enigo",
    "dep:x11rb",            # X11 capture (Linux + FreeBSD)
    "dep:fast_image_resize", # frame scaling
]

[target.'cfg(target_os = "linux")'.dependencies]
ashpd    = { version = "0.9", optional = true, features = ["pipewire"] }

[target.'cfg(target_os = "windows")'.dependencies]
windows  = { version = "0.58", optional = true, features = ["Win32_Graphics_Dxgi"] }
```

No new server-side dependencies. The relay is pure byte forwarding.

---

## Implementation Phases

### Phase A — Capture + encode (agent only, no UI)

1. Write `capture/mod.rs` with the `CaptureBackend` trait and `Frame` type.
2. Implement `capture/x11.rs` (Linux + FreeBSD). Verify with a standalone binary that captures to a PNG.
3. Implement `capture/windows.rs`. Same verification step.
4. Implement `capture/wayland.rs`. Mark unstable — only tested if CI has a Wayland runner.
5. Wire `open_capture_backend()` into `capabilities.rs` replacing the `captis::Display::open()` stub.
6. Implement `encode/mod.rs` wrapping `openh264`. Write a test that captures one frame and produces a valid SPS/PPS + IDR NAL sequence (check with `h264bitstream` or similar).

### Phase B — Agent tunnel (QUIC stream open, frame send)

7. Replace `tunnel/remote_desktop.rs` stub: open QUIC stream, write tunnel hello, spawn capture+encode task, send framed NAL chunks.
8. Implement input event deserialization (`wg-shared/src/rdp.rs`) and `input/` dispatcher.
9. Add `CloseRemoteDesktopTunnel` to `control.proto` and regenerate.

### Phase C — Server relay

10. Add `rdp()` WebSocket handler in `api/ws.rs`.
11. Add `POST /devices/{id}/tunnels/rdp` in `api/tunnels.rs`.
12. Extend `TunnelRegistry` to hold an RDP slot type (or reuse the existing generic slot).

### Phase D — Browser UI

13. Replace `components/remote_desktop.rs` stub with the real component (WebCodecs decoder + canvas renderer + input capture).
14. Add `api/tunnels.rs` client-side call for `POST .../tunnels/rdp`.
15. Add the RDP tab to `pages/devices/tunnels.rs`.

### Phase E — End-to-end testing

16. Integration test `remote_desktop_tunnel` in `wg-testkit`: mock capture backend → encode → relay → verify NAL chunks arrive at browser-side WebSocket.
17. Integration test `remote_desktop_pli`: inject decode error event, verify agent sends IDR.
18. Integration test `remote_desktop_headless`: probe returns `Err`, feature absent from Hello.
19. Integration test `remote_desktop_freebsd_with_display`: mock probe returns `Ok`, feature advertised.

---

## Security Considerations

- **Authorization**: The `POST /devices/{id}/tunnels/rdp` endpoint requires Admin+ role, same as SSH/TTY. The WebSocket upgrade validates the JWT token in the `?token=` query parameter.
- **Session isolation**: Each `session_id` is a random UUID. The server only pairs one WebSocket to one agent QUIC stream — no cross-session leakage.
- **Input injection surface**: The agent only processes `InputEvent` structs deserialized from the trusted QUIC stream (mTLS peer-verified). No shell is involved.
- **Clipboard**: Clipboard sync is opt-in (explicit button press) to prevent silent data exfiltration via a compromised session.
- **Frame encryption**: All video travels inside the existing QUIC+TLS tunnel — no additional encryption layer needed.

---

## Open Questions

| # | Question | Recommendation |
|---|---|---|
| 1 | Does `enigo` compile and work on FreeBSD with Xlib? | Test on a FreeBSD 14 VM before Phase B; if not, contribute the Xlib backend or use raw Xlib via `x11rb` for input too. |
| 2 | Does `ashpd` 0.9 build on musl (for the static agent release)? | Check; if not, gate Wayland support behind `target_env = "gnu"`. |
| 3 | Should we target a specific H.264 profile? | Baseline is safe. Main profile enables CABAC (better compression) — measure decode load in a mid-range browser tab. |
| 4 | Multi-monitor support? | Phase A captures the primary monitor (monitor index 0). Multi-monitor can be added as a follow-up by exposing the monitor list through a new proto field. |
| 5 | Frame rate and resolution limits? | Default: 1920×1080 @ 15 fps, 2 Mbps. Expose as config in `OpenRemoteDesktopTunnel` — already has `width`, `height`, `fps`, `kbps` fields. |
