use crate::camera::{build_camera, Camera};
use crate::lights::{build_light, Light};
use crate::materials::{build_material, Material};

pub struct SceneParser {
    camera: Box<dyn Camera>,
    lights: Vec<Box<dyn Light>>,
    materials: Vec<Box<dyn Material>>
}
pub fn build_sceneparser(scene_name: String) -> SceneParser {
    let json_raw = std::fs::read_to_string(scene_name)
        .expect("File not exist!");
    let mut json_parsed = json::parse(&json_raw)
        .expect("Json invalid!");
    let mut camera = json_parsed.remove("Camera");
    let mut lights = json_parsed.remove("Lights");
    let mut materials = json_parsed.remove("Materials");
    let group = json_parsed.remove("Group");
    assert!(camera.is_object());
    assert!(lights.is_array());
    assert!(materials.is_array());
    assert!(group.is_array());
    let camera = build_camera(&mut camera);
    let lights: Vec<Box<dyn Light>> = lights
        .members_mut()
        .map(|x| build_light(x))
        .collect();
    let materials: Vec<Box<dyn Material>> = materials
        .members_mut()
        .map(|x| build_material(x))
        .collect();
    SceneParser {
        camera,
        lights,
        materials
    }
}