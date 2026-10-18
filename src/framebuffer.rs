// Representación de color ARGB compatible con minifb
#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    // Escala el color por un factor de intensidad de luz [0.0, 1.0]
    pub fn scale(self, intensity: f32) -> u32 {
        let factor = intensity.clamp(0.0, 1.0);
        let r = ((self.r as f32) * factor) as u32;
        let g = ((self.g as f32) * factor) as u32;
        let b = ((self.b as f32) * factor) as u32;
        (r << 16) | (g << 8) | b
    }
}

// Framebuffer para almacenar píxeles de color y mapa de profundidad (Z-Buffer) en CPU
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
    pub z_buffer: Vec<f32>,
    pub current_color: Color,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height],
            z_buffer: vec![f32::INFINITY; width * height],
            current_color: Color::new(255, 255, 255),
        }
    }

    // Limpia el buffer de color y reinicia el Z-Buffer a infinito
    pub fn clear(&mut self, color: Color) {
        self.buffer.fill(color.to_u32());
        self.z_buffer.fill(f32::INFINITY);
    }

    // Actualiza el color actual para trazar elementos
    pub fn set_color(&mut self, color: Color) {
        self.current_color = color;
    }

    // Dibuja un píxel individual validando límites de pantalla
    pub fn draw_pixel(&mut self, x: isize, y: isize) {
        if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
            self.buffer[y as usize * self.width + x as usize] = self.current_color.to_u32();
        }
    }

    // Escribe un píxel si pasa la prueba de profundidad del Z-Buffer
    #[inline(always)]
    pub fn draw_pixel_depth(&mut self, index: usize, z: f32, color_u32: u32) {
        if z < self.z_buffer[index] {
            self.z_buffer[index] = z;
            self.buffer[index] = color_u32;
        }
    }

    // Algoritmo de Bresenham optimizado para dibujar líneas entre dos puntos 2D
    pub fn draw_line(&mut self, x0: isize, y0: isize, x1: isize, y1: isize) {
        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.draw_pixel(x, y);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
}
