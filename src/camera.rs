use core::f64;
use json::JsonValue;
use rand::{thread_rng, Rng};
use std::sync::Arc;
use vecmat::{matrix::Matrix3x3, traits::Dot, vector::Vector3, Matrix, Vector};

use crate::object3d::{Object3d, Plane};
use crate::utils::parse_vector;
use crate::{materials::DiffuseMaterial, ray::Ray};

pub struct PerspectiveCamera {
    center: Vector3<f64>,
    direction: Vector3<f64>,
    horizontal: Vector3<f64>,
    up: Vector3<f64>,
    width: u32,
    height: u32,
    dist: f64,
}

impl PerspectiveCamera {
    pub fn new(
        center: Vector3<f64>,
        direction: Vector3<f64>,
        up: Vector3<f64>,
        angle: f64,
        width: u32,
        height: u32,
    ) -> Self {
        let direction = direction.normalize();
        let horizontal: Vector3<f64> = direction.cross(up);
        let horizontal = horizontal.normalize();
        let up: Vector3<f64> = horizontal.cross(direction);
        let angle = angle * std::f64::consts::PI / 180.0;
        Self {
            center,
            direction,
            horizontal,
            up,
            width,
            height,
            dist: height as f64 / (2.0 * f64::tan(angle / 2.0)),
        }
    }
}

pub struct DoFCamera {
    center: Vector3<f64>,
    direction: Vector3<f64>,
    horizontal: Vector3<f64>,
    up: Vector3<f64>,
    width: u32,
    height: u32,
    aperture: f64,
    dist: f64,
    focus_dist: f64,
}

impl DoFCamera {
    pub fn new(
        center: Vector3<f64>,
        direction: Vector3<f64>,
        up: Vector3<f64>,
        angle: f64,
        width: u32,
        height: u32,
        focus: Vector3<f64>,
        aperture: f64,
    ) -> Self {
        let direction = direction.normalize();
        let horizontal: Vector3<f64> = direction.cross(up);
        let horizontal = horizontal.normalize();
        let up: Vector3<f64> = horizontal.cross(direction);
        let angle = angle * std::f64::consts::PI / 180.0;
        let focus_dist = focus.dot(direction);
        Self {
            center,
            direction,
            horizontal,
            up,
            width,
            height,
            aperture,
            dist: height as f64 / (2.0 * f64::tan(angle / 2.0)),
            focus_dist,
        }
    }
}

pub trait Camera {
    fn generate_ray(&self, point: &Vector<f64, 2>) -> Ray;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

impl Camera for PerspectiveCamera {
    fn generate_ray(&self, point: &Vector<f64, 2>) -> Ray {
        let dir = Vector3::<f64>::from([
            point[0] - self.width as f64 / 2.,
            point[1] - self.height as f64 / 2.,
            self.dist,
        ]);
        let rot =
            Matrix::<f64, 3, 3>::from_array_of_vectors([self.horizontal, self.up, self.direction])
                .transpose();
        let dir = rot.dot(dir).normalize();
        Ray::new(self.center, dir, Some(Vector3::<f64>::from([1., 1., 1.])))
    }
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
}

impl Camera for DoFCamera {
    fn generate_ray(&self, point: &Vector<f64, 2>) -> Ray {
        let mut rng = thread_rng();
        let uniform_x = rng.gen_range(0. ..1.);
        let uniform_y = rng.gen_range(0. ..1.);
        let normal_x = f64::sqrt(-2. * f64::log(uniform_x, f64::consts::E))
            * f64::cos(2. * f64::consts::PI * uniform_y);
        let normal_y = f64::sqrt(-2. * f64::log(uniform_x, f64::consts::E))
            * f64::sin(2. * f64::consts::PI * uniform_y);
        let dir = Vector3::<f64>::from([
            point[0] - self.width as f64 / 2.,
            point[1] - self.height as f64 / 2.,
            self.dist,
        ]);
        let rot =
            Matrix3x3::<f64>::from_array_of_vectors([self.horizontal, self.up, self.direction])
                .transpose();
        let dir = rot.dot(dir).normalize();
        let temp_ray = Ray::new(self.center, dir, None);
        let temp_material = Arc::new(DiffuseMaterial::new(Vector3::<f64>::from([1., 1., 1.])));
        let focus_plane = Plane::new(temp_material, self.direction, self.focus_dist);
        let hit = focus_plane.intersect(&temp_ray, 0.015).unwrap(); //??????????????????
        let delta: Vector3<f64> = self.aperture * (normal_x * self.horizontal + normal_y * self.up);
        let real_dir: Vector3<f64> = temp_ray.point_at_param(hit.get_t()) - (self.center + delta);
        let real_dir = real_dir.normalize();
        Ray::new(
            self.center + delta,
            real_dir,
            Some(Vector3::<f64>::from([1., 1., 1.])),
        )
    }
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
}

pub fn build_camera(camera_attr: &JsonValue) -> Arc<dyn Camera + Send + Sync> {
    let cam_type = camera_attr["Type"].as_str().unwrap();
    let center = parse_vector(&camera_attr["Center"]);
    let direction = parse_vector(&camera_attr["Direction"]);
    let up = parse_vector(&camera_attr["Up"]);
    let angle = camera_attr["Angle"].as_f64().unwrap();
    let width = camera_attr["Width"].as_u32().unwrap();
    let height = camera_attr["Height"].as_u32().unwrap();
    match cam_type {
        "Perspective" => Arc::new(PerspectiveCamera::new(
            center, direction, up, angle, width, height,
        )),
        "DoF" => {
            let focus = parse_vector(&camera_attr["Focus"]);
            let aperture = camera_attr["Aperture"].as_f64().unwrap();
            Arc::new(DoFCamera::new(
                center, direction, up, angle, width, height, focus, aperture,
            ))
        }
        _ => panic!("Invalid Camera Type!"),
    }
}
