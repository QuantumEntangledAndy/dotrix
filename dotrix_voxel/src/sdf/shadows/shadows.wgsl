//! Compute the shadows
//!
//! The textures are init with 0. meaning no shadow
//! Each ping/pong the shadow values increase
//!
//! We use soft shadows by probing if a shadow ray will almost hit a surface
//!

{% include "dotrix_voxel/common/camera.inc.wgsl" %}

// Data for the ao calculation
struct ShadowCalc {
  max_iterations: u32;
  max_probe: f32;
  k: f32;
};
[[group(1), binding(0)]]
var<uniform> u_shadow_calc: ShadowCalc;

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
var shadow_buffer: texture_storage_2d<r32float,write>;

{% include "dotrix_voxel/shading/lighting.inc.wgsl" %}
{% include "dotrix_voxel/shadows/soft_shadows_closet_approach.inc.wgsl" %}

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
  var penumba: f32 = 0.;

  if (object_id >= 0.) {
    let depth: f32 = depth_buffer_data.r;
    let rd: vec3<f32> = get_ray_direction(tex_coords.xy, resolution);
    let ro: vec3<f32> = get_ray_origin() + rd * depth;


    let light_count: u32 = get_light_count();
    for (var i: u32 = 0u; i<light_count; i = i + 1u) {
      let light_out: LightCalcOutput = calculate_nth_light_ray(i, ro);
      let L: vec3<f32> = light_out.light_direction;
      let nor: vec3<f32> = textureLoad(normal_buffer, depth_coords, 0).xyz;

      let intensity: f32 = dot(light_out.light_direction, nor);
      // If perpendicular don't bother (numerically unstable)
      if (abs(intensity) > 0.1  ) {
        let expanded_bb: OBB = expand_obb(u_bb, 1.1);
        var ray: Ray;
        ray.origin = ro;
        ray.direction = light_out.light_direction;
        let ray_hit: RayHit = ray_hit_obb(ray, expanded_bb);
        if (ray_hit.hit) {
          if (u_shadow_calc.max_probe >= ray_hit.t_in) {
            var ray_in: SoftShadowInput;
            ray_in.origin = ro;
            ray_in.direction = light_out.light_direction;
            ray_in.max_iterations = u_shadow_calc.max_iterations;
            ray_in.min_distance = max(0.01, ray_hit.t_in);
            ray_in.max_distance = min(u_shadow_calc.max_probe, ray_hit.t_out);
            ray_in.k = u_shadow_calc.k;

            let ray_out: SoftShadowResult = softshadow(ray_in);

            penumba = penumba + (1. - clamp(ray_out.radiance, 0., 1.));
          }
        }
      }
    }

  }

  let current_penumba: f32 = textureLoad(ping_buffer, tex_coords, 0).r;
  let new_penumba: f32 = clamp(current_penumba + penumba, 0., 1.);
  let radiance: f32 = 1. - clamp(new_penumba, 0., 1.);
  textureStore(pong_buffer, tex_coords, vec4<f32>(vec3<f32>(new_penumba), 1.));
  textureStore(shadow_buffer, tex_coords, vec4<f32>(vec3<f32>(radiance), 1.));

}
