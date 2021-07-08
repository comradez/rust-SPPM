#![allow(clippy::too_many_arguments)]
mod camera;
mod hit;
mod lights;
mod materials;
mod mesh;
mod object3d;
mod photon;
mod ray;
mod sceneparser;
mod utils;
use core::f64;
use std::{
    env,
    sync::{Arc, Barrier, Mutex},
    thread, u32, usize,
};
use crate::{
    materials::MaterialType,
    object3d::{Group, Object3d},
    photon::{HitPoint, KDTree, Photon},
    ray::Ray,
    sceneparser::build_sceneparser,
    utils::trunc,
};
use image::{ImageBuffer, ImageError, ImageResult, Rgb};
use vecmat::vector::Vector2;

const PHOTON_NUMBER: u32 = 1000000;
const ROUND_NUMBER: u32 = 5;
const SAMPLE_NUMBER: u32 = 8;
const PARALLEL_NUMBER: usize = 8;
const _PHOTONS_PER_ROUND: u32 = PHOTON_NUMBER / PARALLEL_NUMBER as u32;
const TMIN: f64 = 0.015;
const NUMBER: f64 = (PHOTON_NUMBER * ROUND_NUMBER) as f64;

fn render(
    pic: &[Arc<Mutex<Vec<Vec<HitPoint>>>>],
    output_file: &str,
    width: u32,
    height: u32,
) -> ImageResult<()> {
    ImageBuffer::from_fn(width, height, |x, y| {
        let interval = width as usize / PARALLEL_NUMBER;
        let dim_1 = x as usize / interval;
        let dim_2 = x as usize % interval;
        let point = &pic[dim_1].lock().unwrap()[dim_2][(height - 1 - y) as usize];
        let area = f64::consts::PI * point.radius * point.radius;
        Rgb([
            trunc(point.tau.x() / (area * NUMBER)),
            trunc(point.tau.y() / (area * NUMBER)),
            trunc(point.tau.z() / (area * NUMBER)),
        ])
    })
    .save(output_file)?;
    Ok(())
}

fn photon_trace(group: &Arc<Group>, mut ray: Ray, photon_map: &mut Vec<Photon>) {
    let mut depth = 0;
    loop {
        if depth > 100 {
            break;
        }
        let hit = group.intersect(&ray, TMIN);
        if let Some(hit) = hit {
            let material = hit.get_material();
            let position = ray.point_at_param(hit.get_t());
            let direction = ray.get_direction();
            depth += 1;
            if let MaterialType::DIFFUSE = material.get_type() {
                photon_map.push(Photon::new(
                    position,
                    *direction,
                    *hit.get_normal(),
                    *ray.get_flux(),
                ));
            }
            if !material.bsdf(&mut ray, hit.get_normal(), &position, depth >= 10) {
                break;
            }
        } else {
            break;
        }
    }
}

fn ray_trace(
    group: &Arc<Group>,
    mut ray: Ray,
    kd_tree: &Arc<KDTree>,
    radius: f64,
    buffer_pixel: &mut HitPoint
) {
    let mut depth = 0;
    loop {
        if depth > 100 {
            break;
        }
        let hit = group.intersect(&ray, TMIN);
        if let Some(hit) = hit {
            let material = hit.get_material();
            let color = material.get_color();
            let position = ray.point_at_param(hit.get_t());
            depth += 1;
            match material.get_type() {
                MaterialType::DIFFUSE => {
                    buffer_pixel.radius = radius;
                    buffer_pixel.pos = Some(position);
                    kd_tree.search(buffer_pixel, color, hit.get_normal(), ray.get_flux());
                    break;
                }
                MaterialType::SPECULAR | MaterialType::REFRACTION => {
                    if !material.bsdf(&mut ray, hit.get_normal(), &position, depth >= 20) {
                        break;
                    }
                }
            }
        } else {
            //没交上
            break;
        }
    }
}

fn main() -> Result<(), ImageError> {
    let mut args = env::args().skip(1);
    let scene_file = args.next().expect("No scene file specified.");
    let output_file = args.next().expect("No output file specified.");
    let parser = build_sceneparser(scene_file);
    let camera = parser.camera;
    let lights = parser.lights;
    let group = parser.group;
    let width = camera.get_width() as usize;
    let height = camera.get_height() as usize;
    let mut pictures: Vec<Arc<Mutex<Vec<Vec<HitPoint>>>>> = Vec::with_capacity(PARALLEL_NUMBER);
    for _ in 0..PARALLEL_NUMBER {
        pictures.push(Arc::new(Mutex::new(vec![
            vec![HitPoint::new(); height];
            width / PARALLEL_NUMBER
        ])));
    }
    let barrier = Arc::new(Barrier::new(PARALLEL_NUMBER + 1));

    for round in 0..ROUND_NUMBER {
        let mut photon_map: Vec<Photon> = Vec::new();
        for light in &lights {
            for _ in 0..PHOTON_NUMBER {
                let ray = light.get_ray();
                photon_trace(&group, ray, &mut photon_map);
            }
        }
        println!("Round {} photon pass complete", &round);
        let kd_tree = KDTree::new(photon_map);
        let arc_kd_tree = Arc::new(kd_tree);
        println!("Round {} kdtree build complete", &round);
        for (i, picture) in pictures.iter().enumerate() {
            let group = group.clone();
            let camera = camera.clone();
            let arc_kd_tree = arc_kd_tree.clone();
            let picture = Arc::clone(picture);
            let barrier = barrier.clone();
            thread::spawn(move || {
                let column_begin = width * i / PARALLEL_NUMBER;
                let column_end = width * (i + 1) / PARALLEL_NUMBER;
                // println!(
                //     "thread {} spawns with column range [{}, {})",
                //     &i, &column_begin, &column_end
                // );
                let mut buffer = vec![vec![HitPoint::new(); height]; column_end - column_begin];
                let mut picture = picture.lock().unwrap();
                for (x, global_x) in (column_begin..column_end).enumerate() {
                    for y in 0..height {
                        let buffer_pixel = &mut buffer[x][y];
                        let picture_pixel = &mut picture[x][y];
                        for _ in 0..SAMPLE_NUMBER {
                            let dest_x = global_x as f64 + rand::random::<f64>();
                            let dest_y = y as f64 + rand::random::<f64>();
                            let mut ray =
                                camera.generate_ray(&Vector2::<f64>::from([dest_x, dest_y]));
                            ray.set_color(*ray.get_flux() / (SAMPLE_NUMBER as f64));
                            ray_trace(
                                &group,
                                ray,
                                &arc_kd_tree,
                                picture_pixel.radius,
                                buffer_pixel
                            );
                        }
                        if round == 0 {
                            picture_pixel.n = buffer_pixel.n;
                            picture_pixel.tau = buffer_pixel.tau;
                        } else if picture_pixel.n + buffer_pixel.n > 0. {
                            let ratio = (picture_pixel.n + photon::ALPHA * buffer_pixel.n)
                                / (picture_pixel.n + buffer_pixel.n);
                            picture_pixel.radius *= f64::sqrt(ratio);
                            picture_pixel.tau = (picture_pixel.tau + buffer_pixel.tau) * ratio;
                            picture_pixel.n += buffer_pixel.n * ratio;
                        }
                    }
                }
                drop(picture);
                // println!(
                //     "thread {} ends with column range [{}, {})",
                //     &i, &column_begin, &column_end
                // );
                barrier.wait();
            });
        }
        barrier.wait();
        println!("Round {} complete", &round);
    }
    render(&pictures, &output_file, width as u32, height as u32)?;
    Ok(())
}
