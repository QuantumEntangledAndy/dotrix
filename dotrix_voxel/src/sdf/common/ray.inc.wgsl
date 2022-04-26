//! Common Ray code
//!

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
