//! Compute the ambient occulsion
//!
//! This is done by probing the space around a point
//! to determine how full it is
//!
//!

{% include "dotrix_voxel/common/camera.inc.wgsl" %}

// Data for the ao calculation
struct AoCalc {
  samples: u32;
  steps: u32;
  step_size: f32;
};
[[group(1), binding(0)]]
var<uniform> u_ao_calc: AoCalc;

{% include "dotrix_voxel/common/ray.inc.wgsl" %}
{% include "dotrix_voxel/common/obb.inc.wgsl" %}
[[group(1), binding(1)]]
var<uniform> u_bb: OBB;

[[group(1), binding(2)]]
var depth_buffer: texture_2d<f32>;
[[group(1), binding(3)]]
var normal_buffer: texture_2d<f32>;

{% include "dotrix_voxel/circle_trace/map.inc.wgsl" %}

[[group(2), binding(0)]]
var ping_buffer: texture_2d<f32>;
[[group(2), binding(1)]]
var pong_buffer: texture_storage_2d<r32float,write>;
[[group(2), binding(2)]]
var ao_buffer: texture_storage_2d<r32float,write>;

{% include "dotrix_voxel/ao/hemisphere_ambient_occulsion.inc.wgsl" %}

[[stage(compute), workgroup_size(16, 16)]]
fn cs_main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let dimensions: vec2<i32> = textureDimensions(ping_buffer);
  let resolution: vec2<f32> = vec2<f32>(f32(dimensions.x), f32(dimensions.y));

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

  // Coords for this calc
  let tex_coords: vec2<i32> = vec2<i32>(i32(global_invocation_id.x), i32(global_invocation_id.y));

  // Coords for depth/normal buffer
  let dimensions_depth: vec2<i32> = textureDimensions(depth_buffer);
  let depth_coords: vec2<i32> = vec2<i32>(
    tex_coords.x * dimensions_depth.x / dimensions.x,
    tex_coords.y * dimensions_depth.y / dimensions.y,
  );
  let depth_buffer_data = textureLoad(depth_buffer, depth_coords, 0);

  let object_id: f32 = depth_buffer_data.g;
  var ao: f32 = 1.;

  if (object_id >= 0.) {
    let depth: f32 = depth_buffer_data.r;

    let rd: vec3<f32> = get_ray_direction(tex_coords.xy, resolution);
    let ro: vec3<f32> = get_ray_origin() + rd * depth;

    let N: vec3<f32> = textureLoad(normal_buffer, depth_coords.xy, 0).xyz;

    var ray_in: AoInput;
    ray_in.origin = ro;
    ray_in.direction = N;
    ray_in.samples = u_ao_calc.samples;
    ray_in.steps = u_ao_calc.steps;
    ray_in.ao_step_size = u_ao_calc.step_size;

    ao = 1. - clamp(0., .1, ambient_occlusion(ray_in).ao);
  }

  textureStore(ao_buffer, tex_coords, vec4<f32>(ao, ao, ao, 1.));

}
