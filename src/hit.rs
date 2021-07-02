use crate::materials::Material;
use vecmat::vector::Vector3;
use std::rc::Rc;

pub struct Hit {
    t: f64,
    material: Rc<dyn Material>,
    normal: Vector3::<f64>
}

impl Hit {
    pub fn new(t: f64, material: Rc<dyn Material>, normal: Vector3::<f64>) -> Self { 
        Self { t, material, normal } 
    }

    pub fn clone_obj(&self) -> Self {
        Self { t: self.t, material: Rc::clone(&self.material), normal: self.normal }
    }

    pub fn get_t(&self) -> f64 {
        self.t
    }

    pub fn get_material(&self) -> &Rc<dyn Material> {
        &self.material
    }

    pub fn get_normal(&self) -> &Vector3::<f64> {
        &self.normal
    }
}