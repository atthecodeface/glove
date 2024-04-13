//a Imports

//tp KernelArgs
/// This type must be mappable for accelerators, hence u32 and f32
/// only
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
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
    /// Angle as cos
    pub cos_a: f32,
    /// Angle as cos
    pub sin_a: f32,
    /// Width of the source 'image'
    pub src_width: u32,
    /// Height of the source 'image'
    pub src_height: u32,
}

//ip Default for KernelArgs
impl std::default::Default for KernelArgs {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            cx: 0,
            cy: 0,
            size: 0,
            scale: 1.0,
            cos_a: 1.0,
            sin_a: 0.0,
            src_width: 0,
            src_height: 0,
        }
    }
}
//ip From<(usize, usize)> for KernelArgs {
impl From<(usize, usize)> for KernelArgs {
    fn from((width, height): (usize, usize)) -> Self {
        Self {
            width: width as u32,
            height: height as u32,
            src_width: width as u32,
            src_height: height as u32,
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
    pub fn with_src(mut self, (w, h): (usize, usize)) -> Self {
        self.src_width = w as u32;
        self.src_height = h as u32;
        self
    }
    pub fn with_angle(mut self, angle: f32) -> Self {
        self.cos_a = angle.cos();
        self.sin_a = angle.sin();
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
