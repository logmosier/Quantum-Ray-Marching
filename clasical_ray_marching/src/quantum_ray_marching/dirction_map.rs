use std::f32::consts::PI;

use fixed::FixedU16;
use glm::{Vec3, distance};
use itertools::Itertools;
use super::ray_packet::RayPacketValue;


pub struct DirMap {
    directions: Vec<Vec3>,
    pub direction_num: usize,
}

impl DirMap {
    pub fn new(num_directions: usize, random: bool) -> Self {
        let directions = if random {
            (0..num_directions)
                .into_iter()
                .map(|_| {
                    Vec3::new(
                        rand::random::<f32>(),
                        rand::random::<f32>(),
                        rand::random::<f32>(),
                    )
                })
                .collect()
        } else {
            let phi = PI * (3.0 - f32::sqrt(5.0)); // golden angle in rad
            (0..num_directions)
                .into_iter()
                .map(|i| {
                    let y = 1.0 - (i as f32 / (num_directions - 1) as f32) * 2.0; // y goes from 1 to -1
                    let radius = f32::sqrt(1.0 - y * y); // radius at y
                    let theta = phi * i as f32; // golden angle increment
                    let x = f32::cos(theta) * radius;
                    let z = f32::sin(theta) * radius;
                    Vec3::new(x, y, z).normalize()
                })
                .collect()
        };

        DirMap {
            directions,
            direction_num: num_directions,
        }
    }

    pub fn step(&self, p: glm::TVec3<RayPacketValue>, dir: usize) -> glm::TVec3<RayPacketValue> {
        let dir = self.directions[dir].map(|v| FixedU16::from_num(v));
        p + dir
    }
    pub fn map(&self, dir: Vec3) -> usize {
        self.directions
            .iter()
            .enumerate()
            .min_by(
                |(_, a), (_, b)| 
                {
                    if let Some(o) = distance(*a, &dir).partial_cmp(&distance(*b, &dir)){
                        o
                    }
                    else {
                        std::cmp::Ordering::Equal
                    }
                }
            )
            .map(|(index, _)| index)
            .unwrap()
    }
    pub fn scatter(&self, normal: &Vec3) ->  Vec<usize>{
        self.directions.iter().enumerate().filter_map(|(i, dir)| {
            if dir.dot(&normal) > 0.0{
                Some(i)
            }
            else {
                None
            }
        }).collect_vec()
    }
}
