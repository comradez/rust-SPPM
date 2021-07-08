use adqselect::nth_element;
use crate::{
    hit::Hit,
    materials::Material,
    object3d::{Object3d, Triangle},
    ray::Ray,
    utils::{elementwise_division, get_max, get_min, prior_hit},
};
use json::JsonValue;
use lazy_static::lazy_static;
use std::{
    cmp::Ordering,
    sync::Arc,
    usize
};
use tobj::{self, LoadOptions};
use vecmat::vector::Vector3;
struct Node {
    min_pos: Vector3<f64>,
    max_pos: Vector3<f64>,
    lchild: Option<Box<Node>>,
    rchild: Option<Box<Node>>,
    triangle_index: usize,
}

impl Node {
    fn new(
        min_pos: Vector3<f64>,
        max_pos: Vector3<f64>,
        lchild: Option<Box<Node>>,
        rchild: Option<Box<Node>>,
        triangle_index: usize,
    ) -> Self {
        Self {
            min_pos,
            max_pos,
            lchild,
            rchild,
            triangle_index,
        }
    }
}

struct TriangleCompare {
    dimension: u8,
}

impl TriangleCompare {
    fn new(dimension: u8) -> Self {
        Self { dimension }
    }
    fn compare(&self, a: &TriangleIndex, b: &TriangleIndex) -> Ordering {
        match self.dimension {
            0 => a.min_pos.x().partial_cmp(&b.min_pos.x()).unwrap(),
            1 => a.min_pos.y().partial_cmp(&b.min_pos.y()).unwrap(),
            2 => a.min_pos.z().partial_cmp(&b.min_pos.z()).unwrap(),
            _ => panic!("NodeCompare dimension wrong!"),
        }
    }
}

#[derive(Clone, Copy)]
struct TriangleIndex {
    vertices: [usize; 3],
    min_pos: Vector3<f64>,
    max_pos: Vector3<f64>,
}

impl TriangleIndex {
    fn new(vertices: [usize; 3], points: &[Vector3<f64>]) -> Self {
        let min_pos = get_min(
            &get_min(&points[vertices[0]], &points[vertices[1]]),
            &points[vertices[2]],
        );
        let max_pos = get_max(
            &get_max(&points[vertices[0]], &points[vertices[1]]),
            &points[vertices[2]],
        );
        Self {
            vertices,
            min_pos,
            max_pos,
        }
    }
}

lazy_static! {
    static ref COMPS: [TriangleCompare; 3] = [
        TriangleCompare::new(0),
        TriangleCompare::new(1),
        TriangleCompare::new(2)
    ];
}

pub struct Mesh {
    root: Option<Box<Node>>,
    v: Vec<Vector3<f64>>,
    t: Vec<TriangleIndex>,
    vn: Option<Vec<Vector3<f64>>>,
    // mesh: objMesh,
    material: Arc<dyn Material + Send + Sync>,
    // n的计算*有点问题*，我在三角形的实现里是现场算的，这里先不放了
    // 因为我前面也没做好写Texture的准备，所以这里vt也摸了
}

