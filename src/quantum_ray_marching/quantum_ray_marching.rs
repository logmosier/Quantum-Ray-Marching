// use crate::{
//     core::camera::{self, Camera},
//     rendering::{
//         gui::UiDefiner,
//         voxels::{self, Voxels}, renderer::Renderer, vulkano_objects,
//     }, shaders,
// };
// use egui::{DragValue, Ui, Button, epaint::tessellator::PathType};
use fixed::{types::extra::{U3, U0}, FixedU8, traits::Fixed, FixedU16};
use glm::{distance, Vec3, Vec2, UVec2};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle, ParallelProgressIterator};
use itertools::Itertools;
use ndarray::Array;
use rayon::prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator, IntoParallelIterator, IndexedParallelIterator};
use serde::Serialize;
// use vulkano::{command_buffer::{SecondaryAutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferUsage, CommandBufferInheritanceInfo}, pipeline::{GraphicsPipeline, graphics::{vertex_input::BuffersDefinition, input_assembly::{InputAssemblyState, PrimitiveTopology}, viewport::ViewportState, depth_stencil::DepthStencilState, rasterization::{PolygonMode, RasterizationState}}, Pipeline}, render_pass::Subpass, buffer::{CpuAccessibleBuffer, BufferUsage, TypedBufferAccess}, shader::reflect};
use std::{f32::consts::PI, sync::Arc, fs::{File, self}, io::Write, collections::HashMap};
use std::io::Cursor;
use image::{io::Reader as ImageReader, ImageBuffer, Luma, GrayImage, RgbImage};
use super::{coins::QuantumCoin, camera::Camera, ray_packet::RayPacket, dirction_map::DirMap, path::QuantumSample, voxels::Voxels};

pub struct QuantumRayMarcherer {
    direction_num: usize,
    steps: usize,
    camera: Camera,
    packets: Vec<RayPacket>,
    pub render_flag: bool,
    pub cheat: bool,
}

impl QuantumRayMarcherer {
    pub fn new(camera: Camera, voxels: &Voxels) -> Self {
        let d_map = DirMap::new(15, false);
        let packets = camera.get_start_points(voxels, &d_map).iter().map(|(_, p)|p.clone()).collect();

        QuantumRayMarcherer {
            direction_num: 15,
            steps: 1,
            camera: Camera::new(
                Vec3::new(0.0, 2.0, 10.0),
                Vec3::new(0.0, 0.0, 0.0),
                (70.0f32).to_radians(),
                glm::vec2(100, 100),
            ),
            packets: packets,
            render_flag: false,
            cheat: false,
        }
    }

    pub fn process_packet(&self, voxels: &Voxels, packet: &mut RayPacket, d_map: &DirMap) -> (Vec3, Vec<QuantumSample>) {
        let paths =  packet.evaluate(voxels, d_map, self.cheat, self.steps, QuantumSample::new());
        // println!("paths: {:?}", paths);
        let (n, paths_sum) = paths.iter().fold((0.0, Vec3::zeros()), |(n,acc), p| {
            (
                if p.value.unwrap().norm() > 0.0 {n + 1.0} else {n},
                acc + p.value.unwrap()
            )
        });
        (paths_sum.component_div(&Vec3::new(n, n, n)), paths)
    }

    pub fn render(&mut self, voxels: &Voxels, camera: &Camera) {
        self.render_flag = false;
        // let voxels = Self::prepare_voxels(self.voxels.clone());
        let d_map = DirMap::new(self.direction_num, false);
        let mut img = RgbImage::new(self.camera.pixels.x, self.camera.pixels.y);
        //only does gray scale images right now need to run 3 times for color
        let style = ProgressStyle::default_bar();
        let mut pixels =vec![];
        let start_points = camera.get_start_points(voxels, &d_map);
        println!("Start Points: {:?}", start_points.len());
        // progress_with_style(style)
        start_points.into_par_iter().progress_with_style(style)
            .map(|(pixel, mut start_packet)| {
            let (value, path) = self.process_packet(
                voxels, 
                &mut start_packet, 
                &d_map, 
            );
            (
                pixel,  
                image::Rgb(value.map(|v| (v  * 255.0).clamp(0.0, 255.0) as u8 ).into()),
                path
            )
        }).collect_into_vec(&mut pixels);
        for (pixel, value, _) in pixels.iter(){
            img.put_pixel(pixel.x, pixel.y, *value);
            // let path_string = path.iter().map(|p| format!("p,{:?}", p.path.iter().map(|(emited, diffuse)| format!("{:?}:{:?}",emited,diffuse)).join("-"))).join("\n");
            // write!(file, "i,{},{}\n{}\n", pixel.x, pixel.y, path_string).unwrap();
        }
        let paths = pixels.iter().map(|(_, _, paths)| paths.iter().map(|sample| sample.path.clone()).collect_vec()).collect_vec();
        let pixels = pixels.iter().map(|(pixel, _, _)| pixel).collect_vec();
        let serialzed = serde_json::to_vec(&(pixels, paths)).unwrap();
        File::create("paths.json").unwrap().write_all(&serialzed).unwrap();
        img.save("test.png").unwrap();
    }
}