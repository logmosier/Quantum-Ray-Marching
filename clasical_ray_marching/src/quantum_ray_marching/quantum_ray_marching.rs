// use crate::{
//     core::camera::{self, Camera},
//     rendering::{
//         gui::UiDefiner,
//         voxels::{self, Voxels}, renderer::Renderer, vulkano_objects,
//     }, shaders,
// };
// use egui::{DragValue, Ui, Button, epaint::tessellator::PathType};

use glm::{Vec3};
use indicatif::{ProgressIterator, ProgressStyle, ParallelProgressIterator};
use itertools::Itertools;

use nalgebra_glm::UVec2;
use rand::{seq::IteratorRandom, thread_rng};
use rayon::prelude::{ParallelIterator, IntoParallelIterator, IndexedParallelIterator};
use serde::Serialize;

// use vulkano::{command_buffer::{SecondaryAutoCommandBuffer, AutoCommandBufferBuilder, CommandBufferUsage, CommandBufferInheritanceInfo}, pipeline::{GraphicsPipeline, graphics::{vertex_input::BuffersDefinition, input_assembly::{InputAssemblyState, PrimitiveTopology}, viewport::ViewportState, depth_stencil::DepthStencilState, rasterization::{PolygonMode, RasterizationState}}, Pipeline}, render_pass::Subpass, buffer::{CpuAccessibleBuffer, BufferUsage, TypedBufferAccess}, shader::reflect};
use std::{fs::{File}, io::Write};

use image::{RgbImage, Rgb};
use super::{camera::Camera, ray_packet::RayPacket, dirction_map::DirMap, path::{QuantumSample, SamplePath}, voxels::Voxels};

#[derive(Debug, Clone, Serialize)]
pub struct PixelData {
    pub position: UVec2,
    pub paths: Vec<SamplePath>,
}
impl PixelData {
    pub fn new(position: UVec2, samples: &Vec<QuantumSample>) -> Self {
        let paths = samples.iter().map(|s|s.path.clone()).collect();
        PixelData { position, paths}
    }
}

pub struct QuantumRayMarcherer {
    direction_num: usize,
    steps: usize,
    camera: Camera,
    pub cheat: bool,
    paths: Option<Vec<PixelData>>,
    num_paths: i32,
}

impl QuantumRayMarcherer {
    pub fn new(camera: Camera) -> Self {
        QuantumRayMarcherer {
            direction_num: 15,
            steps: 4,
            camera,
            cheat: true,
            paths: None,
            num_paths: -1
        }
    }

    pub fn process_packet(&self, voxels: &Voxels, packet: &mut RayPacket, d_map: &DirMap) -> (Vec3, Vec<QuantumSample>) {
        let paths =  packet.evaluate(voxels, d_map, self.cheat, self.steps, QuantumSample::new());
        let used_paths = if self.num_paths >0{
            paths.into_iter().choose_multiple(&mut thread_rng(), self.num_paths as usize)
        }
        else{
            paths
        };
        println!("used_paths: {:?} vs contributing: {:?}", used_paths.len(), used_paths.iter().filter(|v| v.value.is_some_and(|v| v.norm() > 0.0)).collect_vec().len() );
        let contributing = used_paths.iter().filter(|v| v.value.is_some_and(|v| v.norm() > 0.0)).collect_vec().len();
        let (paths_sum) = used_paths.iter().fold(Vec3::zeros(), |acc, p| {
            acc + p.value.unwrap()
        });
        (paths_sum.component_div(&Vec3::from_element(used_paths.len() as f32)), used_paths)
    }

    pub fn write_paths(&self){
        if let Some(p) = self.paths.as_ref(){
            let serialzed = serde_json::to_vec(p).unwrap();
            File::create("paths.json").unwrap().write_all(&serialzed).unwrap();
        }
    }

    pub fn render(&mut self, voxels: &Voxels) {
        // let voxels = Self::prepare_voxels(self.voxels.clone());
        let d_map = DirMap::new(self.direction_num, false);
        let mut img = RgbImage::new(self.camera.pixels.x, self.camera.pixels.y);
        //only does gray scale images right now need to run 3 times for color
        let style = ProgressStyle::default_bar();
        let mut pixels: Vec<(UVec2, Rgb<u8>, Vec<QuantumSample>)> =vec![];
        let start_points = self.camera.get_start_points(voxels, &d_map);
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
        self.paths = Some(pixels.iter().map(|(pixel, _, paths)| PixelData::new(*pixel, paths)).collect_vec());
        img.save("test.png").unwrap();
        self.write_paths();
    }
}