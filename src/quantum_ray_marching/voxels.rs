
use std::{cell::RefCell, sync::Arc, rc::Rc, ops::{Div, Range, RangeInclusive}};

use az::Cast;
use fixed::{FixedU16, types::extra::U3, types::extra::U0, traits::Fixed};
use glm::{Vec3, IVec3, UVec3, Vec2};
use image::{ImageBuffer, Rgba};
use na::Vector3;
use ndarray::{Array1, Array3, Axis, s};
// use az::Cast;


use itertools::{Itertools, min};
use rayon::prelude::*;

use super::{coins::QuantumCoin, ray_packet::RayPacketValue, object::Object};

#[derive(Clone, Copy, Debug)]
pub struct Voxel {
    pub coin: QuantumCoin,
    pub min: glm::Vec3,
    pub max: glm::Vec3,
    pub center: glm::Vec3,
    pub size: glm::Vec3,
}

impl Voxel {
    pub fn new(min: glm::Vec3, max: glm::Vec3) -> Self{
        let center = (min + max) / 2.0;
        let size = max - min;
        Self{coin:QuantumCoin::Air, min, max, center, size}
    }
    pub fn new_center(center: glm::Vec3, size: glm::Vec3) -> Self{
        let min = center - size / 2.0;
        let max = center + size / 2.0;
        Self{coin:QuantumCoin::Air, min, max, center, size}
    }
    pub fn new_min_size(min: glm::Vec3, size: glm::Vec3) -> Self{
        let max = min + size;
        let center = (min + max) / 2.0;
        Self{coin:QuantumCoin::Air,min, max, center, size}
    }

    pub fn intersect_ray(&self, ray_org: glm::Vec3, ray_inv_dir: glm::Vec3) -> Option<f32>{
        let tx1 = (self.min.x - ray_org.x) * ray_inv_dir.x;
        let tx2 = (self.max.x - ray_org.x) * ray_inv_dir.x;

        let tmin = tx1.min(tx2);
        let tmax = tx1.max(tx2);

        let ty1 = (self.min.y - ray_org.y) * ray_inv_dir.y;
        let ty2 = (self.max.y - ray_org.y) * ray_inv_dir.y;

        let tmin = tmin.max(ty1.min(ty2));
        let tmax = tmax.min(ty1.max(ty2));

        let tz1 = (self.min.z - ray_org.z) * ray_inv_dir.z;
        let tz2 = (self.max.z - ray_org.z) * ray_inv_dir.z;

        let tmin = tmin.max(tz1.min(tz2));
        let tmax = tmax.min(tz1.max(tz2));

        if tmax >= tmin.max(0.0){
            Some(tmin)
        }
        else{
            None
        }
    }

    pub fn get_sample_diffuse(&self) -> glm::Vec3{
        self.coin.sample_difuse()
    }
    pub fn get_sample_emited(&self) -> glm::Vec3{
        self.coin.sample_emited()
    }
    pub fn get_vertices(&self) -> [glm::Vec3; 8]{
        [
            self.min,
            glm::vec3(self.min.x, self.min.y, self.max.z),
            glm::vec3(self.min.x, self.max.y, self.min.z),
            glm::vec3(self.min.x, self.max.y, self.max.z),
            glm::vec3(self.max.x, self.min.y, self.min.z),
            glm::vec3(self.max.x, self.min.y, self.max.z),
            glm::vec3(self.max.x, self.max.y, self.min.z),
            self.max,
        ]
    }

    pub fn add_surface(mut self, diffuse: glm::Vec3, emisive: glm::Vec3, normal: glm::Vec3) -> Self{
        self.coin = QuantumCoin::Surface{diffuse, emited: emisive, normal: normal};
        self
    }

    pub fn indeices() -> [usize; 36]{
        [
            0, 1, 2,
            1, 3, 2,
            4, 5, 6,
            5, 7, 6,
            0, 2, 4,
            2, 6, 4,
            1, 5, 3,
            5, 7, 3,
            0, 4, 1,
            4, 5, 1,
            2, 3, 6,
            3, 7, 6,
        ]
    }

    pub fn get_interval(&self, axis: &glm::Vec3) -> Range<f32>{
        let (min, max) = self.get_vertices().iter().fold(
            (f32::INFINITY, f32::NEG_INFINITY), |(min, max), v| 
            (min.min(axis.dot(v)), max.max(axis.dot(v)))
        );

        min .. max
    }

    pub fn merge(&self, other: &Voxel) -> Voxel{
        Voxel::new(self.min.inf(&other.min), self.max.sup(&other.max))
    }
}

pub struct Voxels{
    pub bounding_box: Voxel,
    pub dim: glm::UVec3,
    pub voxels: Option<Array3<Voxel>>,
    voxel_size: glm::Vec3,
    voxel_mask: [bool; 3],
    voxels_dirty: bool,
    slice_range: (glm::UVec3, glm::UVec3)
}

impl Voxels{
    pub fn new(objects: &Vec<Arc<Object>>)-> Self{
        let bounding_box = objects.iter().fold(objects[0].mesh.bounding_box, |acc, obj| obj.mesh.bounding_box.merge(&acc));
        let dim = UVec3::new(10,10,10);
        let voxel_size = (bounding_box.max - bounding_box.min).component_div(&dim.cast());
        let voxels = None;
        let slice_range = (UVec3::new(0,0,0), dim);
        let mut v = Self {
            bounding_box: bounding_box.clone(), 
            dim: dim, 
            voxels: voxels, 
            voxel_size: voxel_size, 
            voxel_mask: [false,false,false], 
            voxels_dirty: true,
            slice_range: slice_range
        };
        v.generate_voxels(objects);
        v
    }
    
