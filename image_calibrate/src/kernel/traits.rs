//pt Accelerate
/// This should probably be sync + send + clone

#[derive(Debug, Default, Clone)]
pub struct AccelerateArgs {
    pub width: usize,
    pub height: usize,
    pub window_size: usize,
    pub scale: u32, // 24.8
}

impl From<(usize, usize)> for AccelerateArgs {
    fn from((width, height): (usize, usize)) -> Self {
        Self {
            width,
            height,
            scale: 1,
            ..std::default::Default::default()
        }
    }
}
impl AccelerateArgs {
    pub fn with_window(mut self, window_size: usize) -> Self {
        self.window_size = window_size;
        self
    }
    pub fn with_scale(mut self, scale: u32) -> Self {
        self.scale = scale;
        self
    }
}

pub trait Accelerate: std::fmt::Debug {
    // The accelerator will already have input and output buffers, and
    // any internal buffers and bindings
    //
    // The accelerator can also already have an encoded command buffer if the input
    // and output buffers are big enough for the whole data
    //
    // If they are not, then a new command buffer may be needed to copy
    // slices of the input buffer to the storage and run the pipeline
    // and copy slices out again
    //
    // This will also have to slice the src_data (if required) into
    // input_buffer sized lumps, and run the whole command buffer many
    // times over
    //
    fn run_shader(
        &self,
        shader: &str,
        args: &AccelerateArgs,
        src_data: Option<&[u32]>,
        out_data: &mut [u32],
    ) -> Result<bool, String>;
}
