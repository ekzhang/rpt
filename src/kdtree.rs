use crate::shape::{HitRecord, Ray, Shape};

const SCORE_THRESHOLD: f64 = 0.75;

/// A geometric object with a bounding box (needed for kd-tree intersections)
pub trait Bounded {
    /// Returns the object's bounding box
    fn bounding_box(&self) -> BoundingBox;
}

/// A 3D axis-aligned bounding box
#[derive(Copy, Clone, Debug)]
pub struct BoundingBox {
    /// The coordinates (x_min, y_min, z_min)
    pub p_min: glm::Vec3,
    /// The coordinates (x_max, y_max, z_max)
    pub p_max: glm::Vec3,
}

impl Default for BoundingBox {
    fn default() -> Self {
        BoundingBox {
            p_min: glm::vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            p_max: glm::vec3(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY),
        }
    }
}

impl BoundingBox {
    /// Combine two bounding boxes together, to form their union
    pub fn merge(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            p_min: glm::min2(&self.p_min, &other.p_min),
            p_max: glm::max2(&self.p_max, &other.p_max),
        }
    }

    /// Returns the minimum and maximum times of intersection with a ray
    pub fn intersect(&self, ray: &Ray) -> (f32, f32) {
        let x1 = (self.p_min.x - ray.origin.x) / ray.dir.x;
        let x2 = (self.p_max.x - ray.origin.x) / ray.dir.x;
        let (x1, x2) = (f32::min(x1, x2), f32::max(x1, x2));
        let y1 = (self.p_min.y - ray.origin.y) / ray.dir.y;
        let y2 = (self.p_max.y - ray.origin.y) / ray.dir.y;
        let (y1, y2) = (f32::min(y1, y2), f32::max(y1, y2));
        let z1 = (self.p_min.z - ray.origin.z) / ray.dir.z;
        let z2 = (self.p_max.z - ray.origin.z) / ray.dir.z;
        let (z1, z2) = (f32::min(z1, z2), f32::max(z1, z2));
        (
            f32::max(f32::max(x1, y1), z1),
            f32::min(f32::min(x2, y2), z2),
        )
    }
}

/// A kd-tree based on bounding boxes, to accelerate ray intersections
///
/// This is a simple implementation; we don't care about slight performance
/// optimizations from things like cache locality and packing structs into 8 bytes
/// (such as what's given in the PBRT book).
///
/// The tree construction & ray intersection code was largely adapted from
/// [https://github.com/fogleman/pt/blob/master/pt/tree.go]. Parts of the
/// construction algorithm were also taken from PBRT.
pub struct KdTree<T: Bounded> {
    root: Box<KdNode>,
    objects: Vec<T>,
    bounds: BoundingBox,
}

impl<T: Bounded> KdTree<T> {
    /// Construct a new kd-tree from a collection of objects
    pub fn new(objects: Vec<T>) -> Self {
        let indices = (0..objects.len()).collect();
        let bounds = objects
            .iter()
            .map(T::bounding_box)
            .fold(BoundingBox::default(), |b1, b2| b1.merge(&b2));
        Self {
            root: construct(&objects, indices),
            objects,
            bounds,
        }
    }
}

impl<T: Bounded> Bounded for KdTree<T> {
    fn bounding_box(&self) -> BoundingBox {
        self.bounds
    }
}

impl<T: Bounded + Shape> Shape for KdTree<T> {
    fn intersect(&self, ray: &Ray, t_min: f32, record: &mut HitRecord) -> bool {
        let (b_min, b_max) = self.bounds.intersect(ray);
        if f32::max(b_min, t_min) > f32::min(b_max, record.time) {
            // No potential for intersecting, even the broader bounding box
            return false;
        }
        self.intersect_subtree(&self.root, ray, t_min, record)
    }
}

impl<T: Bounded + Shape> KdTree<T> {
    fn intersect_subtree(
        &self,
        node: &KdNode,
        ray: &Ray,
        t_min: f32,
        record: &mut HitRecord,
    ) -> bool {
        // Guarantee: we always find the closest intersection in the current kd-cell, if any
        let (t_split, first, second) = match node {
            KdNode::Leaf(indices) => {
                // Try to intersect the ray with all objects in the node
                let mut result = false;
                for &index in indices {
                    if self.objects[index].intersect(ray, t_min, record) {
                        result = true;
                    }
                }
                return result;
            }
            KdNode::SplitX(value, left, right) => {
                let t_split = (value - ray.origin.x) / ray.dir.x;
                let left_first =
                    (ray.origin.x < *value) || (ray.origin.x == *value && ray.dir.x <= 0.0);
                if left_first {
                    (t_split, left, right)
                } else {
                    (t_split, right, left)
                }
            }
            KdNode::SplitY(value, left, right) => {
                let t_split = (value - ray.origin.y) / ray.dir.y;
                let left_first =
                    (ray.origin.y < *value) || (ray.origin.y == *value && ray.dir.y <= 0.0);
                if left_first {
                    (t_split, left, right)
                } else {
                    (t_split, right, left)
                }
            }
            KdNode::SplitZ(value, left, right) => {
                let t_split = (value - ray.origin.z) / ray.dir.z;
                let left_first =
                    (ray.origin.z < *value) || (ray.origin.z == *value && ray.dir.z <= 0.0);
                if left_first {
                    (t_split, left, right)
                } else {
                    (t_split, right, left)
                }
            }
        };
        if record.time < t_split || t_split <= 0.0 {
            self.intersect_subtree(first, ray, t_min, record)
        } else if t_split < t_min {
            self.intersect_subtree(second, ray, t_min, record)
        } else {
            let h1 = self.intersect_subtree(first, ray, t_min, record);
            if h1 && record.time < t_split {
                true
            } else {
                // We still might need to visit the second subtree, since the first
                // subtree might have discovered an intersection that lies outside of the
                // actual subtree bounding box itself, but is suboptimal.
                let h2 = self.intersect_subtree(second, ray, t_split, record);
                h1 || h2
            }
        }
    }
}

