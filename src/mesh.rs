use vecmat::vector::Vector3;
use vecmat::Vector;
use tobj::{self, LoadOptions};
use tobj::Mesh as objMesh;

use crate::materials::Material;
use crate::matrix::get_max;
use crate::matrix::get_min;
struct Node {
    min_pos: Vector3<f64>,
    max_pos: Vector3<f64>,
    lchild: Option<Box<Node>>,
    rchild: Option<Box<Node>>,
    triangle_index: usize,
    dimension: u8,
}

impl Node {
    fn new(min_pos: Vector3<f64>, max_pos: Vector3<f64>, lchild: Option<Box<Node>>, rchild: Option<Box<Node>>, triangle_index: usize, dimension: u8) -> Self {
        Self { min_pos, max_pos, lchild, rchild, triangle_index, dimension }
    }
}

struct NodeCompare {
    dimension: u8
}

impl NodeCompare {
    fn new(dimension: u8) -> Self { Self { dimension } }
    fn compare(&self, a: &Node, b: &Node) -> bool {
        match self.dimension {
            0 => a.min_pos.x() < b.min_pos.x(),
            1 => a.min_pos.y() < b.min_pos.y(),
            2 => a.min_pos.z() < b.min_pos.z(),
            _ => panic!("NodeCompare dimension wrong!")
        }
    }
}

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

pub struct Mesh {
    comps: [NodeCompare; 3],
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
        let (models, materials) = tobj::load_obj(
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
        Self {
            comps: [NodeCompare::new(0), NodeCompare::new(1), NodeCompare::new(2)],
            root: None,
            v, t, vn, material
        }
        // positions每三个代表一个点的位置，对应v
        // normals每三个代表一个点的法向（没有点法向就是空），对应vn
        // indices每三个点代表一个三角形的顶点index（因为triangulate是true所以一定是三个三个），对应t
    }
}