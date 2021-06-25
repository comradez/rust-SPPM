pub mod utils {
    use vecmat::{Vector, vector::Vector3};
    pub fn gen_vert(vec: &Vector3::<f64>) -> Vector3::<f64> {
        let temp: Vector3::<f64>;
        if vec.x() > 0.2 {
            temp = Vector3::<f64>::from([0., 1., 0.]);
        } else {
            temp = Vector3::<f64>::from([1., 0., 0.]);
        }
        let vec: Vector3::<f64> = vec.cross(temp);
        vec.normalize();
        vec
    }
}