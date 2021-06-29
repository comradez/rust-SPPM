use vecmat::vector::Vector3;
use vecmat::matrix::Matrix4x4;
use core::f64;
pub fn gen_translation(translation: &Vector3<f64>) -> Matrix4x4<f64> {
    Matrix4x4::from_array_of_arrays([
        [1., 0., 0., translation[0]],
        [0., 1., 0., translation[1]],
        [0., 0., 1., translation[2]],
        [0., 0., 0., 1.]
    ])
}

pub fn to_radian(degree: f64) -> f64 {
    degree * f64::consts::PI / 180.
}

pub fn gen_rotate(degree: f64, dim: usize) -> Matrix4x4<f64> {
    let radian = to_radian(degree);
    let c = f64::cos(radian);
    let s = f64::sin(radian);
    match dim {
        0 => Matrix4x4::from_array_of_arrays([
            [1., 0., 0., 0.],
            [0., c , -s, 0.],
            [0., s ,  c, 0.],
            [0., 0., 0., 1.]
        ]),
        1 => Matrix4x4::from_array_of_arrays([
            [c , 0., s , 0.],
            [0., 1., 0., 0.],
            [-s, 0., c , 0.],
            [0., 0., 0., 1.]
        ]),
        2 => Matrix4x4::from_array_of_arrays([
            [c , -s, 0., 0.],
            [s ,  c, 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.]
        ]),
        _ => panic!("Wrong dimention")
    }    
}