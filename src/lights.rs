pub mod lights {
    use core::f64;

    use vecmat::prelude::NormL1;
    use vecmat::vector::Vector3;
    use crate::ray::ray::Ray;
    use crate::utils::utils::{gen_vert, parse_vector};
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
            let y_axis: Vector3::<f64> = x_axis.cross(norm);
            y_axis.normalize();
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
            let y_axis: Vector3::<f64> = x_axis.cross(norm);
            y_axis.normalize();
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

    pub fn build_light(light_attr: &mut json::JsonValue) -> Box<dyn Light> {
        let light_type = light_attr.remove("Type").take_string().unwrap();
        let scale = light_attr.remove("Scale").as_f64().unwrap();
        let pos = parse_vector(light_attr.remove("Position"));
        let flux = parse_vector(light_attr.remove("Flux"));
        if light_type == "SphereLight" {
            Box::new(SphereLight::new(Some(scale), pos, flux))
        } else if light_type == "ConeLight" {
            let normal = parse_vector(light_attr.remove("Normal"));
            let angle = light_attr.remove("Angle").as_f64().unwrap();
            Box::new(ConeLight::new(Some(scale), pos, normal, flux, angle))
        } else if light_type == "DirectionCircleLight" {
            let normal = parse_vector(light_attr.remove("Normal"));
            let radius = light_attr.remove("Radius").as_f64().unwrap();
            Box::new(DirectionCircleLight::new(Some(scale), pos, normal, flux, radius))
        } else {
            panic!("Wrong light type!");
        }
    }
}