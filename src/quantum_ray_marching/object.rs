use std::{sync::Arc, ops::Range};
use itertools::Itertools;
use glm::*;

use super::{mesh::Mesh, voxels::Voxel};

// #[derive(Send, Sync)]
pub struct Object{
    pub name: String,
    pub mesh: Arc<Mesh>,
    pub transform: glm::Mat4
}

impl Object{
    pub fn get_interval(triangle: &[glm::Vec3; 3], axis: &glm::Vec3) -> Range<f32>{
        let mut min = glm::dot(axis, &triangle[0]);
        let mut max = min;
        for v in triangle[1..].iter(){
            let p = glm::dot(axis, &v);
            min = min.min(p);
            max = max.max(p);
        }
        min .. max
    }

    pub fn overlap_on_axis(triangle: &[glm::Vec3; 3], axis: &glm::Vec3, voxel: Voxel) -> bool{
        let box_range = voxel.get_interval(axis); //implement box get interval self.get_interval(axis);
        let triangle_range = Object::get_interval(triangle, axis);
        triangle_range.start <= box_range.end && triangle_range.end >= box_range.start
    }

    pub fn get_num_ray_intersects(&self, org: glm::Vec3, dir: glm::Vec3) -> u32{
        self.mesh.vertices.chunks(3).fold(0, |accum, v| {
            let triangle_verts = v.iter().map(
                |v| (self.transform * glm::vec4(v.position[0] , v.position[1], v.position[2], 1.0)).xyz()
            ).collect::<Vec<_>>();
            let triangle = [triangle_verts[0], triangle_verts[1], triangle_verts[2]];
            
            let edges = vec![
                triangle[1] - triangle[0],
                triangle[2] - triangle[1],
                triangle[0] - triangle[2]
            ];

            let plane_norm = glm::cross(&edges[0], &edges[1]);
            let plane_norm_dot_dir = plane_norm.dot(&dir);
            if plane_norm_dot_dir == 0.0 {
                return accum;
            }

            let d = -plane_norm.dot(&triangle[0]);
            let t = -(plane_norm.dot(&org) + d) / plane_norm_dot_dir;
            if t < 0.0 {
                return accum;
            }

            let p = org + dir * t;
            for (edge, v) in edges.iter().zip(triangle.iter()){
                if plane_norm.dot(&edge.cross(&(p - v))) < 0.0 {
                    return accum;
                }
            }

            accum + 1
        })
    }

    pub fn intersect(&self, voxel: Voxel) -> Option<(glm::Vec3, glm::Vec3)>{
        self.mesh.vertices.chunks(3).find_map(|v|{
            let triangle_verts = v.iter().map(
                |v| (self.transform * glm::vec4(v.position[0] , v.position[1], v.position[2], 1.0)).xyz()
            ).collect::<Vec<_>>();
            let triangle = [triangle_verts[0], triangle_verts[1], triangle_verts[2]];
            let edges = vec![
                triangle[1] - triangle[0],
                triangle[2] - triangle[1],
                triangle[0] - triangle[2]
            ];

            let voxel_face_normals = vec![
                vec3(1.0, 0.0, 0.0),
                vec3(0.0, 1.0, 0.0),
                vec3(0.0, 0.0, 1.0),
            ];

            let axis = [
                //Voxel Face Normals
                voxel_face_normals[0],
                voxel_face_normals[1],
                voxel_face_normals[2],

                //Triangle Face Normals
                edges[0].cross(&edges[1]),

                //Triangle Voxel Cross Normals
                voxel_face_normals[0].cross(&edges[0]),
                voxel_face_normals[0].cross(&edges[1]),
                voxel_face_normals[0].cross(&edges[2]),

                voxel_face_normals[1].cross(&edges[0]),
                voxel_face_normals[1].cross(&edges[1]),
                voxel_face_normals[1].cross(&edges[2]),

                voxel_face_normals[2].cross(&edges[0]),
                voxel_face_normals[2].cross(&edges[1]),
                voxel_face_normals[2].cross(&edges[2]),
                
            ];
            if axis.iter().all(|axis| Object::overlap_on_axis(&triangle, axis, voxel)){
                Some((
                    glm::vec3(v[0].color[0], v[0].color[1], v[0].color[2]),
                    glm::vec3(v[0].emissive[0], v[0].emissive[1], v[0].emissive[2])
                ))
            }
            else{
                None
            }
        })    
    }

    fn distance(&self, pos: Vec3) -> (usize, f32, Vec3) {
        // https://github.com/ranjeethmahankali/galproject/blob/main/galcore/Mesh.cpp
        self.mesh.vertices.chunks(3).enumerate().map(|(triangle_id, v)|{
            let triangle_verts = v.iter().map(
                |v| (self.transform * glm::vec4(v.position[0] , v.position[1], v.position[2], 1.0)).xyz()
            ).collect::<Vec<_>>();
            let triangle = [triangle_verts[0], triangle_verts[1], triangle_verts[2]];
            let plane_normal = glm::cross(&(triangle[1] - triangle[0]), &(triangle[2] - triangle[0])).normalize();
            let v = pos - triangle[0];
            let d_to_plane = v.dot(&plane_normal).abs();
            let pos_prime = pos - d_to_plane * plane_normal;

            let mut outside = false;
            let mut d_min = f32::INFINITY;
            let mut closest_point = Vec3::zeros();
            for i in 0..3{
                let v1 = triangle[i];
                let v2 = triangle[(i + 1) % 3];
                let edge_normal = glm::cross(&(v2 - v1), &plane_normal).normalize();
                if plane_normal.dot(&edge_normal) < 0.0{ // outside
                    outside = true;
                    let edge = v2 - v1;
                    let point_on_edge = v1 + edge * (glm::dot(&edge, &(pos_prime - v1)) / edge.dot(&edge)).clamp(0.0, 1.0);
                    let d = glm::distance(&pos, &point_on_edge);
                    if d < d_min{
                        d_min = d;
                        closest_point = point_on_edge;
                    }
                }
            }
            if !outside {
                (triangle_id, d_to_plane, pos_prime)
            }
            else{
                (triangle_id, d_min, closest_point)
            }
        }).min_by(|a: &(usize, f32, Vec3) ,b:&(usize, f32, Vec3)| a.1.partial_cmp(&b.1).unwrap()).unwrap()    
    }

    pub fn sdf(&self, pos: Vec3) -> f32{
        let (triangle_id, d, point) = self.distance(pos);
        let v = self.mesh.vertices.chunks(3).nth(triangle_id).unwrap();
        let triangle_verts = v.iter().map(
            |v| (self.transform * glm::vec4(v.position[0] , v.position[1], v.position[2], 1.0)).xyz()
        ).collect::<Vec<_>>();
        let triangle = [triangle_verts[0], triangle_verts[1], triangle_verts[2]];
        let plane_normal = glm::cross(&(triangle[1] - triangle[0]), &(triangle[2] - triangle[0])).normalize();
        // println!("{:?} {:?} {:?} triangle {}", d, point, plane_normal, triangle_id);
        let u = pos - point;
        d * u.dot(&plane_normal).signum()
    }

    pub fn closest_normal(&self, pos: Vec3) -> (Vec3,f32){
        let (triangle, distacne,_) = self.distance(pos);
        (
        self.mesh.vertices.chunks(3).nth(triangle).unwrap().iter().map(
            |v| (self.transform * glm::vec4(v.normal[0] , v.normal[1], v.normal[2], 0.0)).xyz()
        ).collect::<Vec<_>>().iter().sum::<Vec3>().normalize(),
        distacne
        )
    }
}