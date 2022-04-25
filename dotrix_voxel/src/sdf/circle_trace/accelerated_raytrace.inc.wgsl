// Given:
// A `map` function that gives the distance to the sufrace from any point
// A direction (`rd`) and origin (`ro`) that represents the ray to be marched
// An intial distance already travelled (`t_in`)
// Ray differentails that represent the direction of neighbouring rays (`rdx`, `rdy`)
// Then:
// Compute the point at which this ray intersects the surface
//
// This implementation uses
// Accelerated raymarching
// https://www.researchgate.net/publication/331547302_Accelerating_Sphere_Tracing
// Which attempts to overstep on the ray in order to reduce the number of steps marched
// on the ray
//
struct RaymarchIn {
  init_t: f32;
  max_t: f32;
  origin: vec3<f32>;
  direction: vec3<f32>;
  dx_direction: vec3<f32>;
  dy_direction: vec3<f32>;
  max_iterations: u32;
};
struct RaymarchOut {
  t: f32;
  success: bool;
};

// Use pixel based cones to get the size of the pizel
// This is used for an early exit, given the directions and
// the distance traveled it computes the approixmate size of a pixel on screen
// If the distance to the surface is less then the returned pixel size then we
// Stop marching
fn pixel_radius(t: f32, direction: vec3<f32>, direction_x: vec3<f32>, direction_y: vec3<f32>) -> f32 {
  let dx: f32 = length(t*(direction_x-direction));
  let dy: f32 = length(t*(direction_y-direction));
  return length(vec2<f32>(dx, dy)) * 0.1; // 10% of pixel size is the cut-off
}

fn raymarch(in: RaymarchIn) -> RaymarchOut {
  let o: vec3<f32> = in.origin;
  let d: vec3<f32> = in.direction;
  let dx: vec3<f32> = in.dx_direction;
  let dy: vec3<f32> = in.dy_direction;

  let STEP_SIZE_REDUCTION: f32 = 0.95;
  let MAX_DISTANCE: f32 = in.max_t;
  let MAX_ITERATIONS: u32 = in.max_iterations;

  var t: f32 = in.init_t;
  var rp: f32 = 0.; // prev
  var rc: f32 = map(o + (t)*d);; // current
  var rn: f32 = t + MAX_DISTANCE * 2.0; // next (set to effectivly infinity)

  var di: f32 = 0.;

  var out: RaymarchOut;
  out.success = false;

  for(var i: u32 = 0u; i < MAX_ITERATIONS && t < MAX_DISTANCE; i = i + 1u)
  {
    di = rc + STEP_SIZE_REDUCTION * rc * max( (di - rp + rc) / (di + rp - rc), 0.6);
    rn = map(o + (t + di)*d);
    if(di > rc + rn) {
      di = rc;
      rn = map(o + (t + di)*d);
    }
    t = t + di;
    out.t = t;
    if(rn < pixel_radius(t, d, dx, dy)) {
      out.success = true;
      return out;
    }

    rp = rc;
    rc = rn;
  }

  return out;
}
