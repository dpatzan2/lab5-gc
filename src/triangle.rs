use crate::{color::Color, fragment::Fragment, vertex::Vertex};
use nalgebra_glm::{dot, Vec2, Vec3};

fn edge(a: &Vec3, b: &Vec3, c: &Vec3) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let a = v1.transformed_position;
    let b = v2.transformed_position;
    let c = v3.transformed_position;

    let min_x = a.x.min(b.x).min(c.x).floor() as i32;
    let min_y = a.y.min(b.y).min(c.y).floor() as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil() as i32;
    let max_y = a.y.max(b.y).max(c.y).ceil() as i32;

    let light_dir = Vec3::new(0.0, 0.0, 1.0);
    let area = edge(&a, &b, &c);
    if area.abs() < 1e-6 { return fragments; }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
            let w1 = edge(&b, &c, &p) / area;
            let w2 = edge(&c, &a, &p) / area;
            let w3 = edge(&a, &b, &p) / area;
            if w1 >= 0.0 && w2 >= 0.0 && w3 >= 0.0 {
                let normal = (v1.transformed_normal * w1 + v2.transformed_normal * w2 + v3.transformed_normal * w3).normalize();
                let intensity = dot(&normal, &light_dir).max(0.0);
                let depth = a.z * w1 + b.z * w2 + c.z * w3;
                let vertex_position = v1.position * w1 + v2.position * w2 + v3.position * w3;
                fragments.push(Fragment::new(Vec2::new(x as f32, y as f32), depth, normal, intensity, vertex_position));
            }
        }
    }

    fragments
}
