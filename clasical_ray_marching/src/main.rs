pub mod quantum_ray_marching;
extern crate nalgebra_glm as glm;
extern crate nalgebra as na;

use std::sync::Arc;

use itertools::Itertools;
use quantum_ray_marching::camera::Camera;
use glm::Vec3;

use crate::quantum_ray_marching::{object::Object, mesh::Mesh, voxels:: Voxels, quantum_ray_marching::QuantumRayMarcherer};
fn main() {
    let camera = Camera::new(
        &Vec3::new(0.0, 2.0, 1.0),

        &Vec3::new(0.0, 2.0, 0.0), 
        (70.0f32).to_radians(),
        glm::vec2(5, 5),
    );
    let mut ray_marcher = QuantumRayMarcherer::new(camera);
    let cornell_meshs = Mesh::from_obj(&"obj/cornell-box-back.obj".to_string());
    let objects = cornell_meshs.iter().map(|mesh|{
        Arc::new(Object{
            mesh: mesh.clone(),
            transform: glm::Mat4::identity()
        })
    }).collect_vec();
    let voxels=  Voxels::new(&objects);
    ray_marcher.render(&voxels);
    println!("Hello, world!");
}
