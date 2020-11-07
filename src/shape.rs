// TODO: Constructive solid geometry
use color_eyre::eyre::{anyhow, bail};
use std::fs::File;
use std::io::{prelude::*, BufReader, SeekFrom};
use std::sync::Arc;

pub use cube::Cube;
pub use mesh::{Mesh, Triangle};
pub use plane::Plane;
pub use sphere::Sphere;

mod cube;
mod mesh;
mod plane;
mod sphere;

/// Represents a physical shape, which can be hit by a ray to find intersections
pub trait Shape: Send + Sync {
    /// Intersect the shape with a ray, for `t >= t_min`, returning true and mutating
    /// `h` if an intersection was found before the current closest one
    fn intersect(&self, ray: &Ray, t_min: f32, record: &mut HitRecord) -> bool;
}

/// An infinite ray in one direction
#[derive(Copy, Clone)]
pub struct Ray {
    /// The origin of the ray
    pub origin: glm::Vec3,

    /// The unit direction of the ray
    pub dir: glm::Vec3,
}

impl Ray {
    /// Evaluates the ray at a given value of the parameter
    pub fn at(&self, time: f32) -> glm::Vec3 {
        return self.origin + time * self.dir;
    }

    /// Apply a homogeneous transformation to the ray (not normalizing direction)
    pub fn apply_transform(&self, transform: &glm::Mat4) -> Self {
        let ref_pt = self.at(1.0);
        let origin = transform * (self.origin.to_homogeneous() + glm::vec4(0.0, 0.0, 0.0, 1.0));
        let origin = glm::vec4_to_vec3(&(origin / origin.w));
        let ref_pt = transform * (ref_pt.to_homogeneous() + glm::vec4(0.0, 0.0, 0.0, 1.0));
        let ref_pt = glm::vec4_to_vec3(&(ref_pt / ref_pt.w));
        Self {
            origin,
            dir: ref_pt - origin,
        }
    }
}

/// Record of when a hit occurs, and the corresponding normal
///
/// TODO: Look into adding more information, such as (u, v) texels
pub struct HitRecord {
    /// The time at which the hit occurs (see `Ray`)
    pub time: f32,

    /// The normal of the hit in some coordinate system
    pub normal: glm::Vec3,
}

impl Default for HitRecord {
    fn default() -> Self {
        Self {
            time: f32::INFINITY,
            normal: glm::vec3(0.0, 0.0, 0.0),
        }
    }
}

impl HitRecord {
    /// Construct a new `HitRecord` at infinity
    pub fn new() -> Self {
        Default::default()
    }
}

/// Helper function to construct an `Arc` for a sphere
pub fn sphere() -> Arc<Sphere> {
    Arc::new(Sphere)
}

/// Helper function to construct an `Arc` for a plane
pub fn plane(normal: glm::Vec3, value: f32) -> Arc<Plane> {
    Arc::new(Plane { normal, value })
}

/// Helper function to construct an `Arc` for a cube
pub fn cube() -> Arc<Cube> {
    Arc::new(Cube)
}

fn parse_index(value: &str) -> Option<usize> {
    value.parse::<i32>().ok().and_then(|index| {
        if index > 0 {
            Some((index - 1) as usize)
        } else {
            None
        }
    })
}

/// Helper function to load a mesh from a Wavefront .OBJ file
///
/// See https://www.cs.cmu.edu/~mbz/personal/graphics/obj.html for details.
pub fn load_obj(path: &str) -> color_eyre::Result<Arc<Mesh>> {
    // TODO: no texture or material support yet
    let mut vertices: Vec<glm::Vec3> = Vec::new();
    let mut normals: Vec<glm::Vec3> = Vec::new();
    let mut triangles = Vec::new();

    let reader = BufReader::new(File::open(path)?);
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with("#") || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();
        match tokens[0] {
            "v" => {
                // vertex
                let v = glm::vec3::<f32>(
                    tokens[1].parse().expect("Failed to parse vertex in .OBJ"),
                    tokens[2].parse().expect("Failed to parse vertex in .OBJ"),
                    tokens[3].parse().expect("Failed to parse vertex in .OBJ"),
                );
                vertices.push(v);
            }
            "vt" => {
                // vertex texture
                eprintln!("Warning: Found 'vt' in .OBJ file, unimplemented, skipping...");
            }
            "vn" => {
                // vertex normal
                let vn = glm::vec3::<f32>(
                    tokens[1].parse().expect("Failed to parse vertex in .OBJ"),
                    tokens[2].parse().expect("Failed to parse vertex in .OBJ"),
                    tokens[3].parse().expect("Failed to parse vertex in .OBJ"),
                );
                normals.push(vn);
            }
            "f" => {
                // face
                let (vi, vni): (Vec<_>, Vec<_>) = tokens[1..]
                    .iter()
                    .map(|&vertex| {
                        let args: Vec<_> = vertex
                            .split("/")
                            .chain(std::iter::repeat(""))
                            .take(3)
                            .collect();
                        (parse_index(args[0]), parse_index(args[2]))
                    })
                    .unzip();
                for i in 1..(vi.len() - 1) {
                    let a = 0;
                    let b = i;
                    let c = i + 1;
                    let v1 = vertices[vi[a].ok_or(anyhow!("Invalid vertex index"))?];
                    let v2 = vertices[vi[b].ok_or(anyhow!("Invalid vertex index"))?];
                    let v3 = vertices[vi[c].ok_or(anyhow!("Invalid vertex index"))?];
                    if vni[a].is_none() || vni[b].is_none() || vni[c].is_none() {
                        triangles.push(Triangle::from_vertices(v1, v2, v3));
                    } else {
                        triangles.push(Triangle {
                            v1,
                            v2,
                            v3,
                            n1: normals[vni[a].unwrap()],
                            n2: normals[vni[b].unwrap()],
                            n3: normals[vni[c].unwrap()],
                        });
                    }
                }
            }
            "mtllib" => {
                // material library
                eprintln!("Warning: Found 'mtllib' in .OBJ file, unimplemented, skipping...");
            }
            "usemtl" => {
                // material
                eprintln!("Warning: Found 'usemtl' in .OBJ file, unimplemented, skipping...");
            }
            // Ignore other unrecognized or non-standard commands
            _ => (),
        }
    }

    Ok(Arc::new(Mesh::new(triangles)))
}

