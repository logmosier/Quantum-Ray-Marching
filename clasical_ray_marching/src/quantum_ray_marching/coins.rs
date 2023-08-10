use super::{ray_packet::RayPacket, dirction_map::DirMap};


#[derive(Clone, Copy, Debug)]
pub enum QuantumCoin{
    Air, 
    Surface{emited: glm::Vec3, diffuse: glm::Vec3, normal: glm::Vec3}, 
}
impl  QuantumCoin{
    pub fn flip(&self, ray_packet: &RayPacket, d_map: &DirMap) -> Vec<usize>{
        match self {
            QuantumCoin::Air => vec![ray_packet.direction],
            QuantumCoin::Surface{normal,..} =>  d_map.scatter(normal),
        }
    }

    pub fn sample_difuse(&self) -> glm::Vec3{
        match self {
            QuantumCoin::Air => glm::Vec3::from_element(1.0),
            QuantumCoin::Surface{diffuse, ..} => *diffuse,
        }
    }

    pub fn sample_emited(&self) -> glm::Vec3{
        match self {
            QuantumCoin::Air =>  glm::Vec3::zeros(),
            QuantumCoin::Surface{emited, ..} => *emited,
        }
    }
}