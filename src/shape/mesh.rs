use super::{HitRecord, Ray, Shape};

/// A triangle with three vertices and three normals
pub struct Triangle {
    /// The first vertex
    pub v1: glm::Vec3,
    /// The second vertex
    pub v2: glm::Vec3,
    /// The third vertex
    pub v3: glm::Vec3,

    /// The first normal vector
    pub n1: glm::Vec3,
    /// The second normal vector
    pub n2: glm::Vec3,
    /// The third normal vector
    pub n3: glm::Vec3,
}

impl Shape for Triangle {
    fn intersect(&self, ray: &Ray, t_min: f32, record: &mut HitRecord) -> bool {
        todo!();
    }
}

/// A triangle mesh
pub struct Mesh {
    /// The vertex buffer
    pub vertices: Vec<glm::Vec3>,

    /// The unit normal vectors corresponding to the vertex buffer
    pub normals: Vec<glm::Vec3>,

    /// Triangles, given as positively-oriented triplets of indices
    pub triangles: Vec<[usize; 3]>,
}

impl Shape for Mesh {
    fn intersect(&self, ray: &Ray, t_min: f32, record: &mut HitRecord) -> bool {
        todo!();
    }
}
