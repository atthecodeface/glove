struct KernelArgs {
    width: u32,
    height: u32,
    cx: u32,
    cy: u32,
    size: u32,
    scale: f32,
}

@group(0) @binding(0)
var<storage, read_write> out_data: array<f32>; // this is used as both input and output for convenience

@group(0) @binding(1)
var<storage, read> in_data: array<f32>; // this is used as input only

@group(0) @binding(2)
var<storage, read> in_data_b: array<f32>; // this is used as input only

@group(0) @binding(3)
var<uniform> kernel_args: KernelArgs;


struct ResultPair {
  ofs: u32,
  value: f32,
  other: f32,
}

// Invoke with a centre x,y of the window
fn window_correlate(src_x:u32, src_y:u32, x:u32, y:u32) -> ResultPair {
    let half_ws = kernel_args.size/2;
    let ws = half_ws * 2;
    let out_of_bounds = x < half_ws || x > kernel_args.width-half_ws ||
        y < half_ws || y > kernel_args.height-half_ws
    ;
    let src_out_of_bounds = src_x< half_ws || src_x> kernel_args.width-half_ws ||
        src_y < half_ws || src_y > kernel_args.height-half_ws
    ;
    var ab = 0.0;
    var a = 0.0;
    var b = 0.0;
    var b2 = 0.0;
    var a2 = 0.0;
    let src_ofs : u32 = select((src_x - half_ws) + (src_y-half_ws) * kernel_args.width, 0u, src_out_of_bounds);
    let cmp_ofs : u32 = select((x - half_ws) + (y-half_ws) * kernel_args.width, 0u, out_of_bounds);
    let center_ofs = cmp_ofs + half_ws + half_ws * kernel_args.width;
    for ( var dy: u32 = 0; dy < ws; dy++ ) {
        var x_src_ofs = src_ofs + dy * kernel_args.width;
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
    // let value = (n * ab - a * b) * (n * ab - a * b) / ( (b2 *n - b*b) * (a2 * n - a * a) );
    // Note that a2 and a are fixed for ALL of the comparison pixels
    // Also, note that if we are doing this on SD/mean windows then b2*n-b*b will be about the same as a2*n-a*a for good pixels, but it willl be about 0 if we are in a no-info spot; so dividing by that is pointless
    // let value = (n * ab - a * b) / (b2 * n - b * b);
    // let value = (n * ab - a * b) / (a2 * n - a * a);

    // This indeed picks up lots of tiny noisy cmp image spots
    // let value = (n * ab - a * b) * (n * ab - a * b) / ( (b2 *n - b*b) * (a2 * n - a * a) );

    // This should be the same pretty much as without the Var(A) on the bottom
    let value = select((n * ab - a * b) / (a2 * n - a * a), 0.0, out_of_bounds || src_out_of_bounds);

    // Show src image
    // let value = select(in_data[cmp_ofs],0.0, x<src_x-half_ws || x>=src_x+half_ws ||
    //  y<src_y-half_ws || y>=src_y+half_ws );
    let value1 = select(in_data_b[cmp_ofs],0.0, x<499-half_ws || x>=499+half_ws ||
                       y<636-half_ws || y>=636+half_ws );

    // let value = ab - (a * b / n);
    // MUST NOT use square as that makes NEGATIVE correlation a POSITIVE correlation
    let other = value * value * value;
    // let other = value ;
   
    return ResultPair ( center_ofs, value, other );
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
@workgroup_size(64,1)
fn compute_vec_sqrt(@builtin(global_invocation_id) global_id: vec3<u32>) {
   // global_id is [x,y,z] for the work id
   out_data[global_id.x] = sqrt(in_data[global_id.x]) * kernel_args.scale;
}

@compute
@workgroup_size(64,1)
fn compute_vec_square(@builtin(global_invocation_id) global_id: vec3<u32>) {
   // global_id is [x,y,z] for the work id
   out_data[global_id.x] = in_data[global_id.x]*in_data[global_id.x]* kernel_args.scale;
}

@compute
@workgroup_size(64,1)
fn compute_window_sum_x(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var sum = 0.0;
    let out_of_bounds = global_id.x < kernel_args.size/2 || global_id.x > kernel_args.width*kernel_args.height-kernel_args.size/2;
    for ( var i: u32 = 0; i < kernel_args.size; i++ ) {
        if !out_of_bounds { sum += in_data[global_id.x + i -kernel_args.size/2]; }
    }
    out_data[global_id.x] = sum* kernel_args.scale;
}

@compute
@workgroup_size(64,1)
fn compute_window_sum_y(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var sum = 0.0;
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



