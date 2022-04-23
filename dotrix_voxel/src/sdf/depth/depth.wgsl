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
//! that is closer than we don't write to it. Due to limitations of
//! readwrite storage textures we instead use a ping/pong depth buffer
//!
//! Depths are calculated by circletracing the SDF
//! If an object is found it's depth and object ID is stroed into the
//! pong buffer if it closer than that in the ping buffer
//!
//! Ping buffer init values are 4000. distance with an object of -1.
//! (4000. is just an example it should be set to the far plane distance)
//!

struct Ray {
  origin: vec3<f32>;
  direction: vec3<f32>;
};

// An axis aligned BB
struct AABB {
  min: vec3<f32>;
  max: vec3<f32>;
}

// A BB oriented with a transform matrix
struct OBB {
  min: vec3<f32>;
  max: vec3<f32>;
  // Transform should contain translation and rotation but NO scale
  // If it contains scale the min/max should be adjusted instead
  // This is to avoid issues with the Ray direction transformations
  transform: mat4x4<f32>;
  inv_transform: mat4x4<f32>;
};

struct RayHit {
  // How far along the ray before it hits the BB
  t_in: f32;
  // How far along the ray before it exits the BB
  t_out: f32;
  // Did it hit the BB
  hit: bool;
}

fn ray_hit_aabb(ray: Ray, bb: AABB) -> RayHit {

}

fn ray_hit_obb(ray: Ray, bb: OBB) -> RayHit {

}



fn get_ray_origin() -> vec3<f32> {
  return u_camera.pos.xyz;
}

fn get_ray_direction(pixel: vec2<u32>, resolution: vec2<f32>) -> vec3<f32> {
  let pixel_f32: vec2<f32> = vec2<f32>(f32(pixel.x), f32(pixel.y));
  let p: vec2<f32> =  (2.0 * pixel_f32 - resolution.xy)/(resolution.y);
  let z: f32 = -1. / tan(u_camera.fov * 0.5);
  let view_coordinate: vec4<f32> = vec4<f32>(p.x, p.y, z, 1.);
  let world_coordinate: vec4<f32> = u_camera.static_camera_trans * view_coordinate;

  return normalize(world_coordinate.xyz);
}

[[stage(compute), workgroup_size(16, 16)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
  let dimensions: vec2<i32> = textureDimensions(ping_tex);
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
  textureStore(pong_tex, tex_coords, vec4<f32>(0., 0., 0., 1.));
}
