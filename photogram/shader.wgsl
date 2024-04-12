struct KernelArgs {
  width: u32,
  height: u32,
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
    var ab = 0.0;
    var a = 0.0;
    var b = 0.0;
    var b2 = 0.0;
    var a2 = 0.0;
    let src_ofs = x + y * kernel_args.width;
    let cmp_ofs = x + y * kernel_args.width;
    for ( var dy: u32 = 0; dy < ws; dy++ ) {
        var x_src_ofs = src_ofs + (dy-half_ws) * kernel_args.width;
        var x_cmp_ofs = cmp_ofs + (dy-half_ws) * kernel_args.width;
        for ( var dx: u32 = 0; dx < ws; dx++ ) {
            let i_a = in_data[x_src_ofs];
            let i_b = in_data_b[x_cmp_ofs];
            a += i_a;
            b += i_b;
            a2 += i_a * i_a;
            b2 += i_b * i_b;
            ab += i_a * i_b;
            x_src_ofs++;  
            x_cmp_ofs++;  
        }            
    }
    let n = f32(ws*ws);
    let value = (ab - n * a * b) / ( (b2 - n*b) * (a2 - n*a) );
    let other = 0.0;
    return ResultPair ( cmp_ofs, value, other );
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
    let value = a / n;
    let other = (n * a2 - a * a) / ( n * n );
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
@workgroup_size(64,1)
fn compute_window_corr(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let result = window_correlate(global_id.x, global_id.y, global_id.x, global_id.y);
    out_data[result.ofs] = result.value * kernel_args.scale;
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



