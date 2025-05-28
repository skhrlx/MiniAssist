use std::io::{Result, Error, ErrorKind};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{
            Direct3D11::*,
            Direct3D::*,
            Dxgi::*,
            Dxgi::Common::*,
        },
    },
};

// Constants for desktop capture
pub const ACQUISITION_TIMEOUT: u32 = 0; // Frame acquisition timeout (ms)

// Struct to hold desktop capture resources
pub struct DesktopCapture {
    device: Option<ID3D11Device>,
    context: Option<ID3D11DeviceContext>,
    duplication: Option<IDXGIOutputDuplication>,
    staging_texture: Option<ID3D11Texture2D>,
    screen_width: i32,
    screen_height: i32,
}

impl DesktopCapture {
    // Create a new DesktopCapture instance
    pub unsafe fn new(screen_width: i32, screen_height: i32) -> Result<Self> {
        let (device, context, duplication, staging_texture) = Self::setup_directx(screen_width, screen_height)?;
        
        Ok(DesktopCapture {
            device,
            context,
            duplication,
            staging_texture,
            screen_width,
            screen_height,
        })
    }

    // Setup DirectX and create resources needed for screen capture
    unsafe fn setup_directx(
        screen_width: i32,
        screen_height: i32
    ) -> Result<(Option<ID3D11Device>, Option<ID3D11DeviceContext>, Option<IDXGIOutputDuplication>, Option<ID3D11Texture2D>)> {
        // Setup Direct3D11 with maximum performance settings
        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;
        let mut feature_level = D3D_FEATURE_LEVEL_9_3;
        
        // Create device with hardware acceleration
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HINSTANCE(0),
            D3D11_CREATE_DEVICE_FLAG(0),
            Some(&[feature_level]),
            D3D11_SDK_VERSION,
            Some(&mut device),
            Some(&mut feature_level),
            Some(&mut context),
        )?;

        // Create a staging texture optimized for CPU access
        let desc = D3D11_TEXTURE2D_DESC {
            Width: screen_width as u32,
            Height: screen_height as u32,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };

        let mut staging_texture: Option<ID3D11Texture2D> = None;
        device.as_ref().unwrap().CreateTexture2D(
            &desc,
            None,
            Some(&mut staging_texture),
        )?;

        // Get desktop duplication interface
        let device_dxgi: IDXGIDevice = device.as_ref().unwrap().cast()?;
        let adapter: IDXGIAdapter = device_dxgi.GetParent()?;
        let output: IDXGIOutput = adapter.EnumOutputs(0)?;
        let output1: IDXGIOutput1 = output.cast()?;
        
        let duplication = output1.DuplicateOutput(device.as_ref().unwrap())?;

        Ok((device, context, Some(duplication), staging_texture))
    }

    // Capture a screenshot
    pub unsafe fn capture_screenshot(&self) -> Result<Vec<u8>> {
        // Wait for a frame
        let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
        let mut resource: Option<IDXGIResource> = None;
        
        // Try to acquire the next frame with a short timeout
        let result = self.duplication.as_ref().unwrap().AcquireNextFrame(ACQUISITION_TIMEOUT, &mut frame_info, &mut resource);
        if result.is_err() {
            // Don't treat timeout as a critical error
            if result.err().unwrap().code() == DXGI_ERROR_WAIT_TIMEOUT {
                return Err(Error::new(ErrorKind::TimedOut, "Frame acquisition timed out"));
            }
            return Err(Error::new(ErrorKind::Other, "Failed to acquire frame"));
        }

        let desktop_texture: ID3D11Texture2D = resource.unwrap().cast()?;
        
        // Copy to staging texture
        let staging_resource: ID3D11Resource = self.staging_texture.as_ref().unwrap().cast()?;
        let desktop_resource: ID3D11Resource = desktop_texture.cast()?;
        
        self.context.as_ref().unwrap().CopyResource(
            &staging_resource,
            &desktop_resource,
        );
        
        // Map the staging texture
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        
        self.context.as_ref().unwrap().Map(
            &staging_resource,
            0,
            D3D11_MAP_READ,
            0,
            Some(&mut mapped),
        )?;
        
        // Calculate the proper buffer size based on screen dimensions
        let pitch = mapped.RowPitch as usize;
        let buffer_size = (self.screen_height as usize) * pitch;
        let src_data = mapped.pData as *const u8;
        
        // Create a complete copy of the buffer
        let mut owned_buffer = Vec::with_capacity(buffer_size);
        owned_buffer.resize(buffer_size, 0);
        
        // Copy data from mapped memory to our vector
        std::ptr::copy_nonoverlapping(src_data, owned_buffer.as_mut_ptr(), buffer_size);
        
        // Cleanup
        self.context.as_ref().unwrap().Unmap(&staging_resource, 0);
        self.duplication.as_ref().unwrap().ReleaseFrame()?;
        
        Ok(owned_buffer)
    }

    // Get screen dimensions
    pub fn get_dimensions(&self) -> (i32, i32) {
        (self.screen_width, self.screen_height)
    }
} 