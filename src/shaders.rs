use crate::{color::Color, fragment::Fragment, vertex::Vertex, Uniforms};
use nalgebra_glm::{mat4_to_mat3, Vec3, Vec4, Mat3};

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let pos4 = Vec4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0);
    let clip = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * pos4;
    let ndc = Vec4::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w, 1.0);
    let screen = uniforms.viewport_matrix * ndc;

    let model3 = mat4_to_mat3(&uniforms.model_matrix);
    let normal_matrix: Mat3 = model3.transpose().try_inverse().unwrap_or(Mat3::identity());
    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        color: vertex.color,
        transformed_position: Vec3::new(screen.x, screen.y, screen.z),
        transformed_normal,
    }
}


// Estrella: mezcla de colores cálidos modulados por ruido y un brillo hacia el borde
pub fn fragment_star(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let t = uniforms.time * 0.001;
    let p = fragment.vertex_position;

    // Layer 1: base plasma color (core->hot) modulated by low-freq noise (color-affecting)
    let n_base = uniforms.noises[0].get_noise_3d(p.x * 2.2 + t * 0.6, p.y * 2.2, p.z * 2.2 - t * 0.6);
    let core = Color::from_float(1.0, 0.9, 0.35);
    let hot  = Color::from_float(1.0, 0.42, 0.05);
    let mut col = lerp_color(core, hot, ((n_base + 1.0) * 0.5).clamp(0.0, 1.0));

    // Layer 2: sunspots (dark patches) via thresholded noise (color-affecting)
    if uniforms.noises.len() > 1 {
        let n_spot = uniforms.noises[1].get_noise_3d(p.x * 3.0 - t * 0.4, p.y * 3.0, p.z * 3.0 + t * 0.3);
        let mask = smoothstep(0.2, 0.5, n_spot.abs()); // manchas más grandes y notorias
        let dark = Color::from_float(0.18, 0.10, 0.04);
        col = lerp_color(col, dark, mask * 0.65);
    }

    // Layer 3: granulation (bright speckles) with high-freq noise (color-affecting)
    if uniforms.noises.len() > 2 {
        let n_gran = uniforms.noises[2].get_noise_3d(p.x * 24.0, p.y * 24.0, p.z * 24.0);
        let brt = smoothstep(0.35, 0.9, (n_gran + 1.0) * 0.5);
        col = lerp_color(col, Color::from_float(1.0, 0.97, 0.78), brt * 0.35);
    }

    // Layer 4: rim glow (brighter edge) (color-affecting)
    let rim = (1.0 - fragment.normal.z.abs()).clamp(0.0, 1.0).powf(1.3);
    col = lerp_color(col, Color::from_float(1.0, 0.95, 0.8), rim * 0.55);

    let pulse = 0.9 + (t * 0.6).sin() * 0.1;
    col * pulse
}

// Planeta rocoso: capas (base+detalle) para altura y paletas tierra/roca/hielo
pub fn fragment_rocky(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let p = fragment.vertex_position;
    let base = uniforms.noises[0].get_noise_3d(p.x * 0.7, p.y * 0.7, p.z * 0.7);
    let detail = uniforms.noises[1].get_noise_3d(p.x * 2.0, p.y * 2.0, p.z * 2.0);
    let h = ((base * 0.7 + detail * 0.3) + 1.0) * 0.5; // 0..1 height
    let lat = ((p.y + 1.0) * 0.5).clamp(0.0, 1.0); // 0=pole S, 1=pole N, 0.5=ecuador

    // Layer 1: Ocean vs tierra por nivel del mar (color-affecting)
    let sea = 0.52;
    let shore = 0.03;
    let ocean_deep = Color::from_float(0.05, 0.10, 0.30);
    let ocean_shallow = Color::from_float(0.10, 0.45, 0.75);
    let mut col;
    if h < sea {
        let d = ((sea - h) / shore).clamp(0.0, 1.0);
        // cerca de la costa más claro (shallow), profundo más oscuro
        col = lerp_color(ocean_shallow, ocean_deep, d);
    } else {
        // Tierra
        let elev = ((h - sea) / (1.0 - sea)).clamp(0.0, 1.0);

        // Layer 2: Bioma por temperatura (latitud) y humedad (ruido) (color-affecting)
        let moisture = if uniforms.noises.len() > 2 {
            ((uniforms.noises[2].get_noise_3d(p.x * 1.2, p.y * 1.2, p.z * 1.2) + 1.0) * 0.5).clamp(0.0, 1.0)
        } else { 0.5 };
        let temp = 1.0 - (lat - 0.5).abs() * 2.0; // 1 caliente en ecuador, 0 frío en polos
        let desert_factor = smoothstep(0.4, 0.8, (1.0 - moisture) * temp);
        let grass_factor = smoothstep(0.3, 0.7, moisture * temp) * (1.0 - elev * 0.7);

        let desert = Color::from_float(0.73, 0.64, 0.40);
        let grass = Color::from_float(0.20, 0.50, 0.25);
        let dirt  = Color::from_float(0.42, 0.33, 0.26);
        let land_base = lerp_color(dirt, grass, grass_factor);
        let land_biome = lerp_color(land_base, desert, desert_factor * 0.8);

        // Layer 3: Montañas (color-affecting)
        let mountain = Color::from_float(0.62, 0.60, 0.58);
        let m_fac = (elev * 1.3).clamp(0.0, 1.0).powf(1.6);
        let land = lerp_color(land_biome, mountain, m_fac);

        col = land;

        // Layer 4: Hielo/nieve en latitudes altas o gran altitud (color-affecting)
        let snow = Color::from_float(0.96, 0.97, 1.0);
        let polar = smoothstep(0.65, 0.9, (lat - 0.5).abs() * 2.0);
        let snow_alt = smoothstep(0.7, 0.9, elev);
        let s_fac = (polar * 0.7 + snow_alt * 0.6).clamp(0.0, 1.0);
        col = lerp_color(col, snow, s_fac);

        // Extra: nubes delgadas (ligero blanqueo) - usa cuarta textura si hay
        if uniforms.noises.len() > 3 {
            let n_cloud = uniforms.noises[3].get_noise_3d(p.x * 4.0, p.y * 4.0, p.z * 4.0);
            let c = smoothstep(0.55, 0.75, (n_cloud + 1.0) * 0.5);
            col = lerp_color(col, Color::from_float(1.0, 1.0, 1.0), c * 0.20);
        }
    }

    apply_lambert(col, fragment)
}

