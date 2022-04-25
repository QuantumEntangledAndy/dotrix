//! This clears the texture data to the init values
//!
//! This is run prior to the depth computes on the gpu rather than
//! a transfer from cpu to gpu
//!

[[group(0), binding(0)]]
var ping_buffer: texture_storage_2d<rg32float,write>;
[[group(0), binding(1)]]
var pong_buffer: texture_storage_2d<rg32float,write>;
[[group(0), binding(2)]]
var normal_buffer: texture_storage_2d<rgba32float,write>;
[[group(0), binding(3)]]
var depth_buffer: texture_storage_2d<rg32float,write>;

[[stage(compute), workgroup_size(16, 16)]]
fn cs_main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let dimensions: vec2<i32> = textureDimensions(ping_buffer);

  let total_x = u32(dimensions.x);
  let index_x = global_invocation_id.x;
  if (index_x >= total_x) {
    return;
  }
  let total_y = u32(dimensions.y);
  let index_y = global_invocation_id.y;
  if (index_y >= total_y) {
    return;
  }

  let infinity: f32 = 3.4028235e38; // +Inf
  let tex_coords: vec2<i32> = vec2<i32>(i32(global_invocation_id.x), i32(global_invocation_id.y));
  textureStore(ping_buffer, tex_coords, vec4<f32>(infinity, -1.,0., 0.));
  textureStore(pong_buffer, tex_coords, vec4<f32>(infinity, -1.,0., 0.));
  textureStore(depth_buffer, tex_coords, vec4<f32>(infinity, -1.,0., 0.));
  textureStore(normal_buffer, tex_coords, vec4<f32>(0.));
}
