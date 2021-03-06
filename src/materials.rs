use crate::{
    ray::Ray,
    utils::{gen_vert, parse_vector},
};
use core::f64;
use json::JsonValue;
use rand::{thread_rng, Rng};
use std::sync::Arc;
use vecmat::{traits::Dot, vector::Vector3, Vector};

pub trait Material {
    fn bsdf(
        &self,
        ray: &mut Ray,
        norm: &Vector3<f64>,
        pos: &Vector3<f64>,
        russian_roulette: bool,
    ) -> bool;
    fn get_type(&self) -> &MaterialType;
    fn get_color(&self) -> &Vector3<f64>;
}

#[derive(Clone, Copy)]
pub enum MaterialType {
    Diffuse,
    Specular,
    Refraction,
}

#[derive(Clone, Copy)]
pub struct DiffuseMaterial {
    color: Vector3<f64>,
    material_type: MaterialType,
}

impl DiffuseMaterial {
    pub fn new(color: Vector3<f64>) -> Self {
        Self {
            color,
            material_type: MaterialType::Diffuse,
        }
    }
}

impl Material for DiffuseMaterial {
    fn bsdf(
        &self,
        ray: &mut Ray,
        norm: &Vector3<f64>,
        pos: &Vector3<f64>,
        russian_roulette: bool,
    ) -> bool {
        let mut rng = thread_rng();
        let mut flux = *ray.get_flux();
        if russian_roulette {
            let h = self.color.max();
            if rng.gen_range(0. ..1.) > h {
                return false;
            } else {
                flux = flux.map(|x| x / h);
            }
        }
        let x_axis = gen_vert(norm);
        let y_axis: Vector3<f64> = x_axis.cross(*norm).normalize();
        let theta: f64 = 2. * f64::consts::PI * rng.gen_range(0. ..1.);
        let phi: f64 = f64::acos(2. * rng.gen_range(0. ..1.) - 1.);
        let direction_out = f64::cos(theta) * f64::sin(phi) * x_axis
            + f64::sin(theta) * f64::sin(phi) * y_axis
            + f64::cos(phi) * *norm;
        ray.set(*pos, direction_out, flux * self.color);
        true
    }
    fn get_type(&self) -> &MaterialType {
        &self.material_type
    }
    fn get_color(&self) -> &Vector3<f64> {
        &self.color
    }
}

#[derive(Clone, Copy)]
pub struct SpecularMaterial {
    color: Vector3<f64>,
    material_type: MaterialType,
}

impl SpecularMaterial {
    pub fn new(color: Vector3<f64>) -> Self {
        Self {
            color,
            material_type: MaterialType::Specular,
        }
    }
}

impl Material for SpecularMaterial {
    fn bsdf(
        &self,
        ray: &mut Ray,
        norm: &Vector3<f64>,
        pos: &Vector3<f64>,
        russian_roulette: bool,
    ) -> bool {
        let mut rng = thread_rng();
        let direction_in = *ray.get_direction();
        let mut flux = *ray.get_flux();
        if russian_roulette {
            let h = self.color.max();
            if rng.gen_range(0. ..1.) > h {
                return false;
            } else {
                flux = flux.map(|x| x / h);
            }
        }
        ray.set(
            *pos,
            direction_in - 2. * norm.dot(direction_in) * *norm,
            flux * self.color,
        );
        true
    }
    fn get_type(&self) -> &MaterialType {
        &self.material_type
    }
    fn get_color(&self) -> &Vector3<f64> {
        &self.color
    }
}

#[derive(Clone, Copy)]
pub struct RefractionMaterial {
    color: Vector3<f64>,
    refr_index: f64,
    material_type: MaterialType,
}

impl RefractionMaterial {
    pub fn new(color: Vector3<f64>, refr_index: Option<f64>) -> Self {
        Self {
            color,
            refr_index: refr_index.unwrap_or(1.5),
            material_type: MaterialType::Refraction,
        }
    }
}

impl Material for RefractionMaterial {
    fn bsdf(
        &self,
        ray: &mut Ray,
        norm: &Vector3<f64>,
        pos: &Vector3<f64>,
        russian_roulette: bool,
    ) -> bool {
        let mut rng = thread_rng();
        let direction_in = ray.get_direction();
        let mut flux = *ray.get_flux();
        if russian_roulette {
            let h = self.color.max();
            if rng.gen_range(0. ..1.) > h {
                return false;
            } else {
                flux = flux.map(|x| x / h);
            }
        }
        let refl_d: Vector<f64, 3> = *direction_in - 2. * norm.dot(*direction_in) * *norm;
        let nl = if norm.dot(*direction_in) < 0. {
            *norm
        } else {
            -1. * *norm
        };
        let into = norm.dot(nl) > 0.;
        let into_dir = if into { 1. } else { -1. };
        let ratio = if into {
            1. / self.refr_index
        } else {
            self.refr_index
        };
        let proj = direction_in.dot(nl);
        let cos_out_sqr = 1. - ratio * ratio * (1. - proj * proj);
        if cos_out_sqr < 0. {
            // ?????????
            ray.set(*pos, refl_d, flux * self.color);
        } else {
            let refr_d =
                ratio * *direction_in - *norm * into_dir * (proj * ratio + f64::sqrt(cos_out_sqr));
            let refr_d = refr_d.normalize();
            let r_0 = (self.refr_index - 1.) * (self.refr_index - 1.)
                / (self.refr_index + 1.)
                / (self.refr_index + 1.);
            let c = 1. - if into { -1. * proj } else { refr_d.dot(*norm) };
            let r_e = r_0 + (1. - r_0) * c * c * c * c * c;
            if rng.gen_range(0. ..1.) < r_e {
                ray.set(*pos, refl_d, flux * self.color);
            } else {
                ray.set(*pos, refr_d, flux * self.color);
            }
        }
        true
    }
    fn get_type(&self) -> &MaterialType {
        &self.material_type
    }
    fn get_color(&self) -> &Vector3<f64> {
        &self.color
    }
}

pub fn build_material(material_attr: &JsonValue) -> Arc<dyn Material + Send + Sync> {
    let material_type = material_attr["Type"].as_str().unwrap();
    let color = parse_vector(&material_attr["Color"]);
    match material_type {
        "DIFF" => Arc::new(DiffuseMaterial::new(color)),
        "SPEC" => Arc::new(SpecularMaterial::new(color)),
        "REFR" => Arc::new(RefractionMaterial::new(color, None)),
        _ => panic!("Wrong material type!"),
    }
}
