struct KernelArgs {
    /// Width of the 'image'
     width: u32,
    /// Height of the 'image'
     height: u32,
    /// Center (or other) X coordinate if not in the work group
     cx: u32,
    /// Center (or other) Y coordinate if not in the work group
     cy: u32,
    /// Radius of a circle, window size, etc
     size: u32,
    /// Scale factor to apply (depends on kernel)
     scale: f32,
    /// Rotated cos_a
     cos_a: f32,
    /// Rotated dy
     sin_a: f32,
    /// Width of the source 'image'
     src_width: u32,
    /// Height of the source 'image'
     src_height: u32,
}

struct ResultPair {
  ofs: u32,
  value: f32,
  other: f32,
}

@group(0) @binding(0)
var<uniform> kernel_args: KernelArgs;

@group(0) @binding(1)
var<storage, read_write> out_data: array<f32>; // this is used as both input and output for convenience

@group(0) @binding(2)
var<storage, read> in_data: array<f32>; // this is used as input only

@group(0) @binding(3)
var<storage, read> in_data_b: array<f32>; // this is used as input only


@compute
@workgroup_size(256,1)
fn compute_multiply(@builtin(global_invocation_id) global_id: vec3<u32>) {
let ofs = global_id.x;
 out_data[ofs] = in_data[ofs] * in_data_b[ofs] * kernel_args.scale;
}
