pub mod materials {
    use core::f64;

    use vecmat::{Vector, vector::Vector3};
    use crate::ray::ray::Ray;
    use crate::utils::utils::gen_vert;
    use rand::{thread_rng, Rng};
    pub trait Material {
        fn BSDF(&self, ray: &mut Ray, norm: &Vector3::<f64>, pos: &Vector3::<f64>, russian_roulette: bool) -> bool;
    }

    pub struct DiffuseMaterial {
        color: Vector3::<f64>
    }  

    impl DiffuseMaterial {
        pub fn new(color: Vector3::<f64>) -> Self {
            Self {
                color
            }
        }
    }

    impl Material for DiffuseMaterial {
        fn BSDF(&self, ray: &mut Ray, norm: &Vector3::<f64>, pos: &Vector3::<f64>, russian_roulette: bool) -> bool {
            let mut rng = thread_rng();
            let mut flux = ray.get_flux().clone();
            if russian_roulette {
                let h = self.color.max();
                if rng.gen_range(0. .. 1.) > h {
                    return false;
                } else {
                    flux = flux.map(|x| x / h);
                }
            }
            let x_axis = gen_vert(norm);
            let y_axis: Vector3::<f64> = x_axis.cross(*norm);
            y_axis.normalize();
            let theta: f64 = 2. * f64::consts::PI * rng.gen_range(0. .. 1.);
            let phi: f64 = f64::acos(2. * rng.gen_range(0. .. 1.) - 1.);
            let direction_out = f64::cos(theta) * f64::sin(phi) * x_axis + 
                f64::sin(theta) * f64::sin(phi) * y_axis + 
                f64::cos(phi) * *norm;
            ray.set(*pos, direction_out, flux * self.color);
            true
        }
    }

    pub struct SpecularMaterial {
        color: Vector3::<f64>
    }

    impl SpecularMaterial {
        pub fn new(color: Vector3::<f64>) -> Self {
            Self {
                color
            }
        }
    }

    pub struct RefractionMaterial {
        color: Vector3::<f64>,
        refr_index: f64
    }

    impl RefractionMaterial {
        pub fn new(color: Vector3::<f64>, refr_index: Option<f64>) -> Self {
            Self {
                color,
                refr_index: refr_index.unwrap_or(1.5)
            }
        }
    }
}