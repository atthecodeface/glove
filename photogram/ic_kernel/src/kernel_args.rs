//a Imports

//tp KernelArgs
/// This type must be mappable for accelerators, hence u32 and f32
/// only
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct KernelArgs {
    /// Width of the 'image'
    pub width: u32,
    /// Height of the 'image'
    pub height: u32,
    /// Center (or other) X coordinate if not in the work group
    pub cx: u32,
    /// Center (or other) Y coordinate if not in the work group
    pub cy: u32,
    /// Radius of a circle, window size, etc
    pub size: u32,
    /// Scale factor to apply (depends on kernel)
    pub scale: f32,
}

//ip From<(usize, usize)> for KernelArgs {
impl From<(usize, usize)> for KernelArgs {
    fn from((width, height): (usize, usize)) -> Self {
        Self {
            width: width as u32,
            height: height as u32,
            scale: 1.0,
            ..std::default::Default::default()
        }
    }
}

//ip KernelArgs
impl KernelArgs {
    pub fn with_size(mut self, size: usize) -> Self {
        self.size = size as u32;
        self
    }
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    pub fn with_xy(mut self, (x, y): (usize, usize)) -> Self {
        self.cx = x as u32;
        self.cy = y as u32;
        self
    }
    pub fn size(&self) -> usize {
        self.size as usize
    }
    pub fn width(&self) -> usize {
        self.width as usize
    }
    pub fn height(&self) -> usize {
        self.height as usize
    }
    pub fn scale(&self) -> f32 {
        self.scale
    }
    pub fn dims(&self) -> (usize, usize) {
        (self.width as usize, self.height as usize)
    }
}
