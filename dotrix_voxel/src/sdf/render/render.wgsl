//! Render the SDF from the compute buffers onto the render buffer
//!
//! This is done as a sort of billboard with a single triangle
//!

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] clip_space: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
) -> VertexOutput {
  var out: VertexOutput;
  out.position = vec4<f32>(position,1.);
  out.clip_space = out.position.xyz; // Save it without the automatic perspective divide etc
  return out;
}

[[group(0), binding(0)]]
var poster_tex: texture_2d<f32>;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
  let clip_space_coords: vec3<f32> = in.clip_space;
  let tex_dimensions: vec2<i32> = textureDimensions(poster_tex, 0);

  let mapped_coords: vec3<f32> = (clip_space_coords + vec3<f32>(1.,1.,0.))/2.;

  let resolution: vec2<f32> = vec2<f32>(f32(tex_dimensions.x), f32(tex_dimensions.y));

  let tex_coords_f: vec2<f32> = (mapped_coords.xy * resolution.xy);
  let tex_coords: vec2<i32> = vec2<i32>(i32(tex_coords_f.x + 0.5), i32(tex_coords_f.y + 0.5));
  let color: vec4<f32> = textureLoad(poster_tex, tex_coords.xy, 0);
  if (color.g < 0.) {
    // No object_id
    discard;
  }

  return vec4<f32>(vec3<f32>(color.r), 1.);
}
