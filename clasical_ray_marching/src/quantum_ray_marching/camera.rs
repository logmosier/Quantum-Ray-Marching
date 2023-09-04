
use rayon::prelude::{ParallelIterator};
use super::{dirction_map::DirMap, ray_packet::RayPacket, voxels::Voxels};

pub struct Camera {
    up: glm::Vec3,
    left: glm::Vec3,
    front: glm::Vec3,
    position: glm::Vec3,
    yaw: f32,
    pitch: f32,
    pub fov: f32, // should be in rad 
    pub view: glm::Mat4,
    pub pixels: glm::UVec2, 
    pub aspect: f32,
}
impl Camera{
    pub fn new(position: &glm::Vec3, look_at: &glm::Vec3, fov: f32, pixels: glm::UVec2) -> Self{

        let front = (look_at - position).normalize();
        let pitch = front.y.asin();
        let yaw = -(front.x /pitch.cos()).acos();
        let left = front.cross(&glm::Vec3::y()).normalize();
        let up = front.cross(&left).normalize();
        let view = glm::look_at_rh(position, &(position+front), &up);

        Camera{
            up,
            left,
            front,
            position: *position,
            yaw: yaw,
            pitch: pitch,
            fov, 
            view,
            pixels,
            aspect: pixels.x as f32 / pixels.y as f32,
        }
    }

    pub fn get_start_points(&self, voxels: &Voxels, dir_map: &DirMap) -> Vec<(glm::UVec2, RayPacket)>{
        (0..self.pixels.x).into_iter().flat_map(|px| 
            (0..self.pixels.y).into_iter().map(|py| {
                let dx  = (2.0 * (px as f32 + 0.5) / self.pixels.x as f32 - 1.0) * (self.fov / 2.0).tan() * self.aspect;
                let dy = (1.0 - 2.0 * (py as f32 + 0.5) / self.pixels.y as f32) * (self.fov / 2.0).tan();
                let ray_dir = (self.view * glm::vec4(dx,dy,-1.0, 0.0)).xyz();
                if let Some((t, (x,y,z))) = voxels.intersect_ray(self.position, glm::vec3(1.0,1.0,1.0).component_div(&ray_dir)) {
                    let p = glm::vec3(x as f32,y as f32,z as f32);//voxels.convert_to_voxel_space(self.position + t * ray_dir);
                    // glm::vec3(x as f32,y as f32,z as f32);
                    // println!("{} {} {} p: {:?}", x,y,z, p);
                    (glm::UVec2::new(px ,self.pixels.y - 1 - py), RayPacket::new(p, dir_map.map(ray_dir.xyz())))
                }
                else {
                    (glm::UVec2::new(px ,self.pixels.y - 1 - py), RayPacket::new(glm::vec3(0.0,0.0,0.0), dir_map.map(ray_dir.xyz())))
                }
            }).collect::<Vec<(glm::UVec2, RayPacket)>>()
        ).collect()
}
}