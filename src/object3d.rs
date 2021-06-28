use core::f64;
use std::usize;

use vecmat::matrix::{Matrix3x3, Matrix4x4};
use vecmat::prelude::NormL2;
use vecmat::vector::{Vector3, Vector4};
use vecmat::traits::Dot;

use crate::{hit::Hit, materials::Material, ray::Ray};
pub trait Object3d {
    fn intersect(&self, ray: &Ray, tmin: f64) -> Option<Hit>;
}
pub struct Group {
    group: Vec<Box<dyn Object3d>>
}

impl Group {
    pub fn new() -> Self {
        Self { group: Vec::new() }
    }
    pub fn add_object(&mut self, obj: Box<dyn Object3d>) {
        self.group.push(obj);
    }
    pub fn get_size(&self) -> usize {
        self.group.len()
    }
}

impl Object3d for Group {
    fn intersect(&self, ray: &Ray, tmin: f64) -> Option<Hit> {
        let mut ret: Option<Hit> = None;
        for object in &self.group[..] {
            let hit = object.intersect(ray, tmin);
            if let Some(real_hit) = &hit { //hit不为None
                if let Some(real_ret) = &ret { //ret不为None
                    if real_hit.get_t() < real_ret.get_t() { //hit的getT比ret的更小
                        ret = hit;
                    }
                } else { //ret是None
                    ret = hit;
                }
            }
        }
        ret
    }
}

pub struct Plane {
    material: Box<dyn Material>,
    normal: Vector3::<f64>,
    d: f64,
}

impl Plane {
    pub fn new(material: Box<dyn Material>, normal: Vector3::<f64>, d: f64) -> Self { 
        Self { material, normal, d }
    }
}

impl Object3d for Plane {
    fn intersect(&self, ray: &Ray, tmin: f64) -> Option<Hit> {
        let z1 = self.normal.dot(*ray.get_origin());
        let z2 = self.normal.dot(*ray.get_direction());
        if f64::abs(z2) <= 1e-5 {
            return None;
        } else {
            let t = (self.d - z1) / z2;
            return if t <= tmin { None } else { Some(Hit::new(t, &self.material, self.normal)) };
        }
    }
}

pub struct Sphere {
    material: Box<dyn Material>,
    center: Vector3::<f64>,
    radius: f64,
}

impl Sphere {
    pub fn new(material: Box<dyn Material>, center: Vector3::<f64>, radius: f64) -> Self {
        Self { material, center, radius }
    }
}

impl Object3d for Sphere {
    fn intersect(&self, ray: &Ray, tmin: f64) -> Option<Hit> {
        let v1 = ray.get_direction();
        let v2 = ray.get_origin();
        let a = v1.norm_l2_sqr();
        let b = 2. * v1.dot(*v2);
        let c = v2.norm_l2_sqr() - self.radius * self.radius;
        let delta = b * b - 4. * a * c;
        if delta < 0. {
            None
        } else {
            let qd = f64::sqrt(delta);
            let t1 = (-b - qd) / (2. * a);
            let t2 = (-b + qd) / (2. * a);
            if t1 >= tmin {
                let normal = ray.point_at_param(t1) - self.center;
                normal.normalize();
                Some(Hit::new(t1, &self.material, normal))
            } else if t2 >= tmin {
                let normal = ray.point_at_param(t2) - self.center;
                normal.normalize();
                Some(Hit::new(t2, &self.material, normal))
            } else {
                None
            }
        }
    }
}

pub struct Triangle {
    material: Box<dyn Material>,
    vertices: [Vector3::<f64>; 3],
    normals: Option<[Vector3::<f64>; 3]>,
    face_normal: Vector3::<f64>
} //没有实现纹理贴图所以texcoords先不写了

impl Triangle {
    pub fn new(material: Box<dyn Material>, vertices: [Vector3::<f64>; 3], normals: Option<[Vector3::<f64>; 3]>) -> Self { 
        let face_normal: Vector3::<f64> = (vertices[1] - vertices[0]).cross(vertices[2] - vertices[0]);
        face_normal.normalize();
        Self { material, vertices, normals, face_normal } 
    }
}

impl Object3d for Triangle {
    fn intersect(&self, ray: &Ray, tmin: f64) -> Option<Hit> {
        let mat = Matrix3x3::from_array_of_vectors([
            ray.get_direction().clone(),
            self.vertices[0] - self.vertices[1],
            self.vertices[0] - self.vertices[2]
        ]).transpose();
        if mat.det() == 0. { //没有奇异性判定，用det为0来判定
            None
        } else {
            let x = mat.inv().dot(self.vertices[0] - *ray.get_origin());
            // 这里它不能知道x的长度是3，所以没办法用tuple赋值，有点不爽 
            let t = x[0];
            let beta = x[1];
            let gamma  = x[2];
            if t > tmin && 0. <= beta && 0. <= gamma && beta + gamma <= 1. {
                let point = ray.point_at_param(t);
                let w0: Vector3::<f64> = (self.vertices[1] - point).cross(self.vertices[2] - point);
                let w1: Vector3::<f64> = (self.vertices[2] - point).cross(self.vertices[0] - point);
                let w2: Vector3::<f64> = (self.vertices[0] - point).cross(self.vertices[1] - point);
                let w0 = self.face_normal.dot(w0);
                let w1 = self.face_normal.dot(w1);
                let w2 = self.face_normal.dot(w2);
                //重心坐标插值
                let norm = if let Some(normals) = self.normals {
                    normals[0] * w0 + normals[1] * w1 + normals[2] * w2
                } else {
                    self.face_normal
                };
                norm.normalize();
                Some(Hit::new(t, &self.material, norm))
            } else {
                None
            }
        }
    }
}

pub struct Transform {
    object: Box<dyn Object3d>, //变形前的对象
    transform: Matrix4x4<f64>,
}

impl Transform {
    pub fn new(object: Box<dyn Object3d>, transform: Matrix4x4<f64>) -> Self {
        Self { object, transform: transform.inv() }
    }
}

fn transform_point(mat: &Matrix4x4<f64>, point: &Vector3<f64>) -> Vector3<f64> {
    let point = mat.dot(Vector4::<f64>::from([
        point.x(),
        point.y(),
        point.z(),
        1.
    ]));
    Vector3::<f64>::from([point[0], point[1], point[2]])
}

fn transform_direction(mat: &Matrix4x4<f64>, dir: &Vector3<f64>) -> Vector3<f64> {
    let dir = mat.dot(Vector4::<f64>::from([
        dir.x(),
        dir.y(),
        dir.z(),
        0.
    ]));
    Vector3::<f64>::from([dir[0], dir[1], dir[2]])
}

impl Object3d for Transform {
    fn intersect(&self, ray: &Ray, tmin: f64) -> Option<Hit> {
        let tr_source = transform_point(&self.transform, ray.get_origin());
        let tr_direction = transform_direction(&self.transform, ray.get_direction());
        let ray = Ray::new(tr_source, tr_direction, Some(*ray.get_flux()));
        self.object.intersect(&ray, tmin)
    }
}