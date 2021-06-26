use std::env;
mod sceneparser;
mod camera;
mod ray;
mod lights;
mod utils;
mod materials;
fn main() {
    let mut args = env::args().skip(1);
    let scene_file = args.next().expect("No scene file specified.");
    let output_file = args.next().expect("No output file specified.");
    println!("{}\n{}", scene_file, output_file);
}
