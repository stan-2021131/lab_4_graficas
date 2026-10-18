mod camera;
mod framebuffer;
mod light;
mod mesh;
mod renderer;

use camera::Camera;
use framebuffer::{Color, Framebuffer};
use light::Light;
use mesh::Mesh;
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm::{Mat4, Vec3};
use renderer::{render_scene, RenderMode};
use std::path::Path;
use std::time::Instant;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 600;

fn main() {
    // Inicialización de ventana con minifb
    let mut window = Window::new(
        "Software Renderer - [1: Solo Malla]",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("No se pudo crear la ventana: {e}");
    });

    window.set_target_fps(60);

    // Framebuffer con soporte para color y Z-Buffer en CPU
    let mut framebuffer = Framebuffer::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    // Carga de la nave desde assets
    let obj_path = Path::new("src/assets/nave.obj");
    let mesh = Mesh::load_from_obj(obj_path).expect("Fallo al cargar el modelo nave.obj");
    println!(
        "Modelo cargado: {} vértices, {} triángulos",
        mesh.vertices.len(),
        mesh.indices.len() / 3
    );

    // Cámara orbital y matriz de modelo
    let mut camera = Camera::new(Vec3::new(0.0, 0.0, 0.0), 10.0);
    let aspect_ratio = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
    let model = Mat4::identity();

    // Fuente de luz direccional (luz ambiental + difusa)
    let light = Light::new(Vec3::new(1.0, 2.0, -3.0), 0.25, 0.85);

    // Configuración de colores para distinguir malla y relleno
    let wireframe_color = Color::new(255, 235, 50);  // Amarillo brillante para la malla
    let fill_base_color = Color::new(211, 63, 63);  // Rojo oscuro para el relleno

    // Modo de render inicial
    let mut current_mode = RenderMode::Wireframe;

    let mut last_time = Instant::now();

    // Ciclo principal de la aplicación
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f32();
        last_time = now;

        // Procesa controles de cámara (WASD, Flechas, Q/E)
        camera.process_input(&window, dt);

        // Alternar modos de render mediante teclas 1, 2, 3 o M
        let mut mode_changed = false;
        if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            current_mode = RenderMode::Wireframe;
            mode_changed = true;
        } else if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            current_mode = RenderMode::WireframeAndSolid;
            mode_changed = true;
        } else if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            current_mode = RenderMode::Solid;
            mode_changed = true;
        } else if window.is_key_pressed(Key::M, KeyRepeat::No) {
            current_mode = current_mode.next();
            mode_changed = true;
        }

        if mode_changed {
            window.set_title(&format!("Software Renderer - [{}]", current_mode.title_name()));
        }

        // Limpiar el buffer con color de espacio profundo
        framebuffer.clear(Color::new(15, 15, 25));

        // Matrices de vista y proyección
        let view = camera.get_view_matrix();
        let projection = camera.get_projection_matrix(aspect_ratio);
        let view_proj = projection * view;

        // Renderizado con rasterizado, Z-Buffer y sombreado
        render_scene(
            &mut framebuffer,
            &mesh,
            &model,
            &view_proj,
            &light,
            current_mode,
            wireframe_color,
            fill_base_color,
        );

        // Volcar el framebuffer a pantalla
        window
            .update_with_buffer(&framebuffer.buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}
