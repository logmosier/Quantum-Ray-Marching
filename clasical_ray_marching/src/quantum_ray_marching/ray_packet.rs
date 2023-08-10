use fixed::{types::extra::U3, FixedU16};
use glm::Vec3;
use itertools::Itertools;
use super::{dirction_map::DirMap, path::QuantumSample, coins::QuantumCoin, voxels::Voxels};

type NumFracBits = U3;
pub type RayPacketValue = FixedU16<NumFracBits>;

#[derive(Debug, Clone)]
pub struct RayPacket {
    pub position: glm::TVec3<RayPacketValue>,
    pub direction: usize,
}
impl RayPacket{
    pub fn new(index: glm::Vec3, direction: usize) -> Self {
        RayPacket { position: index.map(|v| RayPacketValue::from_num(v)), direction}
    }
    pub fn new_converted(index: glm::TVec3<RayPacketValue>, direction: usize) -> Self {
        RayPacket { position: index, direction}
    }

    pub fn evaluate(&self, voxels: &Voxels, d_map: &DirMap, cheat: bool, depth: usize, mut current_sample: QuantumSample) -> Vec<QuantumSample>{
        let mut packet = (*self).clone();

        while cheat && matches!(voxels.get_coin(packet.position), Some(QuantumCoin::Air)){

            let temp_pos = d_map.step(packet.position, packet.direction);
            if !matches!(voxels.get_coin(temp_pos), Some(QuantumCoin::Air)){
                break;
            }
            packet.position = d_map.step(packet.position, packet.direction); 
            //println!("cheating {:?} {:?}, {:?}", cheat, packet, voxels.get_coin(packet.position));
        }

        let reflection = self.get_sample_diffuse(voxels);
        let emited = self.get_sample_emited(voxels);

        current_sample.update(reflection, emited);

        // println!("depth: {}, emited: {:?}, reflection: {:?}, r: {:?}, sample: {:?}", depth, emited, reflection, current_sample.r, current_sample.value);
        if depth == 0 {
            vec!(current_sample)
        }
        else{
            let directions =  voxels
            .get_coin(packet.position)
            .and_then(|c| Some(c.flip(&packet, d_map)))
            .unwrap_or(vec![]);
            
            if directions.is_empty(){
                vec!(current_sample)
            }
            else{
                directions.into_iter()
                .flat_map(|d| {
                    let packet = RayPacket::new_converted(d_map.step(packet.position, d), d);
                    packet.evaluate(voxels, d_map, cheat, depth - 1, current_sample.clone())
                }).collect_vec()
            }
        }
    }

    fn get_sample_diffuse(&self, voxels: &Voxels) -> Vec3 {
        voxels.get_sample_diffuse(self.position)
    }
    fn get_sample_emited(&self, voxels: &Voxels) -> Vec3 {
        voxels.get_sample_emited(self.position)
    }
}