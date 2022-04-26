struct Camera {
  proj_view: mat4x4<f32>;
  static_camera_trans: mat4x4<f32>;
  pos: vec4<f32>;
  screen_resolution: vec2<f32>;
  fov: f32;
};
[[group({{ dotrix_voxel_camera_group }}), binding({{ dotrix_voxel_camera_binding }})]]
var<uniform> u_camera: Camera;

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
