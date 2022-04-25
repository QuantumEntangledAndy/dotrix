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


struct Camera {
  proj_view: mat4x4<f32>;
  static_camera_trans: mat4x4<f32>;
  pos: vec4<f32>;
  screen_resolution: vec2<f32>;
  fov: f32;
};
[[group(0), binding(0)]]
var<uniform> u_camera: Camera;

// Data for the depth calculation
struct DepthCalc {
  object_id: u32;
  max_iterations: u32;
};
[[group(1), binding(0)]]
var<uniform> u_depth_calc: DepthCalc;

// An Oriented Bounding box
struct OBB {
  axis: mat4x4<f32>;
  center: vec4<f32>;
  half_widths: vec4<f32>;
};
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

struct Ray {
  origin: vec3<f32>;
  direction: vec3<f32>;
};

struct RayHit {
  // How far along the ray before it hits the BB
  t_in: f32;
  // How far along the ray before it exits the BB
  t_out: f32;
  // Did it hit the BB
  hit: bool;
};

// https://www.sciencedirect.com/topics/computer-science/oriented-bounding-box
//
// TODO: Optimise for gpu
fn ray_hit_obb(ray: Ray, bb: OBB) -> RayHit {
    var tmin: f32 = -3.4028235e38; // -Inf
    var tmax: f32 = 3.4028235e38; // +Inf
    let EPS = 1e-4;
    var out: RayHit;
    out.hit = false;

    {% for i in range(end=3) %}
      let r: f32 = dot(bb.axis[{{i}}].xyz, bb.center.xyz - ray.origin.xyz);

      // Check for rays parallel to planes
      if (abs(dot(ray.direction, bb.axis[{{i}}].xyz)) < EPS) {
        // Is parallel
        if (-r - bb.half_widths[{{i}}] > 0. || -r + bb.half_widths[{{i}}] > 0.) {
          // No hit
          out.t_in = 0.;
          out.t_out = 0.;
          return out;
        }
      }

      let s: f32 = dot(bb.axis[{{i}}].xyz, ray.direction);
      // Ray nor parallel so find intersect parameters
      var t0: f32 = (r + bb.half_widths[{{i}}]) / s;
      var t1: f32 = (r - bb.half_widths[{{i}}]) / s;

      // Check ordering
      if (t0 > t1) {
        // swap
        let tmp: f32 = t0;
        t0 = t1;
        t1 = tmp;
      }

      if (t0 > tmin) {
        tmin = t0;
      }
      if (t1 < tmax) {
        tmax = t1;
      }
      // Ray misses entirely
      if (tmin > tmax) {
        out.t_in = 0.;
        out.t_out = 1.;
        return out;
      }
      if (tmax < 0.) {
        out.t_in = 1.;
        out.t_out = 0.;
        return out;
      }

    {% endfor %}

    // We have hit
    out.t_in = max(0., tmin);
    out.t_out = max(tmax, out.t_in);
    out.hit = true;
    return out;
}

fn get_ray_origin() -> vec3<f32> {
  return u_camera.pos.xyz;
}

fn get_ray_direction(pixel: vec2<i32>, resolution: vec2<f32>) -> vec3<f32> {
  let pixel_f32: vec2<f32> = vec2<f32>(f32(pixel.x), f32(pixel.y));
  let p: vec2<f32> =  (2.0 * pixel_f32 - resolution.xy)/(resolution.y);
  let z: f32 = -1. / tan(u_camera.fov * 0.5);
  let view_coordinate: vec4<f32> = vec4<f32>(p.x, p.y, z, 1.);
  let world_coordinate: vec4<f32> = u_camera.static_camera_trans * view_coordinate;

  return normalize(world_coordinate.xyz);
}

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