    pub fn get_coin(&self, pos: glm::TVec3<RayPacketValue>) -> Option<QuantumCoin>{
        let map_pos = pos.map(|p| p.int().cast()); 
        self.voxels.as_ref().unwrap().get((map_pos.x, map_pos.y, map_pos.z)).and_then(|v| Some(v.coin))
    }

    pub fn get_sample_diffuse(&self, pos: glm::TVec3<RayPacketValue>) -> glm::Vec3{
        // let map_pos = pos.map(|p| p.int().cast()); 

        // self.voxels.as_ref().unwrap()[(map_pos.x, map_pos.y, map_pos.z)].coin.sample_difuse()
        self.get_coin(pos).and_then(|c| Some(c.sample_difuse())).unwrap_or(glm::vec3(1.0,1.0,1.0))
    }

    pub fn get_sample_emited(&self, pos: glm::TVec3<RayPacketValue>) -> glm::Vec3{
        // let map_pos = pos.map(|p| p.int().cast()); 
        // self.voxels.as_ref().unwrap()[(map_pos.x, map_pos.y, map_pos.z)].coin.sample_emited()
        self.get_coin(pos).and_then(|c| Some(c.sample_emited())).unwrap_or(glm::vec3(1.0,1.0,1.0))

    }

    pub fn convert_to_voxel_space(&self, pos: glm::Vec3) -> glm::Vec3{
        self.dim.map(|v| (v as f32 -1.0) ).component_mul(&(pos - self.bounding_box.min).component_div(&(self.bounding_box.max - self.bounding_box.min)))
    }

    pub fn intersect_ray(&self, ray_org: glm::Vec3, ray_inv_dir: glm::Vec3) -> Option<(f32, (usize, usize, usize))>{
        let (t, index) = self.voxels.as_ref().unwrap().indexed_iter().fold((f32::INFINITY , (0,0,0)), |(t_min, i_min), (index, vox)|  {
            if matches!(vox.coin, QuantumCoin::Surface{..}){
                let new_t = vox.intersect_ray(ray_org, ray_inv_dir).unwrap_or(f32::INFINITY);
                if new_t < t_min{
                    (new_t, index)
                }
                else{
                    (t_min, i_min)
                }
            }
            else{
                (t_min, i_min)
            }   
        });

        if t != f32::INFINITY{
            Some((t, index))
        }
        else{
            None
        }


    }

    fn sdf(position: Vec3, objects: &Vec<Arc<Object>>)-> f32{
        let mut min_dist = f32::INFINITY;
        for obj in objects{
            let dist = obj.sdf(position);
            if dist < min_dist{
                min_dist = dist;
            }
        }
        min_dist
    }
    fn get_normal(position: Vec3, objects: &Vec<Arc<Object>>, h: &f32) -> Vec3{
        // https://iquilezles.org/articles/normalsSDF/
        // let k = Vec2::new(1.0, -1.0);
        // let v = (
        //     k.xyy() * Self::sdf(position + *h * k.xyy(), objects) +
        //     k.yyx() * Self::sdf(position + *h * k.yyx(), objects) +
        //     k.yxy() * Self::sdf(position + *h * k.yxy(), objects) +
        //     k.xxx() * Self::sdf(position + *h * k.xxx(), objects)
        // ).normalize();
        // println!("{:?}",v);
        // v
        let mut min_distatnce = f32::INFINITY;
        let mut normal = glm::vec3(0.0,0.0,0.0);
        for obj in objects{
            let (norm, dist) = obj.closest_normal(position);
            if dist < min_distatnce{
                min_distatnce = dist;
                normal = norm;
            }
        }
        normal
    }
    fn generate_voxels(&mut self, objects: &Vec<Arc<Object>>){
        (self.bounding_box.min,self.bounding_box.max) = self.bounding_box.max.inf_sup(&self.bounding_box.min);
        self.voxel_size = (self.bounding_box.max - self.bounding_box.min).component_div(&self.dim.cast());

        self.voxels = Some(Array3::<Voxel>::from_shape_vec((self.dim.x as usize, self.dim.y as usize, self.dim.z as usize), 
        (0..self.dim.x).cartesian_product(0..self.dim.y).cartesian_product(0..self.dim.z)
        .map(|((x,y),z)| glm::vec3(x as f32,y as f32,z as f32))
        .collect_vec()
        .into_par_iter()
        .map(|index| {
            let pos = index.component_mul(&self.voxel_size) + self.bounding_box.min;
            let voxel = Voxel::new_min_size(pos, self.voxel_size);
            if let Some((diffsue, e)) = objects.iter().filter_map(|obj| obj.intersect(voxel)).next(){
                voxel.add_surface(diffsue, e, Self::get_normal(pos, objects,&self.voxel_size.x))
            }
            else{
                voxel
            } 
        }).collect()
        ).unwrap());
    }

    pub fn update(&mut self, objects: &Vec<Arc<Object>>){
        if self.voxels_dirty{
            self.generate_voxels(&objects)
        }
    } 
}