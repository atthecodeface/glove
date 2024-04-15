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


// Invoke with a centre x,y of the window
fn window_correlate(src_x:u32, src_y:u32, x:u32, y:u32) -> ResultPair {
    let half_ws = kernel_args.size/2;
    let ws = half_ws * 2;
    let out_of_bounds = x < half_ws || x > kernel_args.width-half_ws ||
        y < half_ws || y > kernel_args.height-half_ws
    ;
    let src_out_of_bounds = src_x< half_ws || src_x> kernel_args.src_width-half_ws ||
        src_y < half_ws || src_y > kernel_args.src_height-half_ws
    ;
    var ab = 0.0;
    var a = 0.0;
    var b = 0.0;
    var b2 = 0.0;
    var a2 = 0.0;
    let src_ofs : u32 = select((src_x - half_ws) + (src_y-half_ws) * kernel_args.src_width, 0u, src_out_of_bounds);
    let cmp_ofs : u32 = select((x - half_ws) + (y-half_ws) * kernel_args.width, 0u, out_of_bounds);
    let center_ofs = cmp_ofs + half_ws + half_ws * kernel_args.width;
    for ( var dy: u32 = 0; dy < ws; dy++ ) {
        var x_src_ofs = src_ofs + dy * kernel_args.src_width;
        var x_cmp_ofs = cmp_ofs + dy * kernel_args.width;
        for ( var dx: u32 = 0; dx < ws; dx++ ) {
            let i_a = in_data[x_src_ofs + dx];
            let i_b = in_data_b[x_cmp_ofs + dx];
            a += i_a;
            b += i_b;
            a2 += i_a * i_a;
            b2 += i_b * i_b;
            ab += i_a * i_b;
        }            
    }
    // There are ws*ws pixels in our window
    let n = f32(ws*ws);
    // let value_unbounded = (n * ab - a * b) * (n * ab - a * b) / ( (b2 *n - b*b) * (a2 * n - a * a) );
    // Note that a2 and a are fixed for ALL of the comparison pixels
    // Also, note that if we are doing this on SD/mean windows then b2*n-b*b will be about the same as a2*n-a*a for good pixels, but it willl be about 0 if we are in a no-info spot; so dividing by that is pointless
    // let value_unbounded = (n * ab - a * b) / (b2 * n - b * b);
    // let value_unbounded = (n * ab - a * b) / (a2 * n - a * a);

    // This indeed picks up lots of tiny noisy cmp image spots
    // let value_unbounded = (n * ab - a * b) * (n * ab - a * b) / ( (b2 *n - b*b) * (a2 * n - a * a) );
    // let value_unbounded = ab - (a * b / n);

    // This should be the same pretty much as without the Var(A) on the bottom, except the range would
    // not be -1 to 1
    //
    // The range is -1 to 1
    let value_unbounded = (n * ab - a * b) / (a2 * n - a * a);
    let value = select(value_unbounded, 0.0, value_unbounded<0.0 || out_of_bounds || src_out_of_bounds);

    // MUST NOT use square as that makes NEGATIVE correlation a POSITIVE correlation
    let other = value * value * value;
   
    return ResultPair ( center_ofs, value, other );
}

// Invoke with a centre x,y of the window
//
// angle is the rotation anticlockwise of the source
fn window_correlate_arbitrary(src_x:u32, src_y:u32, x:u32, y:u32) -> ResultPair {
    let half_ws = kernel_args.size/2;
    let ws = half_ws * 2;
    let src_half_ws = u32(f32(half_ws) *  (abs(kernel_args.cos_a) + abs(kernel_args.sin_a)));
    let out_of_bounds = x < half_ws || x > kernel_args.width-half_ws ||
        y < half_ws || y > kernel_args.height-half_ws
    ;
    let src_out_of_bounds = src_x< src_half_ws || src_x> kernel_args.src_width-src_half_ws ||
        src_y < src_half_ws || src_y > kernel_args.src_height-src_half_ws
    ;
    var ab = 0.0;
    var a = 0.0;
    var b = 0.0;
    var b2 = 0.0;
    var a2 = 0.0;
    let center_ofs = x + y * kernel_args.width;
    let cmp_ofs : u32 = center_ofs - half_ws - half_ws * kernel_args.width;
    for ( var dy: u32 = 0; dy < ws; dy++ ) {
        var x_cmp_ofs = cmp_ofs + dy * kernel_args.width;
        let row_src_x = f32(src_x) - (f32(dy) - f32(half_ws)) * kernel_args.sin_a;
        let row_src_y = f32(src_y) + (f32(dy) - f32(half_ws)) * kernel_args.cos_a;
        for ( var dx: u32 = 0; dx < ws; dx++ ) {
            let src_x_f = row_src_x + (f32(dx) - f32(half_ws)) * kernel_args.cos_a;
            let src_y_f = row_src_y + (f32(dx) - f32(half_ws)) * kernel_args.sin_a;
            let src_x_u = u32(src_x_f);
            let src_y_u = u32(src_y_f);
            let src_ofs_xy = src_x_u + src_y_u * kernel_args.src_width;
            let i_a = in_data[src_ofs_xy];
            let i_b = in_data_b[x_cmp_ofs + dx];
            a += i_a;
            b += i_b;
            a2 += i_a * i_a;
            b2 += i_b * i_b;
            ab += i_a * i_b;
        }            
    }
    // There are ws*ws pixels in our window
    let n = f32(ws*ws);
    let value_unbounded = max((n * ab - a * b) / (a2 * n - a * a), 0.);
    //  let value_unbounded = max((n * ab - a * b) / sqrt(a2 * n - a * a) / max(0.25*f32(n), sqrt(b2 * n - b * b)), 0.);
    let is_noisy = 1 == 0;
    let value = select(value_unbounded, 0.0, out_of_bounds || src_out_of_bounds || is_noisy );
    // Can use square here as value is >=0
    let other = value * value;
   
    return ResultPair ( center_ofs, value * kernel_args.scale, other * kernel_args.scale );
//    return ResultPair ( center_ofs, in_data_b[center_ofs], other * kernel_args.scale );
}

