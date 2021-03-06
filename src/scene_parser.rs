use crate::{
    camera::{build_camera, Camera},
    lights::{build_light, Light},
    materials::{build_material, Material},
    object3d::{build_group, Group},
};
use std::sync::Arc;

pub struct SceneParser {
    pub camera: Arc<dyn Camera + Send + Sync>,
    pub lights: Vec<Arc<dyn Light + Send + Sync>>,
    pub materials: Vec<Arc<dyn Material + Send + Sync>>,
    pub group: Arc<Group>,
}
pub fn build_scene_parser(scene_name: String) -> SceneParser {
    let json_raw = std::fs::read_to_string(scene_name).expect("File not exist!");
    let json_parsed = json::parse(&json_raw).expect("Json invalid!");
    let camera = &json_parsed["Camera"];
    let lights = &json_parsed["Lights"];
    let materials = &json_parsed["Materials"];
    let group = &json_parsed["Group"];
    assert!(camera.is_object());
    assert!(lights.is_array());
    assert!(materials.is_array());
    assert!(group.is_array());
    let camera = build_camera(camera);
    let lights: Vec<Arc<dyn Light + Send + Sync>> =
        lights.members().map(|x| build_light(x)).collect();
    let materials: Vec<Arc<dyn Material + Send + Sync>> =
        materials.members().map(|x| build_material(x)).collect();
    let group: Arc<Group> = build_group(group, &materials);
    SceneParser {
        camera,
        lights,
        materials,
        group,
    }
}
