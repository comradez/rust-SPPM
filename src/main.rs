use core::f64;
use std::{env, usize};
mod sceneparser;
mod camera;
mod ray;
mod lights;
mod materials;
mod object3d;
mod hit;
mod matrix;
mod mesh;
mod photon;
use image::ImageError;
use materials::MaterialType;
use object3d::Group;
use object3d::Object3d;
use vecmat::vector::Vector2;
use vecmat::vector::Vector3;
use image::{Rgb, ImageResult, ImageBuffer};

use crate::{photon::{HitPoint, KDTree, Photon}, sceneparser::build_sceneparser};
use crate::ray::Ray;
use crate::matrix::trunc;
use rand::{thread_rng, Rng};

static PHOTON_NUMBER: u32 = 100000;
static ROUND_NUMBER: u32 = 3;
static SAMPLE_NUMBER: u32 = 3;
static _PARALLEL_NUMBER: u32 = 8;
static _PHOTONS_PER_ROUND: u32 = PHOTON_NUMBER / _PARALLEL_NUMBER;
static TMIN: f64 = 0.015;

fn render(pic: &Vec<Vec<HitPoint>>, output_file: &str) -> ImageResult<()> {
    let width = pic.len() as u32;
    let height = pic[0].len() as u32;
    let img = ImageBuffer::from_fn(
        width,
        height,
        |x, y| {
            let point = &pic[x as usize][(height - 1 - y) as usize];
            let area = f64::consts::PI * point.radius * point.radius;
            let number = (PHOTON_NUMBER * ROUND_NUMBER) as f64;
            Rgb([
                trunc(point.tau.x() / (area * number)),
                trunc(point.tau.y() / (area * number)),
                trunc(point.tau.z() / (area * number)),
            ])
        }
    );
    img.save(output_file)?;
    Ok(())
}

fn photon_trace(group: &Box<Group>, mut ray: Ray, photon_map: &mut Vec<Photon>) {
    let mut depth = 0;
    loop {
        if depth > 100 {
            break;
        }
        let hit = group.intersect(&ray, TMIN);
        if let Some(hit) = hit {
            let material = hit.get_material();
            let position = ray.point_at_param(hit.get_t());
            let direction = ray.get_direction().clone();
            depth += 1;
            match material.get_type() { //对于漫反射介质，存入光子图
                &MaterialType::DIFFUSE => {
                    photon_map.push(Photon::new(
                        position, 
                        direction, 
                        *hit.get_normal(),
                        *ray.get_flux()
                    ));
                },
                _ => {} //其他介质不用考虑
            }
            if material.bsdf(&mut ray, hit.get_normal(), &position, depth >= 10) == false {
                break;
            }
        } else {
            break;
        }
    }
}

fn ray_trace(
    x: usize, y: usize, group: &Box<Group>, mut ray: Ray, kd_tree: &KDTree,
    picture: &Vec<Vec<HitPoint>>, buffer: &mut Vec<Vec<HitPoint>>
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
                &MaterialType::DIFFUSE => {
                    buffer[x][y].radius = picture[x][y].radius;
                    buffer[x][y].pos = Some(position);
                    kd_tree.search(&mut buffer[x][y], color, hit.get_normal(), ray.get_flux());
                    break;
                },
                &MaterialType::SPECULAR | &MaterialType::REFRACTION => {
                    if material.bsdf(&mut ray, hit.get_normal(), &position, depth >= 20) == false {
                        break;
                    }
                }
            }
        } else { //没交上
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
    let mut picture = vec![vec![HitPoint::new(); height]; width];
    let mut buffer = vec![vec![HitPoint::new(); height]; width];
    
    let mut rng = thread_rng();
    for round in 0 .. ROUND_NUMBER {
        let mut photon_map: Vec<Photon> = Vec::new();
        for light in &lights {
            for _ in 0 .. PHOTON_NUMBER {
                let ray = light.get_ray();
                photon_trace(&group, ray, &mut photon_map);
            }
        }
        println!("Round {} photon pass complete", &round);
        let kd_tree = KDTree::new(photon_map);
        println!("Round {} kdtree build complete", &round);
        for x in 0 .. width {
            for y in 0 .. height {
                buffer[x][y].tau = Vector3::<f64>::from([0., 0., 0.]);
                buffer[x][y].n = 0.;
                for _ in 0 .. SAMPLE_NUMBER {
                    let mut ray = camera.generate_ray(&Vector2::<f64>::from([
                        x as f64 + rng.gen_range(0. .. 1.),
                        y as f64 + rng.gen_range(0. .. 1.)
                    ]));
                    ray.set_color(*ray.get_flux() / (SAMPLE_NUMBER as f64));
                    ray_trace(x, y, &group, ray, &kd_tree, &picture, &mut buffer);
                }
                if round == 0 {
                    picture[x][y].n = buffer[x][y].n;
                    picture[x][y].tau = buffer[x][y].tau;
                } else {
                    if picture[x][y].n + buffer[x][y].n > 0. {
                        let ratio = (picture[x][y].n + photon::ALPHA * buffer[x][y].n) / (picture[x][y].n + buffer[x][y].n);
                        picture[x][y].radius *= f64::sqrt(ratio);
                        picture[x][y].tau = (picture[x][y].tau + buffer[x][y].tau) * ratio;
                        picture[x][y].n += buffer[x][y].n * ratio;
                    }
                }
            }
        }
        println!("Round {} complete", &round);
    }
    render(&picture, &output_file)?;
    Ok(())
}
