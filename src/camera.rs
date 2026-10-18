use minifb::{Key, Window};
use nalgebra_glm::{Mat4, Vec3};

// Cámara orbital 3D con soporte para control manual y auto-rotación
pub struct Camera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub rotation_speed: f32,
    pub zoom_speed: f32,
    pub auto_rotate: bool,
}

impl Camera {
    pub fn new(target: Vec3, distance: f32) -> Self {
        Self {
            target,
            distance,
            yaw: 0.0,
            pitch: 0.3,
            rotation_speed: 1.2,
            zoom_speed: 4.0,
            auto_rotate: true,
        }
    }

    // Procesa entradas de teclado (WASD, flechas, Q/E) y actualiza posición
    pub fn process_input(&mut self, window: &Window, dt: f32) {
        let mut manual_rotation = false;

        // Rotación horizontal (Yaw): A / D o Flechas Izquierda / Derecha
        if window.is_key_down(Key::A) || window.is_key_down(Key::Left) {
            self.yaw -= self.rotation_speed * dt;
            manual_rotation = true;
        }
        if window.is_key_down(Key::D) || window.is_key_down(Key::Right) {
            self.yaw += self.rotation_speed * dt;
            manual_rotation = true;
        }

        // Rotación vertical (Pitch): W / S o Flechas Arriba / Abajo
        if window.is_key_down(Key::W) || window.is_key_down(Key::Up) {
            self.pitch += self.rotation_speed * dt;
            manual_rotation = true;
        }
        if window.is_key_down(Key::S) || window.is_key_down(Key::Down) {
            self.pitch -= self.rotation_speed * dt;
            manual_rotation = true;
        }

        // Limita el pitch para evitar invertir la cámara
        self.pitch = self.pitch.clamp(-1.5, 1.5);

        // Zoom: E para acercar (zoom in), Q para alejar (zoom out)
        if window.is_key_down(Key::E) {
            self.distance = (self.distance - self.zoom_speed * dt).max(1.0);
        }
        if window.is_key_down(Key::Q) {
            self.distance = (self.distance + self.zoom_speed * dt).min(50.0);
        }

        // Auto-rotación suave cuando no se está rotando manualmente
        if self.auto_rotate && !manual_rotation {
            self.yaw += 0.5 * dt;
        }
    }

    // Calcula la posición en el espacio 3D a partir de coordenadas esféricas
    pub fn get_eye_position(&self) -> Vec3 {
        let x = self.target.x + self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.target.y + self.distance * self.pitch.sin();
        let z = self.target.z + self.distance * self.pitch.cos() * self.yaw.cos();
        Vec3::new(x, y, z)
    }

    // Matriz de vista (View Matrix) mirando hacia el objetivo
    pub fn get_view_matrix(&self) -> Mat4 {
        nalgebra_glm::look_at(
            &self.get_eye_position(),
            &self.target,
            &Vec3::new(0.0, 1.0, 0.0),
        )
    }

    // Matriz de proyección en perspectiva
    pub fn get_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        nalgebra_glm::perspective(aspect_ratio, 45.0_f32.to_radians(), 0.1, 100.0)
    }
}
