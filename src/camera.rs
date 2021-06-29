use core::f64;

use json::JsonValue;
use vecmat::matrix::Matrix3x3;
use vecmat::{Matrix, Vector, traits::Dot, vector::Vector3};
use rand::{thread_rng, Rng};

use crate::{materials::DiffuseMaterial, ray::Ray};
use crate::utils::parse_vector;
use crate::object3d::{Object3d, Plane};

pub struct PerspectiveCamera {
    center: Vector3::<f64>,
    direction: Vector3::<f64>,
    horizental: Vector3::<f64>,
    up: Vector3::<f64>,
    width: u32,
    height: u32,
    dist: f64,
}

impl PerspectiveCamera {
    pub fn new(center: Vector3::<f64>, direction: Vector3::<f64>, up: Vector3::<f64>, angle: f64, width: u32, height: u32) -> Self {
        direction.normalize();
        let horizental: Vector3::<f64> = direction.cross(up);
        horizental.normalize();
        let up: Vector3::<f64> = horizental.cross(direction);
        let angle = angle / std::f64::consts::PI * 180.0;
        Self { 
            center, direction, horizental, up, 
            width, height,
            dist: height as f64 / (2.0 * f64::tan(angle / 2.0)),
        }
    }
}

pub struct DoFCamera {
    center: Vector3::<f64>,
    direction: Vector3::<f64>,
    horizental: Vector3::<f64>,
    up: Vector3::<f64>,
    width: u32,
    height: u32,
    aperture: f64,
    dist: f64,
    focus_dist: f64
}

impl DoFCamera {
    pub fn new(center: Vector3::<f64>, direction: Vector3::<f64>, up: Vector3::<f64>, angle: f64, width: u32, height: u32, focus: Vector3::<f64>, aperture: f64) -> Self {
        direction.normalize();
        let horizental: Vector3::<f64> = direction.cross(up);
        horizental.normalize();
        let up: Vector3::<f64> = horizental.cross(direction);
        let angle = angle / std::f64::consts::PI * 180.0;
        let focus_dist = focus.dot(direction);
        Self {
            center, direction, horizental, up, 
            width, height, aperture,
            dist: height as f64 / (2.0 * f64::tan(angle / 2.0)),
            focus_dist
        }
    }
}



pub trait Camera {
    fn generate_ray(&self, point: &Vector::<f64, 2>) -> Ray;
}

impl Camera for PerspectiveCamera {
    fn generate_ray(&self, point: &Vector::<f64, 2>) -> Ray {
        let dir = Vector3::<f64>::from([
            point[0] - self.width as f64 / 2.,
            point[1] - self.height as f64 / 2.,
            self.dist
        ]);
        let rot = Matrix::<f64, 3, 3>::from_array_of_vectors([
            self.horizental, self.up, self.direction
        ]).transpose();
        let dir = rot.dot(dir);
        dir.normalize();
        Ray::new(self.center, dir, None)
    }
}

impl Camera for DoFCamera {
    fn generate_ray(&self, point: &Vector::<f64, 2>) -> Ray {
        let mut rng = thread_rng();
        let uniform_x = rng.gen_range(0. .. 1.);
        let uniform_y = rng.gen_range(0. .. 1.);
        let normal_x = f64::sqrt(-2. * f64::log(uniform_x, f64::consts::E)) * f64::cos(2. * f64::consts::PI * uniform_y);
        let normal_y = f64::sqrt(-2. * f64::log(uniform_x, f64::consts::E)) * f64::sin(2. * f64::consts::PI * uniform_y);
        let dir = Vector3::<f64>::from([
            point[0] - self.width as f64 / 2.,
            point[1] - self.height as f64 / 2.,
            self.dist
        ]);
        let rot = Matrix3x3::<f64>::from_array_of_vectors([
            self.horizental, self.up, self.direction
        ]).transpose();
        let dir = rot.dot(dir);
        dir.normalize();
        let temp_ray = Ray::new(self.center, dir, None);
        let temp_material = Box::new(DiffuseMaterial::new(Vector3::<f64>::from([1., 1., 1.])));
        let focus_plane = Plane::new(temp_material, self.direction, self.focus_dist);
        let hit = focus_plane.intersect(&temp_ray, 0.015).unwrap(); //这里保证有交
        let delta = self.aperture * (normal_x * self.horizental + normal_y * self.up);
        let real_dir = temp_ray.point_at_param(hit.get_t()) - (self.center + delta);
        real_dir.normalize();
        Ray::new(self.center + delta, real_dir, None) // 占位用的
    }
}

pub fn build_camera(camera_attr: &JsonValue) -> Box<dyn Camera> {
    let cam_type = String::from(camera_attr["Type"].as_str().unwrap());
    let center = parse_vector(&camera_attr["Center"]);
    let direction = parse_vector(&camera_attr["Direction"]);
    let up = parse_vector(&camera_attr["Up"]);
    let angle = camera_attr["Angle"].as_f64().unwrap();
    let width = camera_attr["Width"].as_u32().unwrap();
    let height = camera_attr["Height"].as_u32().unwrap();
    if cam_type == "Perspective" {
        Box::new(PerspectiveCamera::new(center, direction, up, angle, width, height))
    } else if cam_type == "DoF" {
        let focus = parse_vector(&camera_attr["Focus"]);
        let aperture = camera_attr["Aperture"].as_f64().unwrap();
        Box::new(DoFCamera::new(center, direction, up, angle, width, height, focus, aperture))
    } else {
        panic!("Invalid Camera Type!");
    }
}