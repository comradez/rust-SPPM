use json::JsonValue;
use vecmat::vector::Vector3;
use tobj::{self, LoadOptions};
use std::cmp::Ordering;
use std::usize;
use lazy_static::lazy_static;

use crate::hit::Hit;
use crate::materials::Material;
use crate::matrix::elementwise_division;
use crate::matrix::get_max;
use crate::matrix::get_min;
use crate::floydrivest::floydrivest;
use crate::object3d::Object3d;
use crate::ray::Ray;
use crate::object3d::Triangle;
struct Node {
    min_pos: Vector3<f64>,
    max_pos: Vector3<f64>,
    lchild: Option<Box<Node>>,
    rchild: Option<Box<Node>>,
    triangle_index: usize
}

impl Node {
    fn new(min_pos: Vector3<f64>, max_pos: Vector3<f64>, lchild: Option<Box<Node>>, rchild: Option<Box<Node>>, triangle_index: usize) -> Self {
        Self { min_pos, max_pos, lchild, rchild, triangle_index }
    }
}

struct TriangleCompare {
    dimension: u8
}


impl TriangleCompare {
    fn new(dimension: u8) -> Self { Self { dimension } }
    fn compare(&self, a: &TriangleIndex, b: &TriangleIndex) -> Ordering {
        match self.dimension {
            0 => a.min_pos.x().partial_cmp(&b.min_pos.x()).unwrap(),
            1 => a.min_pos.y().partial_cmp(&b.min_pos.y()).unwrap(),
            2 => a.min_pos.z().partial_cmp(&b.min_pos.z()).unwrap(),
            _ => panic!("NodeCompare dimension wrong!")
        }
    }
}

#[derive(Clone, Copy)]
struct TriangleIndex {
    vertices: [usize; 3],
    min_pos: Vector3<f64>,
    max_pos: Vector3<f64>
}

