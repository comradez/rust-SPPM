use core::f64;
use std::sync::Arc;
use json::JsonValue;
use vecmat::matrix::{Matrix3x3, Matrix4x4};
use vecmat::vector::{Vector3, Vector4};
use vecmat::traits::Dot;
use crate::{hit::Hit, materials::Material, ray::Ray, matrix::parse_vector};
use crate::matrix::{gen_rotate, gen_translation};
use crate::mesh::build_mesh;
pub trait Object3d {
    fn intersect(&self, ray: &Ray, tmin: f64) -> Option<Hit>;
}
pub struct Group {
    group: Vec<Arc<dyn Object3d + Send + Sync>>,
}

impl Group {
    pub fn new() -> Self {
        Self { group: Vec::new() }
    }
    pub fn add_object(&mut self, obj: Arc<dyn Object3d + Send + Sync>) {
        self.group.push(obj);
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
    material: Arc<dyn Material + Send + Sync>,
    normal: Vector3::<f64>,
    d: f64,
}

impl Plane {
    pub fn new(material: Arc<dyn Material + Send + Sync>, normal: Vector3::<f64>, d: f64) -> Self { 
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
            return if t <= tmin { None } else { Some(Hit::new(t, self.material.clone(), self.normal)) };
        }
    }
}

pub struct Sphere {
    material: Arc<dyn Material + Send + Sync>,
    center: Vector3::<f64>,
    radius: f64,
}

impl Sphere {
    pub fn new(material: Arc<dyn Material + Send + Sync>, center: Vector3::<f64>, radius: f64) -> Self {
        Self { material, center, radius }
    }
}

impl Object3d for Sphere {
    fn intersect(&self, ray: &Ray, tmin: f64) -> Option<Hit> {
        let v1 = ray.get_direction();
        let v2 = *ray.get_origin() - self.center;
        let a = v1.square_length();
        let b = 2. * v1.dot(v2);
        let c = v2.square_length() - self.radius * self.radius;
        let delta = b * b - 4. * a * c;
        if delta < 0. {
            None
        } else {
            let qd = f64::sqrt(delta);
            let t1 = (-b - qd) / (2. * a);
            let t2 = (-b + qd) / (2. * a);
            if t1 >= tmin {
                let normal = (ray.point_at_param(t1) - self.center).normalize();
                Some(Hit::new(t1, self.material.clone(), normal))
            } else if t2 >= tmin {
                let normal = (ray.point_at_param(t2) - self.center).normalize();
                Some(Hit::new(t2, self.material.clone(), normal))
            } else {
                None
            }
        }
    }
}

pub struct Triangle {
    material: Arc<dyn Material + Send + Sync>,
    vertices: [Vector3::<f64>; 3],
    normals: Option<[Vector3::<f64>; 3]>,
    face_normal: Vector3::<f64>
} //没有实现纹理贴图所以texcoords先不写了

impl Triangle {
    pub fn new(material: Arc<dyn Material + Send + Sync>, vertices: [Vector3::<f64>; 3], normals: Option<[Vector3::<f64>; 3]>) -> Self { 
        let face_normal: Vector3::<f64> = (vertices[1] - vertices[0]).cross(vertices[2] - vertices[0]).normalize();
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
                }.normalize();
                Some(Hit::new(t, self.material.clone(), norm))
            } else {
                None
            }
        }
    }
}

pub struct Transform {
    object: Arc<dyn Object3d + Send + Sync>, //变形前的对象
    transform: Matrix4x4<f64>,
}

