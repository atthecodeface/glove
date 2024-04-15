// -*- rustic-analyzer-command: echo; rustic-format-on-save-method: none; -*-
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

// Find the maximum value in a region of in_data
//
// The regions is x,y and scale provides the absolute minimum value we care about
fn max_of_region(x:u32, y:u32) -> ResultPair {
    let base_ofs = x + y * kernel_args.width;
    let width : u32 = min(kernel_args.width-x, kernel_args.size);
    let height : u32 = min(kernel_args.height-y, kernel_args.size);
    var max_so_far = kernel_args.scale;
    var best_ofs = 0u;
    var number_above_scale = 0.0;
    for ( var dy: u32 = 0; dy < height; dy++ ) {
        for ( var dx: u32 = 0; dx < width; dx++ ) {
            let img_ofs = base_ofs + dy * kernel_args.width + dx;
            let data = in_data[img_ofs];
            number_above_scale = select(number_above_scale, number_above_scale+1.0, data > kernel_args.scale);
            best_ofs = select(best_ofs, img_ofs, data > max_so_far);
            // max_so_far = max(data, max_so_far);
            max_so_far = select(max_so_far, data, data > max_so_far);
        }
    }
    if max_so_far <= kernel_args.scale {
      return ResultPair ( 0, 0, 0 );
    } else {
      return ResultPair ( best_ofs, max_so_far, number_above_scale );
    }
}

// Reduce the value in a diameter of 'size' around the given x y
//
// This requires that the x and y regions do not overlap between
// different kernels - or rather, if they do it just reduces each one
// perhaps not-atomically, so only down to zero is permitted
fn reduce_value_around_region(x:u32, y:u32) {
    let half_ws = kernel_args.size/2;
    let ws = half_ws * 2;
    let r_2 = f32(half_ws) * f32(half_ws);

    let base_ofs = x + y * kernel_args.width - half_ws - half_ws * kernel_args.width;
    for ( var dy: u32 = 0; dy < ws; dy++ ) {
        let dy_f = f32(dy) - f32(half_ws);
        let dy_2 = dy_f * dy_f;
        for ( var dx: u32 = 0; dx < ws; dx++ ) {
            let out_of_bounds = (x+dx) > kernel_args.width+half_ws ||
                (y+dy) > kernel_args.height+half_ws
                ;

            let img_ofs = base_ofs + dy * kernel_args.width + dx;
            let dx_f = f32(dx) - f32(half_ws);
            let dx_2 = dx_f * dx_f;
            let dr_2 = r_2 - (dx_2 + dy_2);
            let in_circle = dr_2 >= 0;
            let data = in_data_b[img_ofs];
            // Could use dr_2 (which is 0..r_2) to reduce
            // by scale of (1..scale)
            //
            // This implies reduced = data * (1 + dr_2/r_2*(scale-1))
            //
            // We can actually use cos_a for this too, so that becomes
            //
            // reduced = data * (scale + dr_2/r_2 * (cos_a-scale))
            //
            // to make it flat at scale, set cos_a to scale
            //
            // to make it purely square set scale to 1 and cos_a to
            // the scale factor required for the center spot
            // (generally 1)
            let scale = kernel_args.scale + dr_2 / r_2 * (kernel_args.cos_a - kernel_args.scale);
            let reduced = clamp(data * scale, 0.0, 1.0);
            if in_circle && !out_of_bounds{
                out_data[img_ofs] = reduced;
            }
        }
    }
}

// Call this with at *least* (1+width/size) * (1+height/size)
//
// The top-left corner of each image region of size*size contains the
// offset of the entry with the largest value (at least as big as
// scale); the value of that entry; and the number of entries in the
// region above the scale
//
// size must be at least 4, but should really be larger
//
// The effective workgroup size in the Json should be 8*args.size*args.size
@compute
@workgroup_size(256,1)
fn compute_max_of_region(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let regions_per_width = (kernel_args.width + kernel_args.size-1) / kernel_args.size;
    let x = global_id.x % regions_per_width;
    let y = global_id.x / regions_per_width;
    let img_x = x * kernel_args.size;
    let img_y = y * kernel_args.size;
    if img_y < kernel_args.height && img_x < kernel_args.width {
        let result = max_of_region(img_x, img_y);
        out_data[img_x + img_y * kernel_args.width] = bitcast<f32>(result.ofs);
        out_data[img_x + img_y * kernel_args.width + 1] = result.value;
        out_data[img_x + img_y * kernel_args.width + 2] = result.other;
    }
}

// Call this with in_data containing x and y values, and in_data_b
// containing the image (which should also already be in out_data)
//
// in_data has pairs of x,y
//
// size is the region radius to impact
//
// scale is the value to scale the input down to at the edge of the
// region
//
// cos_a should be set to scale to make the whole region scale down by
// the same amount
//
// cos_a should be set to the amount for the center of the region to
// scale down by if a square reduction is required
//
// The effective workgroup size in the Json should be ?
@compute
@workgroup_size(1,1)
fn compute_reduce_value_around_region(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let img_x = u32(in_data[2*global_id.x+0]);
    let img_y = u32(in_data[2*global_id.x+1]);
    reduce_value_around_region(img_x, img_y);
}


