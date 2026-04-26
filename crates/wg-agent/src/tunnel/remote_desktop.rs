use super::TunnelStream;

/// Phase 8 stub — captis screen capture backend is pending source verification.
pub async fn run_remote_desktop_tunnel(
    _stream:      TunnelStream,
    _width:       u32,
    _height:      u32,
    _target_fps:  u32,
    _target_kbps: u32,
) -> anyhow::Result<()> {
    Err(anyhow::anyhow!(
        "remote desktop: captis screen capture backend pending (Phase 8)"
    ))
}
