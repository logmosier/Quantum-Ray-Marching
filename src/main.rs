pub mod quantum_ray_marching;
extern crate nalgebra_glm as glm;
extern crate nalgebra as na;

use quantum_ray_marching::camera::Camera;
use glm::Vec3;
fn main() {
    let q_cam = Camera::new(
        Vec3::new(0.0, 2.0, 10.0),
        Vec3::new(0.0, 0.0, 0.0),
        (70.0f32).to_radians(),
        glm::vec2(100, 100),
    );
    println!("Hello, world!");
}
