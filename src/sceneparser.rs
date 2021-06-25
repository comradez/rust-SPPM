use json;
use crate::camera::camera::build_camera;
mod sceneparser {
    use crate::camera::camera::{build_camera, Camera};

    struct SceneParser {
        camera: Box<dyn Camera>
    }
    fn build_sceneparser(scene_name: String) -> SceneParser {
        let json_raw = std::fs::read_to_string(scene_name)
            .expect("File not exist!");
        let mut json_parsed = json::parse(&json_raw)
            .expect("Json invalid!");
        let camera = json_parsed.remove("Camera");
        let lights = json_parsed.remove("Lights");
        let materials = json_parsed.remove("Materials");
        let group = json_parsed.remove("Group");
        assert!(camera.is_object());
        assert!(lights.is_object());
        assert!(materials.is_object());
        assert!(group.is_object());
        let camera = build_camera(camera);
        SceneParser {
            camera
        }
    }
}