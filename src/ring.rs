use nalgebra_glm::Vec3;
use crate::vertex::Vertex;

// Build a flat annulus in XY plane centered at origin.
// Returns a vertex array where every 3 vertices form a triangle.
pub fn build_ring(inner_radius: f32, outer_radius: f32, segments: usize) -> Vec<Vertex> {
    let segs = segments.max(3);
    let mut verts: Vec<Vertex> = Vec::with_capacity(segs * 6);
    let n = Vec3::new(0.0, 0.0, 1.0);

    for i in 0..segs {
        let t0 = (i as f32) * std::f32::consts::TAU / (segs as f32);
        let t1 = ((i + 1) as f32) * std::f32::consts::TAU / (segs as f32);

    let (s0, c0) = t0.sin_cos();
    let (s1, c1) = t1.sin_cos();

        let inner0 = Vec3::new(c0 * inner_radius, s0 * inner_radius, 0.0);
        let outer0 = Vec3::new(c0 * outer_radius, s0 * outer_radius, 0.0);
        let inner1 = Vec3::new(c1 * inner_radius, s1 * inner_radius, 0.0);
        let outer1 = Vec3::new(c1 * outer_radius, s1 * outer_radius, 0.0);

        // Triangle 1: outer0, inner0, inner1
        verts.push(Vertex::new(outer0, n));
        verts.push(Vertex::new(inner0, n));
        verts.push(Vertex::new(inner1, n));
        // Triangle 2: outer0, inner1, outer1
        verts.push(Vertex::new(outer0, n));
        verts.push(Vertex::new(inner1, n));
        verts.push(Vertex::new(outer1, n));
    }

    verts
}
