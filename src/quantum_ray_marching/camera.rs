use std::sync::Arc;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use super::{dirction_map::DirMap, ray_packet::RayPacket, voxels::Voxels};


pub struct Camera {
    pub position: glm::Vec3,
    pub rotation: glm::Vec3,
    pub fov: f32, // should be in rad 
    pub pixels: glm::UVec2, 
    pub aspect: f32,
    matrix: glm::Mat4,
}
impl Camera{
    pub fn new(position: glm::Vec3, rotation: glm::Vec3, fov: f32, pixels: glm::UVec2) -> Self{
        Camera{
            position, 
            rotation, 
            fov, 
            pixels,
            aspect: pixels.x as f32 / pixels.y as f32,
            matrix: glm::rotate_x(&glm::rotate_y(&glm::rotate_z( &glm::translation(&position), rotation.z), rotation.y), rotation.x)
        }
    }

    pub fn get_start_points(&self, voxels: &Voxels, dir_map: &DirMap) -> Vec<(glm::UVec2, RayPacket)>{
        // println!("max {:?}", voxels.convert_to_voxel_space(voxels.bounding_box.max));
        // println!("min {:?}", voxels.convert_to_voxel_space(voxels.bounding_box.min));
        // println!("center {:?}", voxels.convert_to_voxel_space(voxels.bounding_box.center));
        (0..self.pixels.x).into_iter().flat_map(|px| 
            (0..self.pixels.y).into_iter().filter_map(|py| {
                let dx  = (2.0 * (px as f32 + 0.5) / self.pixels.x as f32 - 1.0) * (self.fov / 2.0).tan() * self.aspect;
                let dy = (1.0 - 2.0 * (py as f32 + 0.5) / self.pixels.y as f32) * (self.fov / 2.0).tan();
                let ray_dir = (self.matrix * glm::vec4(dx,dy,-1.0, 0.0)).xyz();
                if let Some((t, (x,y,z))) = voxels.intersect_ray(self.position, glm::vec3(1.0,1.0,1.0).component_div(&ray_dir)) {
                    let p = voxels.convert_to_voxel_space(self.position + t * ray_dir);
                    Some((glm::UVec2::new(px ,py), RayPacket::new(p, dir_map.map(ray_dir.xyz()))))
                }
                else {
                    None
                }
            }).collect::<Vec<(glm::UVec2, RayPacket)>>()
        ).collect()
    }
}