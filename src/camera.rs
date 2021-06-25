pub mod camera {
    use core::f64;

    use json::JsonValue;
    use vecmat::{Matrix, Vector, traits::{Dot}, vector::Vector3};

    //  use crate::ray;
    use crate::ray::ray::Ray;

    pub struct PerspectiveCamera {
        center: Vector3::<f64>,
        direction: Vector3::<f64>,
        horizental: Vector3::<f64>,
        up: Vector3::<f64>,
        angle: f64,
        width: u32,
        height: u32,
        dist: f64,
    }

    impl PerspectiveCamera {
        pub fn new(center: Vector3::<f64>, direction: Vector3::<f64>, up: Vector3::<f64>, angle: f64, width: u32, height: u32) -> Self {
            direction.normalize();
            let horizental: Vector3::<f64> = direction.cross(up);
            horizental.normalize();
            let up: Vector3::<f64> = horizental.cross(direction);
            Self { 
                center, direction, horizental, up, 
                angle: angle / std::f64::consts::PI * 180.0,
                width, height,
                dist: height as f64 / (2.0 * f64::tan(angle / 2.0)),
            }
        }
    }

    pub struct DoFCamera {
        center: Vector3::<f64>,
        direction: Vector3::<f64>,
        horizental: Vector3::<f64>,
        up: Vector3::<f64>,
        angle: f64,
        width: u32,
        height: u32,
        focus: Vector3::<f64>,
        aperture: f64,
        dist: f64,
        focus_dist: f64
    }

    impl DoFCamera {
        pub fn new(center: Vector3::<f64>, direction: Vector3::<f64>, up: Vector3::<f64>, angle: f64, width: u32, height: u32, focus: Vector3::<f64>, aperture: f64) -> Self {
            direction.normalize();
            let horizental: Vector3::<f64> = direction.cross(up);
            horizental.normalize();
            let up: Vector3::<f64> = horizental.cross(direction);
            Self {
                center, direction, horizental, up, 
                angle: angle / std::f64::consts::PI * 180.0,
                width, height,
                focus, aperture,
                dist: height as f64 / (2.0 * f64::tan(angle / 2.0)),
                focus_dist: focus.dot(direction)
            }
        }
    }

    

    pub trait Camera {
        fn generate_ray(&self, point: &Vector::<f64, 2>) -> Ray;
    }

    impl Camera for PerspectiveCamera {
        fn generate_ray(&self, point: &Vector::<f64, 2>) -> Ray {
            let dir = Vector3::<f64>::from([
                point[0] - self.width as f64 / 2 as f64,
                point[1] - self.height as f64 / 2 as f64,
                self.dist
            ]);
            let rot = Matrix::<f64, 3, 3>::from_array_of_vectors([
                self.horizental, self.up, self.direction
            ]).transpose();
            let dir = rot.dot(dir);
            dir.normalize();
            Ray::new(self.center, dir, None)
        }
    }

    impl Camera for DoFCamera {
        fn generate_ray(&self, point: &Vector::<f64, 2>) -> Ray {
            Ray::new(self.center, self.direction, None) // 占位用的
        }
    }

    pub fn parse_vector(mut raw: JsonValue) -> Vector3::<f64> {
        Vector3::<f64>::from([
            raw.array_remove(0).as_f64().unwrap(),
            raw.array_remove(1).as_f64().unwrap(),
            raw.array_remove(2).as_f64().unwrap()
        ])
    }
    pub fn build_camera(mut camera_attr: JsonValue) -> Box<dyn Camera> {
        let cam_type = camera_attr.remove("Type").take_string().unwrap();
        let center = parse_vector(camera_attr.remove("Center"));
        let direction = parse_vector(camera_attr.remove("Direction"));
        let up = parse_vector(camera_attr.remove("Up"));
        let angle = camera_attr.remove("Angle").as_f64().unwrap();
        let width = camera_attr.remove("Width").as_u32().unwrap();
        let height = camera_attr.remove("Height").as_u32().unwrap();
        if cam_type == "Perspective" {
            Box::new(PerspectiveCamera::new(center, direction, up, angle, width, height))
        } else if cam_type == "DoF" {
            let focus = parse_vector(camera_attr.remove("Focus"));
            let aperture = camera_attr.remove("Aperture").as_f64().unwrap();
            Box::new(DoFCamera::new(center, direction, up, angle, width, height, focus, aperture))
        } else {
            panic!("Invalid Camera Type!");
        }
    }
}