/// Helper function to load a mesh from a .STL file
///
/// See https://en.wikipedia.org/wiki/STL_%28file_format%29 and
/// https://stackoverflow.com/a/26171886 for details.
pub fn load_stl(path: &str) -> color_eyre::Result<Arc<Mesh>> {
    let size = std::fs::metadata(path)?.len();
    if size < 15 {
        bail!("Opened .STL file {} is too short", path);
    }
    let mut file = File::open(path)?;
    if size >= 84 {
        file.seek(SeekFrom::Start(80))?;
        let mut buf: [u8; 4] = Default::default();
        file.read_exact(&mut buf)?;
        let num_triangles = u32::from_le_bytes(buf) as u64;
        if size == 84 + num_triangles * 50 {
            // Very likely binary STL format
            return load_stl_binary(file, num_triangles);
        }
    }

    file.seek(SeekFrom::Start(0))?;
    let mut buf: [u8; 6] = Default::default();
    file.read_exact(&mut buf)?;
    if std::str::from_utf8(&buf) == Ok("solid ") {
        // ASCII STL format
        load_stl_ascii(file)
    } else {
        bail!("Opened .STL file {}, but could not determine format", path);
    }
}

fn load_stl_ascii(file: File) -> color_eyre::Result<Arc<Mesh>> {
    let reader = BufReader::new(file);
    let mut lines = reader.lines().skip(1);
    let mut triangles = Vec::new();
    while let Some(line) = lines.next() {
        let vn: Vec<_> = line?
            .trim()
            .strip_prefix("facet normal ")
            .ok_or(anyhow!("Malformed STL file: expected `facet normal`"))?
            .split_ascii_whitespace()
            .map(|token| token.parse::<f32>().expect("Invalid facet normal"))
            .collect();
        let vn = glm::vec3(vn[0], vn[1], vn[2]);
        lines.next().unwrap()?; // "outer loop"
        let mut vs: [glm::Vec3; 3] = Default::default();
        for i in 0..3 {
            let v: Vec<_> = lines
                .next()
                .unwrap()?
                .trim()
                .strip_prefix("vertex ")
                .ok_or(anyhow!("Malformed STL file: expected `vertex`"))?
                .split_ascii_whitespace()
                .map(|token| token.parse::<f32>().expect("Invalid vertex"))
                .collect();
            vs[i] = glm::vec3(v[0], v[1], v[2]);
        }
        lines.next().unwrap()?; // "endloop"
        lines.next().unwrap()?; // "endfacet"

        triangles.push(Triangle {
            v1: vs[0],
            v2: vs[1],
            v3: vs[2],
            n1: vn,
            n2: vn,
            n3: vn,
        });
    }
    Ok(Arc::new(Mesh::new(triangles)))
}

fn load_stl_binary(file: File, num_triangles: u64) -> color_eyre::Result<Arc<Mesh>> {
    let mut reader = BufReader::new(file);
    let mut triangles = Vec::new();
    let read_vec3 = |reader: &mut BufReader<File>| -> color_eyre::Result<glm::Vec3> {
        let mut buf: [u8; 4] = Default::default();
        reader.read_exact(&mut buf)?;
        let v1 = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let v2 = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let v3 = f32::from_le_bytes(buf);
        Ok(glm::vec3(v1, v2, v3))
    };
    for _ in 0..num_triangles {
        let vn = read_vec3(&mut reader)?;
        let v1 = read_vec3(&mut reader)?;
        let v2 = read_vec3(&mut reader)?;
        let v3 = read_vec3(&mut reader)?;
        reader.seek(SeekFrom::Current(2))?;
        triangles.push(Triangle {
            v1,
            v2,
            v3,
            n1: vn,
            n2: vn,
            n3: vn,
        });
    }
    Ok(Arc::new(Mesh::new(triangles)))
}