impl Transform {
    pub fn new(object: Arc<dyn Object3d + Send + Sync>, transform: Matrix4x4<f64>) -> Self {
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

pub fn build_group(group_attr: &JsonValue, materials: &Vec<Arc<dyn Material + Send + Sync>>) -> Arc<Group> {
    let mut group = Group::new();
    for object in group_attr.members() {
        group.add_object(build_object3d(object, materials));
    }
    Arc::new(group)
}

pub fn build_plane(plane_attr: &JsonValue, materials: &Vec<Arc<dyn Material + Send + Sync>>) -> Arc<Plane> {
    let material_index = plane_attr["MaterialIndex"].as_usize().unwrap();
    let normal = parse_vector(&plane_attr["Normal"]);
    let d = plane_attr["Offset"].as_f64().unwrap();
    Arc::new(Plane::new(materials[material_index].clone(), normal, d))
}

pub fn build_triangle(triangle_attr: &JsonValue, materials: &Vec<Arc<dyn Material + Send + Sync>>) -> Arc<Triangle> {
    let material_index = triangle_attr["MaterialIndex"].as_usize().unwrap();
    let vertices = &triangle_attr["Vertices"];
    let vertices = [
        parse_vector(&vertices[0]),
        parse_vector(&vertices[1]),
        parse_vector(&vertices[2])
    ];
    let normals: Option<[Vector3<f64>; 3]> = 
        if let JsonValue::Array(point_normals) = &triangle_attr["Normals"] {
            Some([
                parse_vector(&point_normals[0]),
                parse_vector(&point_normals[1]),
                parse_vector(&point_normals[2])
            ])
        } else {
            None
        };
    Arc::new(Triangle::new(materials[material_index].clone(), vertices, normals))
}

pub fn build_sphere(sphere_attr: &JsonValue, materials: &Vec<Arc<dyn Material + Send + Sync>>) -> Arc<Sphere> {
    let material_index = sphere_attr["MaterialIndex"].as_usize().unwrap();
    let center = parse_vector(&sphere_attr["Center"]);
    let radius = sphere_attr["Radius"].as_f64().unwrap();
    Arc::new(Sphere::new(materials[material_index].clone(), center, radius))
}

pub fn build_transform(transform_attr: &JsonValue, materials: &Vec<Arc<dyn Material + Send + Sync>>) -> Arc<Transform> {
    let mut matrix: Matrix4x4<f64> = Matrix4x4::diagonal(Vector4::<f64>::from([1., 1., 1., 1.]));
    let object: Arc<dyn Object3d + Send + Sync> = build_object3d(transform_attr, materials);
    for process in transform_attr.members() {
        let process_type = process["Type"].as_str().unwrap();
        match process_type {
            "Scale" => {
                let scales = &process["Scales"];
                matrix = matrix.dot(Matrix4x4::diagonal(Vector4::<f64>::from([
                    scales[0].as_f64().unwrap(),
                    scales[1].as_f64().unwrap(),
                    scales[2].as_f64().unwrap(),
                    1.
                ])));
            },
            "UniformScale" => {
                let scale = process["Scale"].as_f64().unwrap();
                matrix = matrix.dot(Matrix4x4::diagonal(Vector4::<f64>::from([
                    scale, scale, scale, 1.
                ])));
            },
            "Translate" => {
                let translation = gen_translation(&parse_vector(&process["Translation"]));
                matrix = matrix.dot(translation);
            },
            "XRotate" => {
                let degree = process["Degree"].as_f64().unwrap();
                matrix = matrix.dot(gen_rotate(degree, 0));
            },
            "YRotate" => {
                let degree = process["Degree"].as_f64().unwrap();
                matrix = matrix.dot(gen_rotate(degree, 1));
            },
            "ZRotate" => {
                let degree = process["Degree"].as_f64().unwrap();
                matrix = matrix.dot(gen_rotate(degree, 2));
            },
            _ => panic!("Wrong process type.")
        }
    }
    Arc::new(Transform::new(object, matrix))
}

pub fn build_object3d(object_attr: &JsonValue, materials: &Vec<Arc<dyn Material + Send + Sync>>) -> Arc<dyn Object3d + Send + Sync> {
    let object_type = object_attr["Type"].as_str().unwrap();
    match object_type {
        "Group" => build_group(object_attr, materials),
        "Plane" => build_plane(object_attr, materials),
        "Triangle" => build_triangle(object_attr, materials),
        "Sphere" => build_sphere(object_attr, materials),
        "Transform" => build_transform(object_attr, materials),
        "Mesh" => build_mesh(object_attr, materials),
        _ => panic!("Wrong object type")
    }
}