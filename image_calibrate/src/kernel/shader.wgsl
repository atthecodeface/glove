@group(0) @binding(0)
var<storage, read_write> out_data: array<f32>; // this is used as both input and output for convenience

@group(0) @binding(1)
var<storage, read> in_data: array<f32>; // this is used as input only

struct Globals {
    a_uniform: vec2<f32>,
    another_uniform: vec2<f32>,
}
@group(0) @binding(2)
var<uniform> globals: Globals;

@compute
@workgroup_size(1)
fn vec_sqrt(@builtin(global_invocation_id) global_id: vec3<u32>) {
   // global_id is [x,y,z] for the work id
   out_data[global_id.x] = sqrt(out_data[global_id.x]);
}

@compute
@workgroup_size(1)
fn vec_square(@builtin(global_invocation_id) global_id: vec3<u32>) {
   // global_id is [x,y,z] for the work id
   out_data[global_id.x] = out_data[global_id.x]*out_data[global_id.x];
}
