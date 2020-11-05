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

impl Triangle {
    /// Construct a triangle from three vertices, inferring the normals
    pub fn from_vertices(v1: glm::Vec3, v2: glm::Vec3, v3: glm::Vec3) -> Self {
        let n = (v2 - v1).cross(&(v3 - v1)).normalize();
        Self {
            v1,
            v2,
            v3,
            n1: n,
            n2: n,
            n3: n,
        }
    }
}

impl Shape for Triangle {
    fn intersect(&self, ray: &Ray, t_min: f32, record: &mut HitRecord) -> bool {
        let plane_normal = (self.v2 - self.v1).cross(&(self.v3 - self.v1)).normalize();
        let cosine = plane_normal.dot(&ray.dir);
        if cosine.abs() < 1e-8 {
            // Parallel ray and plane of triangle
            return false;
        }
        let time = plane_normal.dot(&(self.v1 - ray.origin)) / cosine;
        if time < t_min || time >= record.time {
            return false;
        }

        // Okay, so let's compute barycentric coordinates now, fast
        // https://gamedev.stackexchange.com/a/23745
        let p = ray.at(time);
        let (d0, d1, d2) = (self.v2 - self.v1, self.v3 - self.v1, p - self.v1);
        let d00 = d0.dot(&d0);
        let d01 = d0.dot(&d1);
        let d11 = d1.dot(&d1);
        let d20 = d2.dot(&d0);
        let d21 = d2.dot(&d1);
        let denom = d00 * d11 - d01 * d01;
        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;
        let u = 1.0 - v - w;

        if u >= 0.0 && v >= 0.0 && w >= 0.0 {
            record.time = time;
            record.normal = (u * self.n1 + v * self.n2 + w * self.n3).normalize();
            true
        } else {
            false
        }
    }
}

/// A triangle mesh
pub struct Mesh {
    /// An array containing triangles, which constitute the mesh
    ///
    /// Unlike GLOO, we don't need to care about OpenGL, so storing separate buffers
    /// for positions, vertex normals, and indices doesn't really make sense.
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    /// Construct a new mesh from a collection of triangles
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Self { triangles }
    }
}

impl Shape for Mesh {
    fn intersect(&self, ray: &Ray, t_min: f32, record: &mut HitRecord) -> bool {
        // Currently this implementation is brute force
        // TODO: Optimize this to use BVH or kd-trees
        self.triangles
            .iter()
            .map(|triangle| triangle.intersect(ray, t_min, record))
            .any(|x| x)
    }
}