// Invoke with a centre x,y of the window
fn window_mean_variance(x:u32, y:u32) -> ResultPair {
    let half_ws = kernel_args.size/2;
    let ws = half_ws * 2;
    let out_of_bounds = x < half_ws || x > kernel_args.width-half_ws ||
        y < half_ws || y > kernel_args.height-half_ws
    ;
    var a = 0.0;
    var a2 = 0.0;
    let ofs = x + y * kernel_args.width;
    for ( var dy: u32 = 0; dy < ws; dy++ ) {
        var x_ofs = ofs + (dy-half_ws) * kernel_args.width;
        for ( var dx: u32 = 0; dx < ws; dx++ ) {
            let i_a = in_data[x_ofs+dx];
            a += i_a;
            a2 += i_a * i_a;
            x_ofs++;  
        }            
    }
    let n = f32(ws*ws);
    let value = select(a / n, 0.0, out_of_bounds);
    let other = select((n * a2 - a * a) / ( n * n ), 0.0, out_of_bounds);
    return ResultPair ( ofs, value, other );
}

@compute
@workgroup_size(256,1)
fn compute_vec_sqrt(@builtin(global_invocation_id) global_id: vec3<u32>) {
   // global_id is [x,y,z] for the work id
   out_data[global_id.x] = sqrt(in_data[global_id.x]) * kernel_args.scale;
}

@compute
@workgroup_size(256,1)
fn compute_vec_square(@builtin(global_invocation_id) global_id: vec3<u32>) {
   // global_id is [x,y,z] for the work id
   out_data[global_id.x] = in_data[global_id.x]*in_data[global_id.x]* kernel_args.scale;
}

@compute
@workgroup_size(256,1)
fn compute_window_sum_x(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var sum = 0.0;
    let x = global_id.x % kernel_args.width;
    let y = global_id.x / kernel_args.width;
    let out_of_bounds = global_id.x < kernel_args.size/2 || global_id.x > kernel_args.width*kernel_args.height-kernel_args.size/2;
    for ( var i: u32 = 0; i < kernel_args.size; i++ ) {
        if !out_of_bounds { sum += in_data[global_id.x + i -kernel_args.size/2]; }
    }
    out_data[global_id.x] = sum* kernel_args.scale;
}

@compute
@workgroup_size(256,1)
fn compute_window_sum_y(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var sum = 0.0;
    let x = global_id.x % kernel_args.width;
    let y = global_id.x / kernel_args.width;
    let out_of_bounds =  global_id.x < kernel_args.width * kernel_args.size/2 || global_id.x > kernel_args.width*(kernel_args.height-kernel_args.size/2);
    for ( var i: u32 = 0; i < kernel_args.size; i++ ) {
        if !out_of_bounds {sum += in_data[global_id.x + (i -kernel_args.size/2) * kernel_args.width];}
    }
    out_data[global_id.x] = sum * kernel_args.scale;
}

@compute
@workgroup_size(256,1)
fn compute_window_corr(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cx = kernel_args.cx;
    let cy = kernel_args.cy;
    let x = global_id.x % kernel_args.width;
    let y = global_id.x / kernel_args.width;
    let result = window_correlate(cx, cy, x, y);
    out_data[result.ofs] = result.other * kernel_args.scale;
}

@compute
@workgroup_size(256,1)
fn compute_window_corr_arbitrary(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cx = kernel_args.cx;
    let cy = kernel_args.cy;
    let x = global_id.x % kernel_args.width;
    let y = global_id.x / kernel_args.width;
    let result = window_correlate_arbitrary(cx, cy, x, y);
    out_data[result.ofs] = result.value;
}

@compute
@workgroup_size(256,1)
fn compute_window_mean(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x % kernel_args.width;
    let y = global_id.x / kernel_args.width;
    let result = window_mean_variance(x, y);
    out_data[result.ofs] = result.value * kernel_args.scale;
}

@compute
@workgroup_size(256,1)
fn compute_window_var(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x % kernel_args.width;
    let y = global_id.x / kernel_args.width;
    let result = window_mean_variance(x, y);
    out_data[result.ofs] = result.other * kernel_args.scale;
}

@compute
@workgroup_size(256,1)
fn compute_window_var_scaled(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x % kernel_args.width;
    let y = global_id.x / kernel_args.width;
    let result = window_mean_variance(x, y);
    out_data[result.ofs] = result.other * kernel_args.scale / result.value;
}

@compute
@workgroup_size(256,1)
fn compute_copy(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let out_of_bounds = global_id.x > kernel_args.width*kernel_args.height;
    if !out_of_bounds {
        out_data[global_id.x] = in_data[global_id.x];
    }
}



