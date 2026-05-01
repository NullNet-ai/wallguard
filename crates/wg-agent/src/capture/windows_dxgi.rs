//! Windows screen capture via DXGI Desktop Duplication API.
//!
//! Requires Windows 8 / WDDM 1.2 or later.  Captures the primary monitor
//! as a BGRA frame using a D3D11 staging texture for CPU readback.

use anyhow::{Context as _, Result, bail};
use windows::Win32::Graphics::{
    Direct3D::D3D_DRIVER_TYPE_HARDWARE,
    Direct3D11::{
        D3D11CreateDevice, D3D11_BIND_FLAG, D3D11_CPU_ACCESS_READ,
        D3D11_CREATE_DEVICE_FLAG, D3D11_MAP_READ, D3D11_RESOURCE_MISC_FLAG,
        D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
        ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
    },
    Dxgi::{
        IDXGIOutput1, IDXGIOutputDuplication, IDXGIResource,
        DXGI_OUTDUPL_FRAME_INFO,
    },
};

use super::{CaptureBackend, Frame};

pub struct DxgiCapture {
    device:      ID3D11Device,
    context:     ID3D11DeviceContext,
    duplication: IDXGIOutputDuplication,
    width:       u32,
    height:      u32,
}

// Safety: DXGI/D3D11 COM objects are reference-counted and can be moved
// across threads, provided each interface is used from at most one thread
// at a time.  DxgiCapture is used exclusively from a single capture thread.
unsafe impl Send for DxgiCapture {}

impl DxgiCapture {
    pub fn new() -> Result<Self> {
        unsafe {
            // Create D3D11 device.
            let mut device:  Option<ID3D11Device>        = None;
            let mut context: Option<ID3D11DeviceContext> = None;
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_FLAG(0),
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )
            .context("DXGI: D3D11CreateDevice failed")?;

            let device  = device.context("DXGI: D3D11 device is None")?;
            let context = context.context("DXGI: D3D11 context is None")?;

            // Walk the DXGI adapter chain to find the first output.
            use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};
            let factory: IDXGIFactory1 = CreateDXGIFactory1()
                .context("DXGI: CreateDXGIFactory1 failed")?;
            let adapter = factory.EnumAdapters1(0)
                .context("DXGI: no adapter at index 0")?;
            let output = adapter.EnumOutputs(0)
                .context("DXGI: no output (monitor) at index 0")?;
            let output1: IDXGIOutput1 = output.cast()
                .context("DXGI: IDXGIOutput1 not supported (Windows 8+ required)")?;

            let mut desc = Default::default();
            output.GetDesc(&mut desc);
            let width  = (desc.DesktopCoordinates.right  - desc.DesktopCoordinates.left) as u32;
            let height = (desc.DesktopCoordinates.bottom - desc.DesktopCoordinates.top)  as u32;

            if width == 0 || height == 0 {
                bail!("DXGI: monitor reported zero dimensions ({width}×{height})");
            }

            let duplication = output1.DuplicateOutput(&device)
                .context("DXGI: DuplicateOutput failed")?;

            tracing::debug!(width, height, "DXGI capture backend initialised");
            Ok(Self { device, context, duplication, width, height })
        }
    }

    /// Copy a GPU texture into a staging texture readable by the CPU.
    unsafe fn copy_to_staging(&self, src: &ID3D11Texture2D) -> Result<ID3D11Texture2D> {
        let desc = D3D11_TEXTURE2D_DESC {
            Width:          self.width,
            Height:         self.height,
            MipLevels:      1,
            ArraySize:      1,
            Format:         windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc:     windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC {
                Count: 1, Quality: 0,
            },
            Usage:          D3D11_USAGE_STAGING,
            BindFlags:      D3D11_BIND_FLAG(0),
            CPUAccessFlags: D3D11_CPU_ACCESS_READ,
            MiscFlags:      D3D11_RESOURCE_MISC_FLAG(0),
        };
        let mut staging: Option<ID3D11Texture2D> = None;
        self.device
            .CreateTexture2D(&desc, None, Some(&mut staging))
            .context("DXGI: CreateTexture2D (staging) failed")?;
        let staging = staging.context("DXGI: staging texture is None")?;
        self.context.CopyResource(&staging, src);
        Ok(staging)
    }
}

impl CaptureBackend for DxgiCapture {
    fn capture(&mut self) -> Result<Frame> {
        unsafe {
            let mut frame_info  = DXGI_OUTDUPL_FRAME_INFO::default();
            let mut resource: Option<IDXGIResource> = None;

            // 100 ms timeout — caller is responsible for frame-rate pacing.
            self.duplication
                .AcquireNextFrame(100, &mut frame_info, &mut resource)
                .context("DXGI: AcquireNextFrame failed")?;

            let resource  = resource.context("DXGI: frame resource is None")?;
            let gpu_tex: ID3D11Texture2D = resource.cast()
                .context("DXGI: resource is not a Texture2D")?;

            let staging = self.copy_to_staging(&gpu_tex)?;
            self.duplication.ReleaseFrame().ok();

            // Map the staging texture for CPU read.
            let mut mapped = Default::default();
            self.context
                .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
                .context("DXGI: Map failed")?;

            let row_pitch = mapped.RowPitch as usize;
            let total     = row_pitch * self.height as usize;
            let src_slice = std::slice::from_raw_parts(
                mapped.pData as *const u8,
                total,
            );

            // Copy row-by-row in case the staging pitch differs from width*4.
            let stride = self.width as usize * 4;
            let mut data = vec![0u8; stride * self.height as usize];
            for row in 0..self.height as usize {
                let src = &src_slice[row * row_pitch .. row * row_pitch + stride];
                let dst = &mut data[row * stride .. (row + 1) * stride];
                dst.copy_from_slice(src);
            }

            self.context.Unmap(&staging, 0);

            Ok(Frame {
                width:  self.width,
                height: self.height,
                stride: self.width * 4,
                data,
            })
        }
    }
}
