use std::rc::Rc;
use crate::camera::{build_camera, Camera};
use crate::lights::{build_light, Light};
use crate::materials::{build_material, Material};
use crate::object3d::{build_group, Group};

pub struct SceneParser {
    pub camera: Box<dyn Camera>,
    pub lights: Vec<Box<dyn Light>>,
    pub materials: Vec<Rc<dyn Material>>,
    pub group: Box<Group>,
}
pub fn build_sceneparser(scene_name: String) -> SceneParser {
    let json_raw = std::fs::read_to_string(scene_name)
        .expect("File not exist!");
    let json_parsed = json::parse(&json_raw)
        .expect("Json invalid!");
    let camera = &json_parsed["Camera"];
    let lights = &json_parsed["Lights"];
    let materials = &json_parsed["Materials"];
    let group = &json_parsed["Group"];
    assert!(camera.is_object());
    assert!(lights.is_array());
    assert!(materials.is_array());
    assert!(group.is_array());
    let camera = build_camera(&camera);
    let lights: Vec<Box<dyn Light>> = lights
        .members()
        .map(|x| build_light(x))
        .collect();
    let materials: Vec<Rc<dyn Material>> = materials
        .members()
        .map(|x| build_material(x))
        .collect();
    let group: Box<Group> = build_group(&group, &materials);
    SceneParser {
        camera, lights, materials, group
    }
}