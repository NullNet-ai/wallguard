//! H.264 encoding pipeline wrapping `openh264`.
//!
//! [`H264Encoder`] accepts BGRA [`Frame`]s from the capture backend and
//! returns raw NAL units suitable for framing over the tunnel stream.
//! Resizing to the negotiated dimensions is handled inline when the captured
//! frame does not match the encoder's configured resolution.

use anyhow::{Context as _, Result};
use openh264::OpenH264API;
use openh264::encoder::{BitRate, Encoder, EncoderConfig, FrameRate};
use openh264::formats::YUVSlices;

use crate::capture::Frame;

pub struct H264Encoder {
    encoder: Encoder,
    width:   u32,
    height:  u32,
}

// Safety: openh264's Encoder holds raw C pointers.  Each H264Encoder is
// used exclusively from one dedicated capture/encode thread at a time.
unsafe impl Send for H264Encoder {}

impl H264Encoder {
    /// Create a new encoder targeting the given resolution, frame rate, and
    /// bit rate.  Returns `Err` if the openh264 library cannot be initialised.
    pub fn new(width: u32, height: u32, fps: u32, kbps: u32) -> Result<Self> {
        let config = EncoderConfig::new()
            .max_frame_rate(FrameRate::from_hz(fps as f32))
            .bitrate(BitRate::from_bps(kbps * 1000))
            .skip_frames(true);

        let encoder = Encoder::with_api_config(OpenH264API::from_source(), config)
            .context("openh264: failed to create encoder")?;

        Ok(Self { encoder, width, height })
    }

    /// Signal the encoder to produce an IDR (keyframe) on the next
    /// [`encode_frame`] call.  Used to respond to PLI from the browser.
    pub fn force_intra_frame(&mut self) {
        self.encoder.force_intra_frame();
    }

    /// Encode one BGRA frame and return NAL units as individual byte vectors.
    ///
    /// For the first frame (and after a forced IDR) this includes SPS + PPS
    /// NAL units followed by an IDR slice.
    pub fn encode_frame(&mut self, frame: &Frame) -> Result<Vec<Vec<u8>>> {
        let bgra = if frame.width == self.width && frame.height == self.height {
            None
        } else {
            Some(scale_nearest(frame, self.width, self.height))
        };
        let bgra_ref: &[u8] = bgra.as_deref().unwrap_or(&frame.data);

        let (y, u, v) = bgra_to_yuv420(bgra_ref, self.width, self.height);
        let w = self.width  as usize;
        let h = self.height as usize;

        let yuv = YUVSlices::new(
            (&y, &u, &v),
            (w, h),
            (w, w / 2, w / 2),
        );

        let bitstream = self.encoder.encode(&yuv)
            .context("openh264: encode failed")?;

        let mut nals: Vec<Vec<u8>> = Vec::new();
        for layer_idx in 0..bitstream.num_layers() {
            if let Some(layer) = bitstream.layer(layer_idx) {
                for nal_idx in 0..layer.nal_count() {
                    if let Some(nal) = layer.nal_unit(nal_idx) {
                        nals.push(nal.to_vec());
                    }
                }
            }
        }

        Ok(nals)
    }
}

// ---------------------------------------------------------------------------
// Colour space conversion
// ---------------------------------------------------------------------------

/// Convert a BGRA (or BGRX) buffer to YUV 4:2:0 planar format.
/// Uses BT.601 limited-range coefficients (Y: 16–235, UV: 16–240).
fn bgra_to_yuv420(bgra: &[u8], width: u32, height: u32) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let w = width  as usize;
    let h = height as usize;

    let mut y_plane = vec![0u8; w * h];
    let mut u_plane = vec![0u8; (w / 2) * (h / 2)];
    let mut v_plane = vec![0u8; (w / 2) * (h / 2)];

    for row in 0..h {
        for col in 0..w {
            let i = (row * w + col) * 4;
            let b = bgra[i]     as i32;
            let g = bgra[i + 1] as i32;
            let r = bgra[i + 2] as i32;

            let y = ((66 * r + 129 * g +  25 * b + 128) >> 8) + 16;
            y_plane[row * w + col] = y.clamp(16, 235) as u8;

            if row % 2 == 0 && col % 2 == 0 {
                let uv_idx = (row / 2) * (w / 2) + (col / 2);
                let u = ((-38 * r -  74 * g + 112 * b + 128) >> 8) + 128;
                let v = ((112 * r -  94 * g -  18 * b + 128) >> 8) + 128;
                u_plane[uv_idx] = u.clamp(16, 240) as u8;
                v_plane[uv_idx] = v.clamp(16, 240) as u8;
            }
        }
    }

    (y_plane, u_plane, v_plane)
}

