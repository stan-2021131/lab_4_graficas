use nalgebra_glm::Vec3;
use std::path::Path;

// Estructura que almacena los vértices y la topología de la malla
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl Mesh {
    // Carga vértices e índices triangulados desde un archivo .obj usando tobj
    pub fn load_from_obj(path: &Path) -> Result<Self, String> {
        let load_options = tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        };

        let (models, _) = tobj::load_obj(path, &load_options)
            .map_err(|e| format!("Error al cargar el archivo OBJ: {e}"))?;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for model in models {
            let mesh = &model.mesh;
            let base_index = vertices.len() as u32;

            for chunk in mesh.positions.chunks_exact(3) {
                vertices.push(Vec3::new(chunk[0], chunk[1], chunk[2]));
            }

            for &index in &mesh.indices {
                indices.push(base_index + index);
            }
        }

        Ok(Self { vertices, indices })
    }
}
