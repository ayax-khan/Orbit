//! DXGI Desktop Duplication capture backend (Windows only).
//!
//! Captures the primary monitor directly from the GPU and copies each frame
//! into a CPU-readable staging texture, yielding BGRA8 pixel data.

use super::{CaptureError, Frame, ScreenCapturer};
use windows::core::Interface;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_UNKNOWN;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
    D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_FLAG, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE,
    D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING, D3D11_SDK_VERSION,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIAdapter, IDXGIFactory1, IDXGIOutput, IDXGIOutput1,
    IDXGIOutputDuplication, IDXGIResource, DXGI_OUTDUPL_FRAME_INFO,
};

pub struct DxgiCapturer {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    duplication: IDXGIOutputDuplication,
    width: u32,
    height: u32,
    staging: Option<ID3D11Texture2D>,
}

// The COM objects are used from a single capture thread; the agent never
// shares a capturer across threads.
unsafe impl Send for DxgiCapturer {}

impl DxgiCapturer {
    pub fn new() -> Result<Self, CaptureError> {
        unsafe {
            let factory: IDXGIFactory1 =
                CreateDXGIFactory1().map_err(|e| CaptureError::Unavailable(e.to_string()))?;

            let adapter: IDXGIAdapter = factory
                .EnumAdapters(0)
                .map_err(|e| CaptureError::Unavailable(e.to_string()))?;

            let mut device: Option<ID3D11Device> = None;
            let mut context: Option<ID3D11DeviceContext> = None;
            D3D11CreateDevice(
                &adapter,
                D3D_DRIVER_TYPE_UNKNOWN,
                windows::Win32::Foundation::HMODULE::default(),
                D3D11_CREATE_DEVICE_FLAG(0),
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )
            .map_err(|e| CaptureError::Unavailable(e.to_string()))?;

            let device = device.ok_or_else(|| CaptureError::Unavailable("no D3D11 device".into()))?;
            let context =
                context.ok_or_else(|| CaptureError::Unavailable("no D3D11 context".into()))?;

            let output: IDXGIOutput = adapter
                .EnumOutputs(0)
                .map_err(|e| CaptureError::Unavailable(e.to_string()))?;
            let output1: IDXGIOutput1 = output
                .cast()
                .map_err(|e| CaptureError::Unavailable(e.to_string()))?;

            let desc = output.GetDesc().unwrap_or_default();
            let width = (desc.DesktopCoordinates.right - desc.DesktopCoordinates.left).max(0) as u32;
            let height = (desc.DesktopCoordinates.bottom - desc.DesktopCoordinates.top).max(0) as u32;

            let duplication = output1
                .DuplicateOutput(&device)
                .map_err(|e| CaptureError::Unavailable(e.to_string()))?;

            Ok(Self {
                device,
                context,
                duplication,
                width: if width == 0 { 1920 } else { width },
                height: if height == 0 { 1080 } else { height },
                staging: None,
            })
        }
    }

    /// Lazily create (or reuse) a CPU-readable staging texture matching `src`.
    unsafe fn ensure_staging(&mut self, src: &ID3D11Texture2D) -> Result<(), CaptureError> {
        if self.staging.is_some() {
            return Ok(());
        }
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        src.GetDesc(&mut desc);
        desc.Usage = D3D11_USAGE_STAGING;
        desc.BindFlags = 0;
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
        desc.MiscFlags = 0;

        let mut staging: Option<ID3D11Texture2D> = None;
        self.device
            .CreateTexture2D(&desc, None, Some(&mut staging))
            .map_err(|e| CaptureError::Acquire(e.to_string()))?;
        self.staging = staging;
        Ok(())
    }
}

impl ScreenCapturer for DxgiCapturer {
    fn next_frame(&mut self) -> Result<Option<Frame>, CaptureError> {
        unsafe {
            let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
            let mut resource: Option<IDXGIResource> = None;

            // 16ms timeout ~ one frame at 60fps.
            match self
                .duplication
                .AcquireNextFrame(16, &mut frame_info, &mut resource)
            {
                Ok(()) => {}
                Err(_) => return Ok(None), // timeout / no new frame
            }

            let result = (|| {
                let resource = resource
                    .as_ref()
                    .ok_or_else(|| CaptureError::Acquire("no resource".into()))?;
                let texture: ID3D11Texture2D = resource
                    .cast()
                    .map_err(|e| CaptureError::Acquire(e.to_string()))?;

                self.ensure_staging(&texture)?;
                let staging = self
                    .staging
                    .as_ref()
                    .ok_or_else(|| CaptureError::Acquire("no staging texture".into()))?
                    .clone();

                self.context.CopyResource(&staging, &texture);

                let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                self.context
                    .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
                    .map_err(|e| CaptureError::Acquire(e.to_string()))?;

                let row_pitch = mapped.RowPitch as usize;
                let row_bytes = (self.width as usize) * 4;
                let mut data = Vec::with_capacity(row_bytes * self.height as usize);
                let src = mapped.pData as *const u8;
                for y in 0..self.height as usize {
                    let row = src.add(y * row_pitch);
                    let slice = std::slice::from_raw_parts(row, row_bytes);
                    data.extend_from_slice(slice);
                }

                self.context.Unmap(&staging, 0);
                Ok(Frame::new(self.width, self.height, data))
            })();

            let _ = self.duplication.ReleaseFrame();
            result.map(Some)
        }
    }

    fn backend_name(&self) -> &'static str {
        "dxgi"
    }
}
