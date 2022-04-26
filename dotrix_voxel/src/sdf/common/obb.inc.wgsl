//! Oriented Bounding Box calclation code
//!

// An Oriented Bounding box
struct OBB {
  axis: mat4x4<f32>;
  center: vec4<f32>;
  half_widths: vec4<f32>;
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