impl Mesh {
    fn new(file_name: &str, material: Arc<dyn Material + Send + Sync>) -> Self {
        let (models, _) = tobj::load_obj(
            file_name,
            &LoadOptions {
                triangulate: true,
                ..Default::default()
            },
        )
        .expect("Problem loading model");
        let mesh = &models[0].mesh;
        let mut v: Vec<Vector3<f64>> = Vec::new();
        let mut t: Vec<TriangleIndex> = Vec::new();
        let vn: Option<Vec<Vector3<f64>>>;
        assert!(mesh.positions.len() % 3 == 0);
        for index in 0..mesh.positions.len() / 3 {
            v.push(Vector3::<f64>::from([
                mesh.positions[3 * index] as f64,
                mesh.positions[3 * index + 1] as f64,
                mesh.positions[3 * index + 2] as f64,
            ]));
        }
        assert!(mesh.indices.len() % 3 == 0);
        for index in 0..mesh.indices.len() / 3 {
            t.push(TriangleIndex::new(
                [
                    mesh.indices[3 * index] as usize,
                    mesh.indices[3 * index + 1] as usize,
                    mesh.indices[3 * index + 2] as usize,
                ],
                &v,
            ))
        }
        if !mesh.normals.is_empty() {
            let mut real_vn: Vec<Vector3<f64>> = Vec::new();
            assert!(mesh.normals.len() % 3 == 0);
            for index in 0..mesh.normals.len() / 3 {
                real_vn.push(Vector3::<f64>::from([
                    mesh.normals[3 * index] as f64,
                    mesh.normals[3 * index + 1] as f64,
                    mesh.normals[3 * index + 2] as f64,
                ]))
            }
            vn = Some(real_vn);
        } else {
            vn = None;
        }
        let mut root: Option<Box<Node>> = None;
        let len = t.len();
        Self::build(&mut root, &mut t, 0, len, 0);
        Self {
            root,
            v,
            t,
            vn,
            material,
        }
        // positions每三个代表一个点的位置，对应v
        // normals每三个代表一个点的法向（没有点法向就是空），对应vn
        // indices每三个点代表一个三角形的顶点index（因为triangulate是true所以一定是三个三个），对应t
    }
    fn build(
        root: &mut Option<Box<Node>>,
        t: &mut Vec<TriangleIndex>,
        left: usize,
        right: usize,
        dep: usize,
    ) {
        if root.is_none() {
            let mid = (left + right) / 2;
            let relative_mid = (right - left) / 2;
            nth_element(&mut t[left..right], relative_mid, &mut |x, y| {
                (*COMPS)[dep % 3].compare(x, y)
            });
            *root = Some(Box::new(Node::new(
                t[mid].min_pos,
                t[mid].max_pos,
                None,
                None,
                mid,
            )));
            let root = root.as_mut().unwrap();
            if left < mid {
                Self::build(&mut root.lchild, t, left, mid, dep + 1);
                let lchild = root.lchild.as_ref().unwrap();
                root.min_pos = get_min(&root.min_pos, &lchild.min_pos);
                root.max_pos = get_max(&root.max_pos, &lchild.max_pos);
            }
            if mid + 1 < right {
                Self::build(&mut root.rchild, t, mid + 1, right, dep + 1);
                let rchild = root.rchild.as_ref().unwrap();
                root.min_pos = get_min(&root.min_pos, &rchild.min_pos);
                root.max_pos = get_max(&root.max_pos, &rchild.max_pos);
            }
        }
    }
    fn query(&self, p: &Option<Box<Node>>, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        if let Some(p) = p {
            let d = ray.get_direction();
            let o = ray.get_origin();
            let mut t_min = tmin;
            let mut t_max = tmax;
            let min_pos_t = elementwise_division(&(p.min_pos - *o), d);
            let max_pos_t = elementwise_division(&(p.max_pos - *o), d);
            for i in 0..3_usize {
                if d[i] != 0. {
                    t_min = t_min.max(min_pos_t[i].min(max_pos_t[i]));
                    t_max = t_max.min(min_pos_t[i].max(max_pos_t[i]));
                }
            }
            if t_min > t_max {
                None
            } else {
                let ti = &self.t[p.triangle_index];
                let triangle = Triangle::new(
                    self.material.clone(),
                    [
                        self.v[ti.vertices[0]],
                        self.v[ti.vertices[1]],
                        self.v[ti.vertices[2]],
                    ],
                    self.vn
                        .as_ref()
                        .map(|vn| [vn[ti.vertices[0]], vn[ti.vertices[1]], vn[ti.vertices[2]]]),
                );
                let ret = None;
                let ret = prior_hit(ret, self.query(&p.lchild, ray, tmin, tmax));
                let ret = prior_hit(ret, self.query(&p.rchild, ray, tmin, tmax));
                prior_hit(ret, triangle.intersect(ray, tmin))
            }
        } else {
            None
        }
    }
}

impl Object3d for Mesh {
    fn intersect(&self, ray: &Ray, tmin: f64) -> Option<Hit> {
        self.query(&self.root, ray, tmin, 1e38_f64)
    }
}

pub fn build_mesh(
    mesh_attr: &JsonValue,
    materials: &[Arc<dyn Material + Send + Sync>],
) -> Arc<Mesh> {
    let material_index = mesh_attr["MaterialIndex"].as_usize().unwrap();
    let file_name = mesh_attr["File"].as_str().unwrap();
    Arc::new(Mesh::new(file_name, materials[material_index].clone()))
}
