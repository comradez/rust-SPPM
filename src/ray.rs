pub mod ray {
    use vecmat::{vector::Vector3};
    pub struct Ray {
        origin: Vector3::<f64>,
        direction: Vector3::<f64>,
        color: Vector3::<f64>
    }

    impl Ray {
        pub fn new(origin: Vector3::<f64>, direction: Vector3::<f64>, color: Option<Vector3::<f64>>) -> Self {
            direction.normalize();
            Self { 
                origin,
                direction,
                color: match color {
                    Some(color) => color,
                    None => Vector3::<f64>::from([1., 1., 1.])
                }
            }
        }
        pub fn get_origin(&self) -> &Vector3::<f64> {
            &self.origin
        }
        pub fn get_direction(&self) -> &Vector3::<f64> {
            &self.direction
        }
        pub fn get_flux(&self) -> &Vector3::<f64> {
            &self.color
        }
        pub fn set(&mut self, origin: Vector3::<f64>, direction: Vector3::<f64>, color: Vector3::<f64>) {
            self.origin = origin;
            self.direction = direction;
            self.color = color;
        }
        pub fn point_at_param(&self, t: f64) -> Vector3::<f64> {
            self.origin + self.direction * t
        }
    }
}