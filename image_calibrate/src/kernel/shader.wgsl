@group(0) @binding(0)
var<storage, read_write> out_data: array<f32>; // this is used as both input and output for convenience

@group(0) @binding(1)
var<storage, read> in_data: array<f32>; // this is used as input only

struct KernelArgs {
  width: u32,
  height: u32,
  size: u32,
  scale: f32,
}

@group(0) @binding(2)
var<uniform> kernel_args: KernelArgs;

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

