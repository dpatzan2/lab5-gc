mod color;
mod framebuffer;
mod fragment;
mod obj;
mod shaders;
mod triangle;
mod vertex;
mod ring;

use color::Color;
use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};
use framebuffer::Framebuffer;
use image::{ImageBuffer, Rgb};
use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{look_at, perspective, Mat4, Vec2, Vec3, Vec4};
use obj::Obj;
use shaders::{fragment_star, fragment_gas, fragment_rocky, fragment_moon, fragment_ring, vertex_shader};
use ring::build_ring;
use triangle::triangle;
use vertex::Vertex;

pub struct Uniforms<'a> {
    pub model_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub viewport_matrix: Mat4,
    pub time: f32,
    pub noises: Vec<&'a FastNoiseLite>,
}

fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(w: f32, h: f32) -> Mat4 {
    perspective(45f32.to_radians(), w / h, 0.1, 1000.0)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0,
        0.0,
        0.0,
        width / 2.0,
        0.0,
        -height / 2.0,
        0.0,
        height / 2.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation_y: f32) -> Mat4 {
    let (s, c) = rotation_y.sin_cos();
    let rot_y = Mat4::new(
        c, 0.0, s, 0.0, 0.0, 1.0, 0.0, 0.0, -s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
    let transform = Mat4::new(
        scale,
        0.0,
        0.0,
        translation.x,
        0.0,
        scale,
        0.0,
        translation.y,
        0.0,
        0.0,
        scale,
        translation.z,
        0.0,
        0.0,
        0.0,
        1.0,
    );
    transform * rot_y
}

fn create_model_matrix_euler(translation: Vec3, scale: f32, rot_x: f32, rot_y: f32, rot_z: f32) -> Mat4 {
    let (sx, cx) = rot_x.sin_cos();
    let (sy, cy) = rot_y.sin_cos();
    let (sz, cz) = rot_z.sin_cos();

    let rot_x_m = Mat4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, cx, -sx, 0.0,
        0.0, sx, cx, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );
    let rot_y_m = Mat4::new(
        cy, 0.0, sy, 0.0,
        0.0, 1.0, 0.0, 0.0,
        -sy, 0.0, cy, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );
    let rot_z_m = Mat4::new(
        cz, -sz, 0.0, 0.0,
        sz, cz, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let scale_t = Mat4::new(
        scale, 0.0, 0.0, translation.x,
        0.0, scale, 0.0, translation.y,
        0.0, 0.0, scale, translation.z,
        0.0, 0.0, 0.0, 1.0,
    );

    scale_t * rot_z_m * rot_y_m * rot_x_m
}

fn create_noise_fbmn(seed: i32, freq: f32, octaves: i32) -> FastNoiseLite {
    let mut n = FastNoiseLite::with_seed(seed);
    n.set_noise_type(Some(NoiseType::Perlin));
    n.set_fractal_type(Some(FractalType::FBm));
    n.set_fractal_octaves(Some(octaves));
    n.set_frequency(Some(freq));
    n
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    shader_fn: fn(&fragment::Fragment, &Uniforms) -> Color,
) {
    // Vertex stage
    let mut transformed = Vec::with_capacity(vertex_array.len());
    for v in vertex_array {
        transformed.push(vertex_shader(v, uniforms));
    }

    // Assembly
    let mut tris = Vec::new();
    for i in (0..transformed.len()).step_by(3) {
        if i + 2 < transformed.len() {
            tris.push([transformed[i].clone(), transformed[i + 1].clone(), transformed[i + 2].clone()]);
        }
    }

    // Raster
    let mut fragments = Vec::new();
    for tri in &tris {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    // Fragment stage
    for frag in fragments {
        let x = frag.position.x as usize;
        let y = frag.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            let color = shader_fn(&frag, uniforms).to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x as i32, y as i32, frag.depth);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Window
    let w = 800usize;
    let h = 800usize;
    let mut window = Window::new("Lab5 - Planetas", w, h, WindowOptions::default())?;
    let mut fb = Framebuffer::new(w, h);

    // Camera
    let eye = Vec3::new(0.0, 6.0, 22.0);
    let center = Vec3::new(0.0, 0.0, 0.0);
    let up = Vec3::new(0.0, 1.0, 0.0);

    // Matrices
    let projection = create_perspective_matrix(w as f32, h as f32);
    let viewport = create_viewport_matrix(w as f32, h as f32);

    // Load sphere model
    let args: Vec<String> = std::env::args().collect();
        let sphere_path = if args.len() > 1 {
            args[1].clone()
        } else {
            "assets/models/sphere.obj".to_string()
        };
    let obj = Obj::load(&sphere_path)?;
    let sphere_vertices = obj.get_vertex_array();

    // Noises per shader
    // Star: base, spots, granulation
    let star_base = create_noise_fbmn(42, 0.005, 6);
    let star_spots = create_noise_fbmn(43, 0.02, 5);
    let star_gran  = create_noise_fbmn(44, 0.08, 4);

    // Rocky: base, detail, biome, clouds
    let rocky_base = create_noise_fbmn(7, 1.0, 5);
    let rocky_detail = create_noise_fbmn(8, 3.0, 3);
    let rocky_biome = create_noise_fbmn(9, 0.6, 3);
    let rocky_clouds = create_noise_fbmn(10, 0.9, 5);

    // Gas: bands, detail, storms
    let gas_bands = create_noise_fbmn(99, 2.0, 2);
    let gas_detail = create_noise_fbmn(100, 1.2, 3);
    let gas_storms = create_noise_fbmn(101, 0.9, 4);

    let mut time = 0.0f32;
    let mut mode = 0; // 0 = todos, 1=estrella, 2=rocoso, 3=gaseoso
    let mut rotation = 0.0f32;
    let mut animate_orbits = false;

    // Precompute ring geometry (unit annulus in XY)
    let ring_vertices = build_ring(1.2, 2.0, 64);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        time += 16.0; // ms aprox
        rotation += 0.01;
        fb.clear(0x000000);

        let view = create_view_matrix(eye, center, up);

        // orbital positions
        let tsec = time * 0.001;
        let mut star_pos = Vec3::new(-8.0, 0.0, 0.0);
        let mut rocky_pos = if animate_orbits {
            let a = tsec * 0.8;
            star_pos + Vec3::new(a.cos() * 6.0, 0.0, a.sin() * 6.0)
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        };
        let mut gas_pos = if animate_orbits {
            let a = tsec * 0.4;
            star_pos + Vec3::new(a.cos() * 12.0, 0.0, a.sin() * 12.0)
        } else {
            Vec3::new(8.0, 0.0, 0.0)
        };

        // default scales
        let mut star_scale = 2.4f32;
        let mut rocky_scale = 2.0f32;
        let mut gas_scale = 2.6f32;
        let mut ring_scale = gas_scale;
        let mut moon_scale = 0.6f32;

        // focus zoom-in when selecting a single body
        match mode {
            1 => { // star focus
                star_pos = Vec3::new(0.0, 0.0, 0.0);
                star_scale = 3.6;
            }
            2 => { // rocky focus
                rocky_pos = Vec3::new(0.0, 0.0, 0.0);
                rocky_scale = 3.0;
                moon_scale = 0.8;
            }
            3 => { // gas focus
                gas_pos = Vec3::new(0.0, 0.0, 0.0);
                gas_scale = 3.6;
                ring_scale = gas_scale;
            }
            _ => {}
        }

        // Estrella
        if mode == 0 || mode == 1 {
            let uniforms_star = Uniforms {
                model_matrix: create_model_matrix(star_pos, star_scale, rotation),
                view_matrix: view,
                projection_matrix: projection,
                viewport_matrix: viewport,
                time,
                noises: vec![&star_base, &star_spots, &star_gran],
            };
            render(&mut fb, &uniforms_star, &sphere_vertices, fragment_star);
        }

        // Rocoso
        if mode == 0 || mode == 2 {
            let uniforms_rocky = Uniforms {
                model_matrix: create_model_matrix(rocky_pos, rocky_scale, -rotation * 0.3),
                view_matrix: view,
                projection_matrix: projection,
                viewport_matrix: viewport,
                time,
                noises: vec![&rocky_base, &rocky_detail, &rocky_biome, &rocky_clouds],
            };
            render(&mut fb, &uniforms_rocky, &sphere_vertices, fragment_rocky);

            // Moon orbiting rocky planet (always visible)
            let a = tsec * 2.5;
            let moon_center = if animate_orbits { rocky_pos } else { Vec3::new(0.0, 0.0, 0.0) };
            let orbit_r = if mode == 2 { 2.2 } else { 3.2 };
            let moon_pos = moon_center + Vec3::new(a.cos() * orbit_r, 0.5 * (a * 0.7).sin(), a.sin() * orbit_r);
            let uniforms_moon = Uniforms {
                model_matrix: create_model_matrix(moon_pos, moon_scale, rotation * 0.5),
                view_matrix: view,
                projection_matrix: projection,
                viewport_matrix: viewport,
                time,
                noises: vec![&rocky_detail],
            };
            render(&mut fb, &uniforms_moon, &sphere_vertices, fragment_moon);
        }

        // Gaseoso
        if mode == 0 || mode == 3 {
            let uniforms_gas = Uniforms {
                model_matrix: create_model_matrix(gas_pos, gas_scale, rotation * 0.8),
                view_matrix: view,
                projection_matrix: projection,
                viewport_matrix: viewport,
                time,
                noises: vec![&gas_bands, &gas_detail, &gas_storms],
            };
            render(&mut fb, &uniforms_gas, &sphere_vertices, fragment_gas);

            // Rings around gas giant (tilted ring in XZ plane) - always visible
            let ring_rot_x = -std::f32::consts::FRAC_PI_2 * 0.9; // slight tilt
            let uniforms_ring = Uniforms {
                model_matrix: create_model_matrix_euler(gas_pos, ring_scale, ring_rot_x, rotation * 0.2, 0.0),
                view_matrix: view,
                projection_matrix: projection,
                viewport_matrix: viewport,
                time,
                noises: vec![],
            };
            render(&mut fb, &uniforms_ring, &ring_vertices, fragment_ring);
        }

        window.update_with_buffer(&fb.buffer, w, h)?;

        // Keys
        if window.is_key_pressed(Key::Key1, minifb::KeyRepeat::No) {
            mode = 1;
        }
        if window.is_key_pressed(Key::Key2, minifb::KeyRepeat::No) {
            mode = 2;
        }
        if window.is_key_pressed(Key::Key3, minifb::KeyRepeat::No) {
            mode = 3;
        }
        if window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) {
            mode = 0;
        }
        if window.is_key_pressed(Key::O, minifb::KeyRepeat::No) {
            animate_orbits = !animate_orbits;
        }
        if window.is_key_pressed(Key::S, minifb::KeyRepeat::No) {
            let mut img = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(w as u32, h as u32);
            for y in 0..h {
                for x in 0..w {
                    let px = fb.buffer[y * w + x];
                    let r = ((px >> 16) & 0xFF) as u8;
                    let g = ((px >> 8) & 0xFF) as u8;
                    let b = (px & 0xFF) as u8;
                    img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
                }
            }
            let _ = img.save("screenshot.png");
        }
    }

    Ok(())
}

