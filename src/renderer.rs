use crate::framebuffer::{Color, Framebuffer};
use crate::light::Light;
use crate::mesh::Mesh;
use nalgebra_glm::{Mat4, Vec3, Vec4};

// Modos de visualización soportados
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Wireframe,
    WireframeAndSolid,
    Solid,
}

impl RenderMode {
    // Alterna secuencialmente entre los modos disponibles
    pub fn next(self) -> Self {
        match self {
            Self::Wireframe => Self::WireframeAndSolid,
            Self::WireframeAndSolid => Self::Solid,
            Self::Solid => Self::Wireframe,
        }
    }

    // Nombre legible para mostrar en la barra de título
    pub fn title_name(self) -> &'static str {
        match self {
            Self::Wireframe => "1: Solo Malla (Wireframe)",
            Self::WireframeAndSolid => "2: Malla con Relleno",
            Self::Solid => "3: Solo Relleno (Solid)",
        }
    }
}

// Vértice proyectado con coordenadas de pantalla (x, y) y profundidad (z)
#[derive(Clone, Copy)]
struct ScreenVertex {
    x: f32,
    y: f32,
    z: f32,
}

// Proyecta un vector en espacio de clip a coordenadas de viewport en píxeles
#[inline(always)]
fn clip_to_screen(clip: &Vec4, width: usize, height: usize) -> ScreenVertex {
    let inv_w = 1.0 / clip.w;
    let ndc_x = clip.x * inv_w;
    let ndc_y = clip.y * inv_w;
    let ndc_z = clip.z * inv_w;

    let x = (ndc_x + 1.0) * 0.5 * (width as f32);
    let y = (1.0 - (ndc_y + 1.0) * 0.5) * (height as f32);

    ScreenVertex { x, y, z: ndc_z }
}

// Rasteriza un triángulo sólido rellenando píxeles usando coordenadas baricéntricas y Z-Buffer
fn rasterize_triangle(
    framebuffer: &mut Framebuffer,
    v0: ScreenVertex,
    v1: ScreenVertex,
    v2: ScreenVertex,
    color_u32: u32,
) {
    let min_x = (v0.x.min(v1.x).min(v2.x).floor() as isize)
        .max(0)
        .min(framebuffer.width as isize - 1);
    let max_x = (v0.x.max(v1.x).max(v2.x).ceil() as isize)
        .max(0)
        .min(framebuffer.width as isize - 1);
    let min_y = (v0.y.min(v1.y).min(v2.y).floor() as isize)
        .max(0)
        .min(framebuffer.height as isize - 1);
    let max_y = (v0.y.max(v1.y).max(v2.y).ceil() as isize)
        .max(0)
        .min(framebuffer.height as isize - 1);

    if min_x > max_x || min_y > max_y {
        return;
    }

    let denom = (v1.y - v2.y) * (v0.x - v2.x) + (v2.x - v1.x) * (v0.y - v2.y);
    if denom.abs() < 1e-5 {
        return;
    }
    let inv_denom = 1.0 / denom;

    let width = framebuffer.width;

    for y in min_y..=max_y {
        let py = y as f32 + 0.5;
        let row_offset = y as usize * width;

        for x in min_x..=max_x {
            let px = x as f32 + 0.5;

            let w0 = ((v1.y - v2.y) * (px - v2.x) + (v2.x - v1.x) * (py - v2.y)) * inv_denom;
            let w1 = ((v2.y - v0.y) * (px - v2.x) + (v0.x - v2.x) * (py - v2.y)) * inv_denom;
            let w2 = 1.0 - w0 - w1;

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let z = w0 * v0.z + w1 * v1.z + w2 * v2.z;
                let idx = row_offset + x as usize;
                framebuffer.draw_pixel_depth(idx, z, color_u32);
            }
        }
    }
}

// Renderiza la escena 3D aplicando transformación, rasterizado, sombreado y wireframe
pub fn render_scene(
    framebuffer: &mut Framebuffer,
    mesh: &Mesh,
    model: &Mat4,
    view_proj: &Mat4,
    light: &Light,
    mode: RenderMode,
    wireframe_color: Color,
    fill_base_color: Color,
) {
    let mvp = view_proj * model;
    let width = framebuffer.width;
    let height = framebuffer.height;

    // Transforma los vértices a espacio de pantalla y guarda posición en mundo
    let projected: Vec<ScreenVertex> = mesh
        .vertices
        .iter()
        .map(|v| {
            let clip = mvp * Vec4::new(v.x, v.y, v.z, 1.0);
            clip_to_screen(&clip, width, height)
        })
        .collect();

    let world_vertices: Vec<Vec3> = mesh
        .vertices
        .iter()
        .map(|v| {
            let w = model * Vec4::new(v.x, v.y, v.z, 1.0);
            Vec3::new(w.x, w.y, w.z)
        })
        .collect();

    let has_fill = mode == RenderMode::Solid || mode == RenderMode::WireframeAndSolid;
    let has_wireframe = mode == RenderMode::Wireframe || mode == RenderMode::WireframeAndSolid;

    // Iteración sobre los triángulos del modelo
    for triangle in mesh.indices.chunks_exact(3) {
        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;

        let sv0 = projected[i0];
        let sv1 = projected[i1];
        let sv2 = projected[i2];

        // Relleno de caras con iluminación plana
        if has_fill {
            let w0 = world_vertices[i0];
            let w1 = world_vertices[i1];
            let w2 = world_vertices[i2];

            let edge1 = w1 - w0;
            let edge2 = w2 - w0;
            let cross = nalgebra_glm::cross(&edge1, &edge2);
            let normal = if nalgebra_glm::length(&cross) > 1e-6 {
                nalgebra_glm::normalize(&cross)
            } else {
                Vec3::new(0.0, 1.0, 0.0)
            };

            let intensity = light.calculate_intensity(&normal);
            let shaded_color_u32 = fill_base_color.scale(intensity);

            rasterize_triangle(framebuffer, sv0, sv1, sv2, shaded_color_u32);
        }

        // Trazado de aristas en wireframe
        if has_wireframe {
            framebuffer.set_color(wireframe_color);
            framebuffer.draw_line(sv0.x as isize, sv0.y as isize, sv1.x as isize, sv1.y as isize);
            framebuffer.draw_line(sv1.x as isize, sv1.y as isize, sv2.x as isize, sv2.y as isize);
            framebuffer.draw_line(sv2.x as isize, sv2.y as isize, sv0.x as isize, sv0.y as isize);
        }
    }
}
