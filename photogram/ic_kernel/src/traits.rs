use crate::KernelArgs;

//pt Accelerate
/// This should probably be sync + send + clone
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
        args: &KernelArgs,
        work_items: usize,
        src_data: Option<&[f32]>,
        out_data: &mut [f32],
    ) -> Result<bool, String>;
}