// Gigante gaseoso: bandas + ruido para perturbar
pub fn fragment_gas(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let p = fragment.vertex_position;

    // Layer 1: bandas base más contrastadas
    let bands = (p.y * 7.0 + uniforms.noises[0].get_noise_3d(p.x * 0.7, p.y * 0.7, p.z * 0.7) * 1.2).sin();
    let t = ((bands + 1.0) * 0.5).clamp(0.0, 1.0);
    let c1 = Color::from_float(0.78, 0.62, 0.48);
    let c2 = Color::from_float(0.96, 0.88, 0.76);
    let mut col = lerp_color(c1, c2, t);

    // Layer 2: acento fino de bandas
    let fine = ((p.y * 24.0 + uniforms.noises[0].get_noise_3d(p.x * 0.5, p.y * 0.5, p.z * 0.5) * 0.6).sin() + 1.0) * 0.5;
    col = lerp_color(col, Color::from_float(1.0, 0.96, 0.88), fine * 0.18);

    // Layer 3: haze atmosférico
    let d = ((uniforms.noises[1].get_noise_3d(p.x * 1.7, p.y * 1.4, p.z * 1.6) + 1.0) * 0.5).clamp(0.0, 1.0);
    col = lerp_color(col, Color::from_float(1.0, 1.0, 1.0), d * 0.12);

    // Layer 4: tormentas más visibles
    if uniforms.noises.len() > 2 {
        let s = uniforms.noises[2].get_noise_3d(p.x * 0.9 + 1.3, p.y * 0.7 - 0.7, p.z * 0.9);
        let mask = smoothstep(0.5, 0.8, s.abs());
        col = lerp_color(col, Color::from_float(0.30, 0.27, 0.25), mask * 0.45);
    }

    apply_lambert(col, fragment)
}

// Luna: gris con variación de cráteres
pub fn fragment_moon(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let p = fragment.vertex_position;
    let n1 = ((p.x * 2.0 + p.y * 2.0 + p.z * 2.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    // si hay ruido disponible, úsalo, sino usa n1
    let n = if !uniforms.noises.is_empty() {
        let v = uniforms.noises[0].get_noise_3d(p.x * 1.2, p.y * 1.2, p.z * 1.2);
        ((v + 1.0) * 0.5).clamp(0.0, 1.0)
    } else { n1 };
    let base = Color::from_float(0.65, 0.65, 0.67);
    let dark = Color::from_float(0.25, 0.25, 0.27);
    let col = lerp_color(dark, base, n);
    apply_lambert(col, fragment)
}

// Anillos gaseosos: bandas radiales en el plano XY del modelo
pub fn fragment_ring(fragment: &Fragment, _uniforms: &Uniforms) -> Color {
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let r = (x * x + y * y).sqrt();
    let band = (r * 25.0).sin();
    let t = ((band + 1.0) * 0.5).clamp(0.0, 1.0);
    let c1 = Color::from_float(0.75, 0.70, 0.62);
    let c2 = Color::from_float(0.55, 0.50, 0.42);
    let base = lerp_color(c1, c2, t * 0.9);
    // iluminación simple con normal
    let n = fragment.normal.normalize();
    let l = Vec3::new(0.0, 0.0, 1.0);
    let diff = n.dot(&l).max(0.2);
    base * (0.2 + diff * 0.8)
}

fn apply_lambert(base: Color, fragment: &Fragment) -> Color {
    let light_pos = Vec3::new(0.0, 0.0, 20.0);
    let l = (light_pos - fragment.vertex_position).normalize();
    let n = fragment.normal.normalize();
    let diff = n.dot(&l).max(0.0);
    let ambient = 0.2;
    base * (ambient + diff * 0.8)
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: (a.r as f32 + (b.r as f32 - a.r as f32) * t).round() as u8,
        g: (a.g as f32 + (b.g as f32 - a.g as f32) * t).round() as u8,
        b: (a.b as f32 + (b.b as f32 - a.b as f32) * t).round() as u8,
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
