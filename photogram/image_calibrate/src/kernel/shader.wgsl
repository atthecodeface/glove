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


// Invoke with a centre x,y of the window
fn window_correlate(x:u32, y:u32, img_a:array<storage, f32>, img_b:array<storage, f32>, result<storage, f32>) {
    const half_ws = kernel_args.size/2;
    const ws = half_ws * 2;
    let out_of_bounds = x < half_ws || x > kernel_args.width-half_ws ||
        y < half_ws || y > kernel_args.height-half_ws
    ;
    var ab = 0.0;
    var a = 0.0.;
    var b = 0.0.;
    var b2 = 0.0.;
    var a2 = 0.0.;
    const ofs = x + y * kernel_args.width;;
    for ( var dy: u32 = 0; dy < ws; dy++ ) {
        var x_ofs = ofs + (dy-half_ws) * kernel_args.width;
        for ( var dx: u32 = 0; dx < ws; dx++ ) {
            let i_a = img_a[x_ofs+dx];
            let i_b = img_b[x_ofs+dx];
            a += i_a;
            b += i_b;
            a2 += i_ a * i_a;
            b2 += i_ b * i_b;
            ab += i_ a * i_b;
            x_ofs += 1;
        }            
    }
    const n = f32(ws*ws);
    result[ofs] = (ab - n * a * b) / ( (b2 - n*b) * (a2 - n*a) );
}

@compute
@workgroup_size(1)
fn vec_sqrt(@builtin(global_invocation_id) global_id: vec3<u32>) {
   // global_id is [x,y,z] for the work id
   out_data[global_id.x] = sqrt(in_data[global_id.x]) * kernel_args.scale;
}

@compute
@workgroup_size(1)
fn vec_square(@builtin(global_invocation_id) global_id: vec3<u32>) {
   // global_id is [x,y,z] for the work id
   out_data[global_id.x] = in_data[global_id.x]*in_data[global_id.x]* kernel_args.scale;
}

@compute
@workgroup_size(1)
fn window_sum_x(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var sum = 0.0;
    let out_of_bounds = global_id.x < kernel_args.size/2 || global_id.x > kernel_args.width*kernel_args.height-kernel_args.size/2;
    for ( var i: u32 = 0; i < kernel_args.size; i++ ) {
        if !out_of_bounds { sum += in_data[global_id.x + i -kernel_args.size/2]; }
    }
    out_data[global_id.x] = sum* kernel_args.scale;
}

@compute
@workgroup_size(1)
fn window_sum_y(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var sum = 0.0;
    let out_of_bounds =  global_id.x < kernel_args.width * kernel_args.size/2 || global_id.x > kernel_args.width*(kernel_args.height-kernel_args.size/2);
    for ( var i: u32 = 0; i < kernel_args.size; i++ ) {
        if !out_of_bounds {sum += in_data[global_id.x + (i -kernel_args.size/2) * kernel_args.width];}
    }
    out_data[global_id.x] = sum * kernel_args.scale;
}

@compute
@workgroup_size(1)
fn window_corr(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var sum = 0.0;
    let out_of_bounds =  global_id.x < kernel_args.width * kernel_args.size/2 || global_id.x > kernel_args.width*(kernel_args.height-kernel_args.size/2);
    for ( var i: u32 = 0; i < kernel_args.size; i++ ) {
        if !out_of_bounds {sum += in_data[global_id.x + (i -kernel_args.size/2) * kernel_args.width];}
    }
    out_data[global_id.x] = sum * kernel_args.scale;
   const scale = 1.0 / kernel_args.size;
   window_correlate(global_id.x, global_id.y, 1, &in_data, &in_data_b, &out_data);
}