enum KdNode {
    SplitX(f32, Box<KdNode>, Box<KdNode>),
    SplitY(f32, Box<KdNode>, Box<KdNode>),
    SplitZ(f32, Box<KdNode>, Box<KdNode>),
    /// Stores a vector of _indices_ into the objects buffer
    Leaf(Vec<usize>),
}

fn construct<T: Bounded>(objects: &[T], indices: Vec<usize>) -> Box<KdNode> {
    if indices.len() < 16 {
        return Box::new(KdNode::Leaf(indices));
    }
    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    let mut bboxs = Vec::new();
    for &index in indices.iter() {
        let bbox = objects[index].bounding_box();
        xs.push(bbox.p_min.x);
        xs.push(bbox.p_max.x);
        ys.push(bbox.p_min.y);
        ys.push(bbox.p_max.y);
        zs.push(bbox.p_min.z);
        zs.push(bbox.p_max.z);
        bboxs.push(bbox);
    }
    let float_cmp = |a: &f32, b: &f32| a.partial_cmp(b).unwrap();
    xs.sort_by(float_cmp);
    ys.sort_by(float_cmp);
    zs.sort_by(float_cmp);
    let (mx, my, mz) = (median(&xs), median(&ys), median(&zs));

    let partition_score = |dim: usize, value: f32| {
        let (mut left, mut right) = (0usize, 0usize);
        for i in 0..indices.len() {
            if bboxs[i].p_min[dim] <= value {
                left += 1;
            }
            if bboxs[i].p_max[dim] >= value {
                right += 1;
            }
        }
        left.max(right)
    };

    let partition = |dim: usize, value: f32| {
        let (mut left, mut right) = (Vec::new(), Vec::new());
        for (i, &index) in indices.iter().enumerate() {
            if bboxs[i].p_min[dim] <= value {
                left.push(index);
            }
            if bboxs[i].p_max[dim] >= value {
                right.push(index);
            }
        }
        (left, right)
    };

    let sx = partition_score(0, mx);
    let sy = partition_score(1, my);
    let sz = partition_score(2, mz);
    let threshold = (indices.len() as f64 * SCORE_THRESHOLD) as usize;
    if sx.min(sy).min(sz) >= threshold {
        // The split isn't worth it, so we just make this a leaf node
        Box::new(KdNode::Leaf(indices))
    } else {
        let mut split_dir = -1;

        // First try the direction with maximum extent
        let bounds = bboxs
            .iter()
            .fold(BoundingBox::default(), |b1, b2| b1.merge(&b2));
        let extent = bounds.p_max - bounds.p_min;
        if extent.x > extent.y && extent.x > extent.z {
            if sx < threshold {
                split_dir = 0;
            }
        } else if extent.y > extent.z {
            if sy < threshold {
                split_dir = 1;
            }
        } else {
            if sz < threshold {
                split_dir = 2;
            }
        }

        // Then, try any split direction, with best possible score
        if split_dir == -1 {
            if sx < sy && sx < sz {
                split_dir = 0;
            } else if sy < sz {
                split_dir = 1;
            } else {
                split_dir = 2;
            }
        }

        if split_dir == 0 {
            let (left, right) = partition(0, mx);
            Box::new(KdNode::SplitX(
                mx,
                construct(objects, left),
                construct(objects, right),
            ))
        } else if split_dir == 1 {
            let (left, right) = partition(1, my);
            Box::new(KdNode::SplitY(
                my,
                construct(objects, left),
                construct(objects, right),
            ))
        } else {
            assert!(split_dir == 2);
            let (left, right) = partition(2, mz);
            Box::new(KdNode::SplitZ(
                mz,
                construct(objects, left),
                construct(objects, right),
            ))
        }
    }
}

fn median(sorted_array: &[f32]) -> f32 {
    assert!(!sorted_array.is_empty());
    if sorted_array.len() % 2 == 1 {
        sorted_array[sorted_array.len() / 2]
    } else {
        let mid = sorted_array.len() / 2;
        (sorted_array[mid] + sorted_array[mid - 1]) / 2.0
    }
}