/// Nearest-neighbour downscale of a BGRA frame to `dst_w × dst_h`.
fn scale_nearest(frame: &Frame, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let src_w = frame.width  as usize;
    let src_h = frame.height as usize;
    let dw    = dst_w as usize;
    let dh    = dst_h as usize;

    let mut out = vec![0u8; dw * dh * 4];
    for dy in 0..dh {
        let sy = dy * src_h / dh;
        for dx in 0..dw {
            let sx    = dx * src_w / dw;
            let src_i = (sy * src_w + sx) * 4;
            let dst_i = (dy * dw    + dx) * 4;
            out[dst_i..dst_i + 4].copy_from_slice(&frame.data[src_i..src_i + 4]);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_bgra_frame(width: u32, height: u32, b: u8, g: u8, r: u8) -> Frame {
        let mut data = vec![0u8; (width * height * 4) as usize];
        for chunk in data.chunks_exact_mut(4) {
            chunk[0] = b;
            chunk[1] = g;
            chunk[2] = r;
            chunk[3] = 0xff;
        }
        Frame { width, height, stride: width * 4, data }
    }

    #[test]
    fn yuv_white_round_trips() {
        let (y, u, v) = bgra_to_yuv420(&vec![0xffu8; 4 * 4 * 4], 4, 4);
        assert!(y.iter().all(|&b| b >= 220), "white Y should be near 235");
        assert!(u.iter().all(|&b| (120..=136).contains(&b)), "white U ≈ 128");
        assert!(v.iter().all(|&b| (120..=136).contains(&b)), "white V ≈ 128");
    }

    #[test]
    fn yuv_black_round_trips() {
        let (y, u, v) = bgra_to_yuv420(&vec![0x00u8; 4 * 4 * 4], 4, 4);
        assert!(y.iter().all(|&b| b <= 20), "black Y should be near 16");
        assert!(u.iter().all(|&b| (120..=136).contains(&b)), "black U ≈ 128");
        assert!(v.iter().all(|&b| (120..=136).contains(&b)), "black V ≈ 128");
    }

    #[test]
    fn scale_nearest_halves_dimensions() {
        let frame = solid_bgra_frame(64, 64, 0x11, 0x22, 0x33);
        let scaled = scale_nearest(&frame, 32, 32);
        assert_eq!(scaled.len(), 32 * 32 * 4);
        for chunk in scaled.chunks_exact(4) {
            assert_eq!(chunk[0], 0x11);
            assert_eq!(chunk[1], 0x22);
            assert_eq!(chunk[2], 0x33);
        }
    }

    #[test]
    fn encoder_produces_nal_units() {
        let mut enc = H264Encoder::new(320, 240, 15, 500)
            .expect("encoder init should succeed");

        let frame = solid_bgra_frame(320, 240, 128, 128, 128);
        let nals  = enc.encode_frame(&frame).expect("encode should succeed");

        assert!(!nals.is_empty(), "first frame must produce at least one NAL");
        // Baseline H.264: first encode emits SPS + PPS + IDR = 3+ NAL units.
        assert!(
            nals.len() >= 3,
            "expected SPS, PPS, and IDR NAL units; got {}",
            nals.len()
        );
        for nal in &nals {
            assert!(
                nal.starts_with(&[0x00, 0x00, 0x00, 0x01]),
                "NAL unit missing start code: {:02x?}",
                &nal[..nal.len().min(8)]
            );
        }
    }

    #[test]
    fn encoder_accepts_scaled_frame() {
        let mut enc = H264Encoder::new(160, 120, 15, 200)
            .expect("encoder init should succeed");

        // Feed a frame at double the encoder resolution — must be downscaled.
        let frame = solid_bgra_frame(320, 240, 0, 200, 0);
        let nals  = enc.encode_frame(&frame).expect("scaled encode should succeed");
        assert!(!nals.is_empty());
    }
}
