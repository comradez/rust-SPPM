pub mod utils {
    use vecmat::{Vector, vector::Vector3};
    use json::JsonValue;
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
    pub fn parse_vector(mut raw: JsonValue) -> Vector3::<f64> {
        Vector3::<f64>::from([
            raw.array_remove(0).as_f64().unwrap(),
            raw.array_remove(1).as_f64().unwrap(),
            raw.array_remove(2).as_f64().unwrap()
        ])
    }
}