use image::{Rgb, ImageResult, ImageBuffer};
use vecmat::vector::Vector3;
use vecmat::matrix::Matrix4x4;
use vecmat::Vector;
use core::f64;
use json::JsonValue;
use std::ops::{Div, Mul};
use std::{u8, usize};

use crate::photon::HitPoint;
// use std::iter::zip;
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

pub trait IsF32OrF64 : Sized {
    fn get_min(self, other: Self) -> Self;
    fn get_max(self, other: Self) -> Self;
}

impl IsF32OrF64 for f32 {
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }
}
impl IsF32OrF64 for f64 {
    fn get_min(self, other: Self) -> Self {
        self.min(other)
    }
    fn get_max(self, other: Self) -> Self {
        self.max(other)
    }
}

pub fn get_min<T, const N: usize>(a: &Vector<T, N>, b: &Vector<T, N>) -> Vector<T, N> 
    where T: Copy + Clone + IsF32OrF64 {
    Vector::<T, N>::try_from_iter(a.zip(*b).iter().map(|(x, y)| x.get_min(*y))).unwrap()
}

pub fn get_max<T, const N: usize>(a: &Vector<T, N>, b: &Vector<T, N>) -> Vector<T, N> 
    where T: Copy + Clone + IsF32OrF64 {
    Vector::<T, N>::try_from_iter(a.zip(*b).iter().map(|(x, y)| x.get_max(*y))).unwrap()
}

pub fn elementwise_division<T, const N: usize>(a: &Vector<T, N>, b: &Vector<T, N>) -> Vector<T, N>
    where T: Copy + Clone + Div<Output = T> {
    Vector::<T, N>::try_from_iter(a.zip(*b).iter().map(|(x, y)| x.div(*y))).unwrap()
}

pub fn elementwise_product<T, const N: usize>(a: &Vector<T, N>, b: &Vector<T, N>) -> Vector<T, N>
    where T: Copy + Clone + Mul<Output = T> {
    Vector::<T, N>::try_from_iter(a.zip(*b).iter().map(|(x, y)| x.mul(*y))).unwrap()
}

pub fn get_dist(l: f64, r: f64, v: f64) -> f64 {
    assert!(l <= r);
    if v < l {
        l - v
    } else if l <= v && v <= r {
        0.
    } else {
        v - r
    }
}

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
pub fn parse_vector(raw: &JsonValue) -> Vector3::<f64> {
    Vector3::<f64>::from([
        raw[0].as_f64().unwrap(),
        raw[1].as_f64().unwrap(),
        raw[2].as_f64().unwrap()
    ])
}

pub fn trunc(color: f64) -> u8 {
    (color * 256.) as u8
}

pub fn render(pic: &Vec<Vec<HitPoint>>, output_file: &str) -> ImageResult<()> {
    let width = pic.len() as u32;
    let height = pic[0].len() as u32;
    println!("width {}, height {}", width, height);
    let img = ImageBuffer::from_fn(
        width,
        height,
        |x, y| {Rgb([
            trunc(pic[x as usize][(height - 1 - y) as usize].tau.x()),
            trunc(pic[x as usize][(height - 1 - y) as usize].tau.y()),
            trunc(pic[x as usize][(height - 1 - y) as usize].tau.z())
        ])}
    );
    img.save(output_file)?;
    Ok(())
}