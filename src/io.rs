use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, SeekFrom};

use crate::material::Material;
use crate::object::Object;
use crate::shape::{Mesh, Triangle};

fn parse_index(value: &str, len: usize) -> Option<usize> {
    value.parse::<i32>().ok().and_then(|index| {
        if index > 0 {
            Some((index - 1) as usize)
        } else {
            Some((len as i32 + index) as usize)
        }
    })
}

fn invalid_data(message: impl Into<Box<dyn Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// Helper function to load a mesh geometry from a Wavefront .OBJ file
///
/// See [here](https://www.cs.cmu.edu/~mbz/personal/graphics/obj.html) for details.
pub fn load_obj(file: File) -> io::Result<Mesh> {
    let mut vertices: Vec<glm::DVec3> = Vec::new();
    let mut normals: Vec<glm::DVec3> = Vec::new();
    let mut triangles = Vec::new();

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with("#") || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();
        match tokens[0] {
            "v" => {
                // vertex
                let v = parse_obj_point(&tokens)?;
                vertices.push(v);
            }
            "vt" => {
                // vertex texture
                eprintln!("Warning: Found 'vt' in .OBJ file, unimplemented, skipping...");
            }
            "vn" => {
                // vertex normal
                let vn = parse_obj_point(&tokens)?;
                normals.push(vn);
            }
            "f" => {
                // face
                let face = parse_obj_face(&tokens, &vertices, &normals)?;
                triangles.extend(face);
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

    Ok(Mesh::new(triangles))
}

/// Helper function to load an object, with materials, from a Wavefront .OBJ file
///
/// This function ignores the `mtllib` commands that look for files in the same directory,
/// instead choosing a more explicit approach where you pass in the `.mtl` file directly
/// as the second argument.
///
/// See [here](https://www.cs.cmu.edu/~mbz/personal/graphics/obj.html) and
/// [here](http://paulbourke.net/dataformats/mtl/) for details.
pub fn load_obj_with_mtl(obj_file: File, mtl_file: File) -> io::Result<Vec<Object>> {
    let materials = load_mtl(mtl_file)?;

    let mut vertices: Vec<glm::DVec3> = Vec::new();
    let mut normals: Vec<glm::DVec3> = Vec::new();
    let mut objects = Vec::new();

    let mut current_triangles = Vec::new();
    let mut current_material = Material::default();
    let mut last_usemtl = None;

    let reader = BufReader::new(obj_file);
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with("#") || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();
        match tokens[0] {
            "v" => {
                // vertex
                let v = parse_obj_point(&tokens)?;
                vertices.push(v);
            }
            "vt" => {
                // vertex texture
                eprintln!("Warning: Found 'vt' in .OBJ file, unimplemented, skipping...");
            }
            "vn" => {
                // vertex normal
                let vn = parse_obj_point(&tokens)?;
                normals.push(vn);
            }
            "f" => {
                // face
                let face = parse_obj_face(&tokens, &vertices, &normals)?;
                current_triangles.extend(face);
            }
            "usemtl" => {
                // material
                if last_usemtl.is_none() || last_usemtl.as_ref().unwrap() != tokens[1] {
                    if !current_triangles.is_empty() {
                        objects.push(
                            Object::new(Mesh::new(current_triangles.drain(..).collect()))
                                .material(current_material),
                        );
                    }
                    current_material = *materials.get(tokens[1]).ok_or(invalid_data(format!(
                        "Could not found `usemtl {}` in library",
                        tokens[1]
                    )))?;
                    last_usemtl = Some(tokens[1].to_owned());
                }
            }
            // Ignore other unrecognized or non-standard commands
            _ => (),
        }
    }

    if !current_triangles.is_empty() {
        objects.push(
            Object::new(Mesh::new(current_triangles.drain(..).collect()))
                .material(current_material),
        );
    }

    Ok(objects)
}

fn parse_obj_point(line: &[&str]) -> io::Result<glm::DVec3> {
    let parse_vertex = |s: &str| {
        s.parse()
            .map_err(|_| invalid_data("Failed to parse vertex in .OBJ"))
    };
    Ok(glm::vec3::<f64>(
        parse_vertex(line[1])?,
        parse_vertex(line[2])?,
        parse_vertex(line[3])?,
    ))
}

fn parse_obj_face(
    line: &[&str],
    vertices: &[glm::DVec3],
    normals: &[glm::DVec3],
) -> io::Result<Vec<Triangle>> {
    let mut vi = Vec::new();
    let mut vni = Vec::new();
    for vertex in &line[1..] {
        let args: Vec<_> = vertex
            .split("/")
            .chain(std::iter::repeat(""))
            .take(3)
            .collect();
        let vert_index = parse_index(args[0], vertices.len());
        vi.push(vert_index.ok_or(invalid_data("Invalid vertex index"))?);
        vni.push(parse_index(args[2], normals.len()));
    }
    let mut triangles = Vec::new();
    for i in 1..(vi.len() - 1) {
        let (a, b, c) = (0, i, i + 1);
        let v1 = vertices[vi[a]];
        let v2 = vertices[vi[b]];
        let v3 = vertices[vi[c]];
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
    Ok(triangles)
}

fn load_mtl(file: File) -> io::Result<HashMap<String, Material>> {
    let mut materials: HashMap<String, Material> = HashMap::new();
    let mut current = None;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?.trim().to_string();
        if line.starts_with("#") || line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();
        if tokens[0] == "newmtl" {
            let name = String::from(tokens[1]);
            current = Some(name.clone());
            materials.entry(name).or_default();
        } else {
            let current = current.as_ref().ok_or(invalid_data(
                "Material was not specified with `newmtl` before properties were added",
            ))?;
            let mat = materials.get_mut(current).unwrap();
            // Best-effort conversion from Ka/Kd/Ks material to physically-based material
            match tokens[0] {
                "Kd" => mat.color = parse_obj_point(&tokens)?,
                "Ns" => {
                    // Specular power to roughness, see https://computergraphics.stackexchange.com/a/1517
                    let ns: f64 = tokens[1]
                        .parse()
                        .map_err(|_| invalid_data("Could not parse Ks value"))?;
                    mat.roughness = (2.0 / (ns + 2.0)).sqrt().sqrt();
                }
                "Ni" => {
                    let ns: f64 = tokens[1]
                        .parse()
                        .map_err(|_| invalid_data("Could not parse Ns value"))?;
                    // Our materials can't correctly handle IOR of exactly 1.0
                    mat.index = ns.max(1.0 + 1e-4);
                }
                "d" => {
                    let dissolve: f64 = tokens[1]
                        .parse()
                        .map_err(|_| invalid_data("Could not parse d value"))?;
                    if dissolve < 0.8 {
                        mat.transparent = true;
                    }
                }
                // Ignore all other mtllib commands
                _ => (),
            };
        }
    }
    Ok(materials)
}

/// Helper function to load a mesh from a .STL file
///
/// See https://en.wikipedia.org/wiki/STL_%28file_format%29 and
/// https://stackoverflow.com/a/26171886 for details.
pub fn load_stl(mut file: File) -> io::Result<Mesh> {
    let size = file.metadata()?.len();
    if size < 15 {
        return Err(invalid_data("Loaded .STL file is too short"));
    }
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
        Err(invalid_data(
            "Loaded .STL file, but could not determine format",
        ))
    }
}

fn load_stl_ascii(file: File) -> io::Result<Mesh> {
    let reader = BufReader::new(file);
    let mut lines = reader.lines().skip(1);
    let mut triangles = Vec::new();
    while let Some(line) = lines.next() {
        let vn: Vec<_> = line?
            .trim()
            .strip_prefix("facet normal ")
            .ok_or(invalid_data("Malformed STL file: expected `facet normal`"))?
            .split_ascii_whitespace()
            .map(|token| token.parse::<f64>().expect("Invalid facet normal"))
            .collect();
        let vn = glm::vec3(vn[0], vn[1], vn[2]);
        lines.next().unwrap()?; // "outer loop"
        let mut vs: [glm::DVec3; 3] = Default::default();
        for i in 0..3 {
            let v: Vec<_> = lines
                .next()
                .unwrap()?
                .trim()
                .strip_prefix("vertex ")
                .ok_or(invalid_data("Malformed STL file: expected `vertex`"))?
                .split_ascii_whitespace()
                .map(|token| token.parse::<f64>().expect("Invalid vertex"))
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
    Ok(Mesh::new(triangles))
}

fn load_stl_binary(file: File, num_triangles: u64) -> io::Result<Mesh> {
    let mut reader = BufReader::new(file);
    let mut triangles = Vec::new();
    let read_vec3 = |reader: &mut BufReader<File>| -> io::Result<glm::DVec3> {
        let mut buf: [u8; 4] = Default::default();
        reader.read_exact(&mut buf)?;
        let v1 = f32::from_le_bytes(buf) as f64;
        reader.read_exact(&mut buf)?;
        let v2 = f32::from_le_bytes(buf) as f64;
        reader.read_exact(&mut buf)?;
        let v3 = f32::from_le_bytes(buf) as f64;
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
    Ok(Mesh::new(triangles))
}
