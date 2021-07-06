use core::f64;
use std::sync::Arc;
use vecmat::vector::Vector3;
use json::JsonValue;
use crate::ray::Ray;
use crate::matrix::{gen_vert, parse_vector};
use rand::{thread_rng, Rng};

pub trait Light {
    fn get_ray(&self) -> Ray;
}

pub struct SphereLight {
    scale: f64,
    pos: Vector3::<f64>,
    flux: Vector3::<f64>
}

impl SphereLight {
    pub fn new(scale: Option<f64>, pos: Vector3::<f64>, flux: Vector3::<f64>) -> Self {
        Self {
            scale: scale.unwrap_or(1.),
            pos, flux
        }
    }
}

impl Light for SphereLight {
    fn get_ray(&self) -> Ray {
        let mut rng = thread_rng();
        let theta: f64 = rng.gen_range(0.0 .. 1.0) * 2. * std::f64::consts::PI;
        let phi = f64::acos(rng.gen_range(0.0 .. 1.0) * 2. - 1.);
        let direction = Vector3::<f64>::from([
            f64::cos(theta) * f64::sin(phi),
            f64::sin(theta) * f64::sin(phi),
            f64::cos(phi)
        ]);
        Ray::new(self.pos, direction, Some(self.flux * self.scale))
    }
}

pub struct ConeLight {
    scale: f64,
    pos: Vector3::<f64>,
    norm: Vector3::<f64>,
    flux: Vector3::<f64>,
    x_axis: Vector3::<f64>,
    y_axis: Vector3::<f64>,
    angle: f64
}

impl ConeLight {
    pub fn new(scale: Option<f64>, pos: Vector3::<f64>, norm: Vector3::<f64>, flux: Vector3::<f64>, angle: f64) -> Self {
        let x_axis = gen_vert(&norm);
        let y_axis: Vector3::<f64> = x_axis.cross(norm).normalize();
        Self {
            scale: scale.unwrap_or(1.),
            pos, norm, flux, x_axis, y_axis, angle
        }
    }
}

impl Light for ConeLight {
    fn get_ray(&self) -> Ray {
        let mut rng = thread_rng();
        let theta: f64 = rng.gen_range(0.0 .. 1.0) * 2. * std::f64::consts::PI;
        let cos_lower = f64::cos(self.angle);
        let phi = f64::acos(cos_lower + (1. - cos_lower) * rng.gen_range(0.0 .. 1.0));
        let direction = 
            f64::cos(theta) * f64::sin(phi) * self.x_axis +
            f64::sin(theta) * f64::sin(phi) * self.y_axis +
            f64::cos(phi) * self.norm;
        Ray::new(self.pos, direction, Some(self.flux * self.scale))
    }
}

pub struct DirectionCircleLight {
    scale: f64,
    pos: Vector3::<f64>,
    norm: Vector3::<f64>,
    flux: Vector3::<f64>,
    x_axis: Vector3::<f64>,
    y_axis: Vector3::<f64>,
    radius: f64
}

impl DirectionCircleLight {
    pub fn new(scale: Option<f64>, pos: Vector3::<f64>, norm: Vector3::<f64>, flux: Vector3::<f64>, radius: f64) -> Self {
        let x_axis = gen_vert(&norm);
        let y_axis: Vector3::<f64> = x_axis.cross(norm).normalize();
        Self {
            scale: scale.unwrap_or(1.),
            pos, norm, flux, x_axis, y_axis, radius
        }
    }
}

impl Light for DirectionCircleLight {
    fn get_ray(&self) -> Ray {
        let mut rng = thread_rng();
        let r = self.radius * f64::sqrt(rng.gen_range(0. .. 1.));
        let theta = 2. * f64::consts::PI * rng.gen_range(0. .. 1.);
        Ray::new(
            self.pos + r * f64::cos(theta) * self.x_axis + r * f64::sin(theta) * self.y_axis,
            self.norm,
            Some(self.flux * self.scale)
        )
    }
}

pub fn build_light(light_attr: &JsonValue) -> Arc<dyn Light + Send + Sync> {
    let light_type = light_attr["Type"].as_str().unwrap();
    let scale = light_attr["Scale"].as_f64().unwrap();
    let pos = parse_vector(&light_attr["Position"]);
    let flux = parse_vector(&light_attr["Flux"]);
    match light_type {
        "SphereLiht" => Arc::new(SphereLight::new(Some(scale), pos, flux)),
        "ConeLight" => {
            let normal = parse_vector(&light_attr["Normal"]);
            let angle = light_attr["Angle"].as_f64().unwrap();
            Arc::new(ConeLight::new(Some(scale), pos, normal, flux, angle))
        },
        "HalfSphereLight" => {
            let normal = parse_vector(&light_attr["Normal"]);
            Arc::new(ConeLight::new(Some(scale), pos, normal, flux, 90.))
        },
        "DirectionCircleLight" => {
            let normal = parse_vector(&light_attr["Normal"]);
            let radius = light_attr["Radius"].as_f64().unwrap();
            Arc::new(DirectionCircleLight::new(Some(scale), pos, normal, flux, radius))
        },
        _ => {
            panic!("Wrong light type!");
        }
    }
}