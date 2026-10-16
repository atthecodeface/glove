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


/// Calculate Sum(data[cxy + (r,theta)]*e^i.n.theta)
fn dct_n(x:u32, y:u32, n:u32, sc:f32) -> ResultPair {
    let radius = kernel_args.size;
    let out_of_bounds = x < radius || x >= kernel_args.width-radius ||
        y < radius || y >= kernel_args.height-radius
    ;
    var r = 0.0;
    var i = 0.0;
    let ofs = i32(x + y * kernel_args.width);
    for ( var theta: u32 = 0; theta < 32; theta++ ) {
        let angle = f32(theta) * (6.28/32.0);
        let dx = i32(round(f32(radius) / sc * cos(angle)));
        let dy = i32(round(f32(radius) / sc * sin(angle)));
        let dxy = dx + dy*i32(kernel_args.width);
        let i_a = in_data[ofs+dxy];// * 2.0-1.0;
        r += i_a * cos(f32(n)*angle);
        i += i_a * sin(f32(n)*angle);
    }
    let value = select(r/32.0, 0.0, out_of_bounds);
    let other = select(i/32.0, 0.0, out_of_bounds);
    return ResultPair ( u32(ofs), value, other );
}

@compute
@workgroup_size(256,1)
fn compute_dct_2(@builtin(global_invocation_id) global_id: vec3<u32>) {
let x = global_id.x % kernel_args.width;
let y = global_id.x / kernel_args.width;
let result  = dct_n(x, y, 1, 1.0);
let result2 = dct_n(x, y, 1, 2.0);
let power2 = result.value*result.value+result.other*result.other;
let power22 = result2.value*result2.value+result2.other*result2.other;
out_data[result.ofs] = 0.03*-log2(power2);
out_data[result.ofs] = abs((power2-power22) * 10.0);
}
