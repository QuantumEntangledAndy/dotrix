//! Compute the depth map for the sdf
//!
//! Rather than the traditional single map to contain the whole world
//! we are instead breaking the map into chunks. To get a complete
//! map we must pass the screen space multiple times and oversample
//!
//! To allivate this cost of oversample all SDFs have a bounding box
//! which acts as a quick first check
//!
//! We also need to read from and update the current depth map
//! If there is already another object rendered on the depth map
//! that is closer then we don't write to it. Due to limitations of
//! readwrite storage textures we use a ping/pong depth buffer
//!
//! Depths are calculated by circletracing the SDF
//! If an object is found it's depth and object ID is stroed into the
//! pong buffer only if it closer than that in the ping buffer
//!
//! Ping buffer init values are infinie distance with an object of -1.
//! (infinite is just an example it should be set to the far plane distance)
//!
//! We also write the normals to texture because they are used multiple times
//! and it is handy to cache them now
//!


{% include "dotrix_voxel/common/camera.inc.wgsl" %}

// Data for the depth calculation
struct DepthCalc {
  object_id: u32;
  max_iterations: u32;
};
[[group(1), binding(0)]]
var<uniform> u_depth_calc: DepthCalc;

{% include "dotrix_voxel/common/ray.inc.wgsl" %}
{% include "dotrix_voxel/common/obb.inc.wgsl" %}
[[group(1), binding(1)]]
var<uniform> u_bb: OBB;

[[group(2), binding(0)]]
var ping_buffer: texture_2d<f32>;
[[group(2), binding(1)]]
var pong_buffer: texture_storage_2d<rg32float,write>;
[[group(2), binding(2)]]
var normal_buffer: texture_storage_2d<rgba32float,write>;
[[group(2), binding(3)]]
var depth_buffer: texture_storage_2d<rg32float,write>;

{% include "dotrix_voxel/circle_trace/map.inc.wgsl" %}
{% include "dotrix_voxel/circle_trace/accelerated_raytrace.inc.wgsl" %}

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

  let tex_coords: vec2<i32> = vec2<i32>(i32(global_invocation_id.x), i32(global_invocation_id.y));

  var ray: Ray;
  ray.origin = get_ray_origin();
  ray.direction = get_ray_direction(tex_coords, resolution);
  let rdx: vec3<f32> = get_ray_direction(vec2<i32>(tex_coords.x + 1, tex_coords.y), resolution);
  let rdy: vec3<f32> = get_ray_direction(vec2<i32>(tex_coords.x, tex_coords.y + 1), resolution);

  let ray_hit = ray_hit_obb(ray, u_bb);
  if (ray_hit.hit) {
    let current_distance: f32 = textureLoad(ping_buffer, tex_coords, 0).r;
    if (ray_hit.t_in < current_distance) {
      // Check via ray_trace
      var raymarch_input: RaymarchIn;
      raymarch_input.init_t = ray_hit.t_in;
      raymarch_input.max_t = ray_hit.t_out;
      raymarch_input.origin = ray.origin;
      raymarch_input.direction = ray.direction;
      raymarch_input.dx_direction = rdx;
      raymarch_input.dy_direction = rdy;
      raymarch_input.max_iterations = u_depth_calc.max_iterations;
      let raymarch: RaymarchOut = raymarch(raymarch_input);
      if (raymarch.success && raymarch.t < current_distance) {
        let p: vec3<f32> = ray.origin + ray.direction * raymarch.t;


        let result: vec4<f32> = vec4<f32>(raymarch.t, f32(u_depth_calc.object_id), 0., 0.);
        let normal: vec4<f32> = vec4<f32>(map_normal(p), 0.);

        textureStore(pong_buffer, tex_coords, result);
        textureStore(depth_buffer, tex_coords, result);
        textureStore(normal_buffer, tex_coords, normal);

      }
    }
  }


}
