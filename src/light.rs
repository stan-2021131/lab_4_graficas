use nalgebra_glm::Vec3;

// Estructura que define una fuente de luz direccional en el espacio 3D
pub struct Light {
    pub direction: Vec3,
    pub ambient: f32,
    pub diffuse: f32,
}

impl Light {
    pub fn new(direction: Vec3, ambient: f32, diffuse: f32) -> Self {
        Self {
            direction: nalgebra_glm::normalize(&direction),
            ambient,
            diffuse,
        }
    }

    // Calcula la intensidad lumínica para una cara plana usando Lambertian Shading
    #[inline(always)]
    pub fn calculate_intensity(&self, normal: &Vec3) -> f32 {
        let n_dot_l = nalgebra_glm::dot(normal, &-self.direction).max(0.0);
        (self.ambient + n_dot_l * self.diffuse).clamp(0.0, 1.0)
    }
}
