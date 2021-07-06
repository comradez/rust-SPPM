use adqselect::nth_element;
use core::f64;
use std::cmp::Ordering;

use crate::matrix::{get_dist, get_max, get_min};
use lazy_static::lazy_static;
use vecmat::{traits::Dot, vector::Vector3};
#[derive(Clone, Copy)]
pub struct Photon {
    pub pos: Vector3<f64>,
    pub dir: Vector3<f64>,
    pub norm: Vector3<f64>,
    pub flux: Vector3<f64>,
}

impl Photon {
    pub fn new(
        pos: Vector3<f64>,
        dir: Vector3<f64>,
        norm: Vector3<f64>,
        flux: Vector3<f64>,
    ) -> Self {
        Self {
            pos,
            dir,
            norm,
            flux,
        }
    }
}

pub static ALPHA: f64 = 0.7;

#[derive(Clone, Copy)]
pub struct HitPoint {
    pub radius: f64,
    pub n: f64,
    pub tau: Vector3<f64>,
    pub pos: Option<Vector3<f64>>,
}

impl HitPoint {
    pub fn new() -> Self {
        let radius = 0.5;
        let n = 0.;
        let tau = Vector3::<f64>::from([0., 0., 0.]);
        let pos = None;
        Self {
            radius,
            n,
            tau,
            pos,
        }
    }
}

pub struct Node {
    min_pos: Vector3<f64>,
    max_pos: Vector3<f64>,
    lchild: Option<Box<Node>>,
    rchild: Option<Box<Node>>,
    photon_index: usize,
}

impl Node {
    fn new(
        min_pos: Vector3<f64>,
        max_pos: Vector3<f64>,
        lchild: Option<Box<Node>>,
        rchild: Option<Box<Node>>,
        photon_index: usize,
    ) -> Self {
        Self {
            min_pos,
            max_pos,
            lchild,
            rchild,
            photon_index,
        }
    }
}

struct PhotonCompare {
    dimension: usize,
}

impl PhotonCompare {
    fn new(dimension: usize) -> Self {
        Self { dimension }
    }
    fn compare(&self, a: &Photon, b: &Photon) -> Ordering {
        match self.dimension {
            0 => a.pos.x().partial_cmp(&b.pos.x()).unwrap(),
            1 => a.pos.y().partial_cmp(&b.pos.y()).unwrap(),
            2 => a.pos.z().partial_cmp(&b.pos.z()).unwrap(),
            _ => panic!("PhotonCompare dimension wrong!"),
        }
    }
}

lazy_static! {
    static ref COMPS: [PhotonCompare; 3] = [
        PhotonCompare::new(0),
        PhotonCompare::new(1),
        PhotonCompare::new(2)
    ];
}

pub struct KDTree {
    root: Option<Box<Node>>,
    map: Vec<Photon>, //从主函数拿到所有权就可以了，后面不会再用
}

impl KDTree {
    pub fn new(mut map: Vec<Photon>) -> Self {
        let mut root: Option<Box<Node>> = None;
        let len = map.len();
        Self::build(&mut root, &mut map, 0, len, 0);
        Self { root, map }
    }
    fn build(
        root: &mut Option<Box<Node>>,
        map: &mut Vec<Photon>,
        left: usize,
        right: usize,
        dep: usize,
    ) {
        if root.is_none() {
            // println!("kdtree build: left is {} and right is {}", &left, &right);
            let mid = (left + right) / 2;
            let relative_mid = (right - left) / 2;
            nth_element(&mut map[left..right], relative_mid, &mut |x, y| {
                (*COMPS)[dep % 3].compare(x, y)
            });
            // println!("partition complete: left is {} and right is {}", &left, &right);
            *root = Some(Box::new(Node::new(
                map[mid].pos,
                map[mid].pos,
                None,
                None,
                mid,
            )));
            if left < mid {
                if let Some(root) = root {
                    Self::build(&mut root.lchild, map, left, mid, dep + 1);
                    if let Some(lchild) = &root.lchild {
                        root.min_pos = get_min(&root.min_pos, &lchild.min_pos);
                        root.max_pos = get_max(&root.max_pos, &lchild.max_pos);
                    }
                }
            }
            if mid + 1 < right {
                if let Some(root) = root {
                    Self::build(&mut root.rchild, map, mid + 1, right, dep + 1);
                    if let Some(rchild) = &root.rchild {
                        root.min_pos = get_min(&root.min_pos, &rchild.min_pos);
                        root.max_pos = get_max(&root.max_pos, &rchild.max_pos);
                    }
                }
            }
        }
    }
    fn intersect(p: &Node, hitpoint: &HitPoint) -> bool {
        hitpoint.radius
            >= f64::sqrt(
                get_dist(p.min_pos.x(), p.max_pos.x(), hitpoint.pos.unwrap().x())
                    * get_dist(p.min_pos.x(), p.max_pos.x(), hitpoint.pos.unwrap().x())
                    + get_dist(p.min_pos.y(), p.max_pos.y(), hitpoint.pos.unwrap().y())
                        * get_dist(p.min_pos.y(), p.max_pos.y(), hitpoint.pos.unwrap().y())
                    + get_dist(p.min_pos.z(), p.max_pos.z(), hitpoint.pos.unwrap().z())
                        * get_dist(p.min_pos.z(), p.max_pos.z(), hitpoint.pos.unwrap().z()),
            )
    }
    pub fn query(
        &self,
        p: &Option<Box<Node>>,
        hitpoint: &mut HitPoint,
        color: &Vector3<f64>,
        normal: &Vector3<f64>,
        scale: &Vector3<f64>,
    ) {
        let hit_pos = hitpoint.pos.unwrap();
        if let Some(p) = p {
            if Self::intersect(p, hitpoint) {
                let point = &self.map[p.photon_index];
                let dist = point.pos - hit_pos;
                if dist.square_length() <= hitpoint.radius {
                    hitpoint.n += 1.;
                    if normal.dot(point.dir) < 0. {
                        hitpoint.tau += *color * point.flux * *scale / f64::consts::PI
                    }
                }
                self.query(&p.lchild, hitpoint, color, normal, scale);
                self.query(&p.rchild, hitpoint, color, normal, scale);
            }
        }
    }
    pub fn search(
        &self,
        hitpoint: &mut HitPoint,
        color: &Vector3<f64>,
        normal: &Vector3<f64>,
        scale: &Vector3<f64>,
    ) {
        self.query(&self.root, hitpoint, color, normal, scale)
    }
}