impl TriangleIndex {
    fn new(vertices: [usize; 3], points: &Vec<Vector3<f64>>) -> Self {
        let min_pos = get_min(&get_min(&points[vertices[0]], &points[vertices[1]]), &points[vertices[2]]);
        let max_pos = get_max(&get_max(&points[vertices[0]], &points[vertices[1]]), &points[vertices[2]]);
        Self { vertices, min_pos, max_pos } 
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
    material: Box<dyn Material>,
    // n的计算*有点问题*，我在三角形的实现里是现场算的，这里先不放了
    // 因为我前面也没做好写Texture的准备，所以这里vt也摸了
}

impl Mesh {
    fn new(file_name: &str, material: Box<dyn Material>) -> Self {
        let (models, _) = tobj::load_obj(
            file_name,
            &LoadOptions {
                triangulate: true,
                ..Default::default()
            },
        ).expect("Problem loading model");
        let mesh = &models[0].mesh;
        let mut v: Vec<Vector3<f64>> = Vec::new();
        let mut t: Vec<TriangleIndex> = Vec::new();
        let vn: Option<Vec<Vector3<f64>>>;
        assert!(mesh.positions.len() % 3 == 0);
        for index in 0 .. mesh.positions.len() / 3 {
            v.push(Vector3::<f64>::from([
                mesh.positions[3 * index] as f64,
                mesh.positions[3 * index + 1] as f64,
                mesh.positions[3 * index + 2] as f64
            ]));
        }
        assert!(mesh.indices.len() % 3 == 0);
        for index in 0 .. mesh.indices.len() / 3 {
            t.push(TriangleIndex::new(
                [
                    mesh.indices[3 * index] as usize, 
                    mesh.indices[3 * index + 1] as usize, 
                    mesh.indices[3 * index + 2] as usize
                ],
                &v
            ))
        }
        if mesh.normals.is_empty() == false {
            let mut real_vn: Vec<Vector3<f64>> = Vec::new();
            assert!(mesh.normals.len() % 3 == 0);
            for index in 0 .. mesh.normals.len() / 3 {
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
            root, v, t, vn, material
        }
        // positions每三个代表一个点的位置，对应v
        // normals每三个代表一个点的法向（没有点法向就是空），对应vn
        // indices每三个点代表一个三角形的顶点index（因为triangulate是true所以一定是三个三个），对应t
    }
    fn build(root: &mut Option<Box<Node>>, t: &mut Vec<TriangleIndex>, left: usize, right: usize, dep: usize) {
        if let None = root {
            let mid = (left + right) / 2;
            floydrivest(
                t, 
                mid, left, right, 
                &mut |x, y| { (*COMPS)[dep % 3].compare(x, y) }
            );
            *root = Some(Box::new(Node::new(
                t[mid].min_pos,
                t[mid].max_pos,
                None,
                None,
                mid
            )));
            if left < mid {
                if let Some(real_root) = root {
                    Self::build(&mut real_root.lchild, t, left, mid, dep + 1);
                    if let Some(real_lchild) = &real_root.lchild {
                        real_root.min_pos = get_min(&real_root.min_pos, &real_lchild.min_pos);
                        real_root.max_pos = get_max(&real_root.max_pos, &real_lchild.max_pos);
                    }
                }
            }
            if mid + 1 < right {
                if let Some(real_root) = root {
                    Self::build(&mut real_root.rchild, t, mid + 1, right, dep + 1);
                    if let Some(real_rchild) = &real_root.rchild {
                        real_root.min_pos = get_min(&real_root.min_pos, &real_rchild.min_pos);
                        real_root.max_pos = get_max(&real_root.max_pos, &real_rchild.max_pos);
                    }
                }
            }
        }
    }
    fn query(&self, p: &Option<Box<Node>>, ray: &Ray, tmin: f64) -> Option<Hit> {
        if let Some(real_p) = p {
            let d = ray.get_direction();
            let o = ray.get_origin();
            let mut t_min = tmin;
            let mut t_max = 1e38;
            let min_pos_t = elementwise_division(&(real_p.min_pos - *o), &d);
            let max_pos_t = elementwise_division(&(real_p.max_pos - *o), &d);
            for i in 0 .. 3 as usize {
                if d[i] != 0. {
                    t_min = f64::max(t_min, f64::min(min_pos_t[i], max_pos_t[i]));
                    t_max = f64::min(t_max, f64::max(min_pos_t[i], max_pos_t[i]));
                }
            }
            if t_min > t_max {
                None
            } else {
                let ti = &self.t[real_p.triangle_index];
                let triangle = Triangle::new(
                    self.material.clone_box(),
                    [self.v[ti.vertices[0]], self.v[ti.vertices[1]], self.v[ti.vertices[2]]],
                    if let Some(real_vn) = &self.vn {
                        Some([real_vn[ti.vertices[0]], real_vn[ti.vertices[1]], real_vn[ti.vertices[2]]])
                    } else { None }
                );
                let mut ret = triangle.intersect(ray, tmin);
                let hit = self.query(&real_p.lchild, ray, tmin);
                if let Some(real_hit) = &hit { //hit不为None
                    if let Some(real_ret) = &ret { //ret不为None
                        if real_hit.get_t() < real_ret.get_t() { //hit的getT比ret的更小
                            ret = hit;
                        }
                    } else { //ret是None
                        ret = hit;
                    }
                }
                let hit = self.query(&real_p.rchild, ray, tmin);
                if let Some(real_hit) = &hit { //hit不为None
                    if let Some(real_ret) = &ret { //ret不为None
                        if real_hit.get_t() < real_ret.get_t() { //hit的getT比ret的更小
                            ret = hit;
                        }
                    } else { //ret是None
                        ret = hit;
                    }
                }
                if let Some(real_ret) = ret { Some(real_ret.clone_obj()) } else { None }
            }
        } else {
            None
        }
    }
}

impl Object3d for Mesh {
    fn intersect(&self, ray: &Ray, tmin: f64) -> Option<Hit> {
        self.query(&self.root, ray, tmin)
    }
}

pub fn build_mesh(mesh_attr: &JsonValue, materials: &Vec<Box<dyn Material>>) -> Box<Mesh> {
    let material_index = mesh_attr["MaterialIndex"].as_usize().unwrap();
    let file_name = mesh_attr["File"].as_str().unwrap();
    Box::new(Mesh::new(file_name, materials[material_index].clone_box()))
}