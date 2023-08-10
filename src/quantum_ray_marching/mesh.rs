use std::sync::Arc;

use itertools::Itertools;

use tobj;

use super::{voxels::Voxel, vertex::Vertex};


pub struct Mesh{
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub bounding_box: Voxel,
}


impl Mesh{
    pub fn new(name: String, vertices:Vec<Vertex>) -> Mesh{
        let max = glm::Vec3::new(
            vertices.iter().map(|v| v.position[0]).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            vertices.iter().map(|v| v.position[1]).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            vertices.iter().map(|v| v.position[2]).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        );
        let min = glm::Vec3::new(
            vertices.iter().map(|v| v.position[0]).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            vertices.iter().map(|v| v.position[1]).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
            vertices.iter().map(|v| v.position[2]).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
        ); 
        
        Mesh{name, vertices, bounding_box: Voxel::new(min, max)}
    }

    pub fn from_obj(file_name: &String) -> Vec<Mesh>{
        let obj_flie = tobj::load_obj(file_name, &tobj::GPU_LOAD_OPTIONS);

        let (models, materials) = obj_flie.unwrap();
        let materials = materials.unwrap();
        //let models  = vec![models[0].clone(), models[2].clone(), models[3].clone(), models[6].clone(), models[7].clone()];
        models.iter().enumerate().map(|(_, m)|{
            let mut vertices = vec!();
            let mesh = &m.mesh;
            // Normals and texture coordinates are also loaded, but not printed in this example
            assert!(mesh.indices.len() % 3 == 0);
            for f in 0..mesh.indices.len() / 3 {
                for v in &mesh.indices[(3 * f )..(3 * f + 3)]{
                    let vertex_range = (3 * v ) as usize .. (3 * v + 3)as usize;
                    let position = mesh.positions[vertex_range.clone()].try_into().unwrap();
                    let normal = if !mesh.normals.is_empty(){
                        mesh.normals[vertex_range].try_into().unwrap()
                    }
                    else {
                        [1.0, 0.0, 0.0]
                    };
                    let colour = materials[mesh.material_id.unwrap()].diffuse;
                    let emisize = materials[mesh.material_id.unwrap()].unknown_param.get("Ke").unwrap().split_ascii_whitespace().map(|s| s.parse::<f32>().unwrap()).collect::<Vec<f32>>();

                    vertices.push(Vertex::new(position, normal, colour, emisize.try_into().unwrap()));

                
                    // let position = [
                    //     mesh.positions[(3* *v) as usize], 
                    //     mesh.positions[((3* *v)+1) as usize], 
                    //     mesh.positions[((3* *v)+2) as usize]
                    // );
                    // let normal = [
                    //     mesh.normals[(3* *v) as usize], 
                    //     mesh.normals[((3* *v)+1) as usize], 
                    //     mesh.normals[((3* *v)+2) as usize]
                    // ];
                }
            }
            (m.name.clone(), vertices)
        }).map(|(name,verts)| Mesh::new(name, verts)).collect::<Vec<Mesh>>()

        // for (i, m) in materials.iter().enumerate() {
        //     println!("material[{}].name = \'{}\'", i, m.name);
        //     println!(
        //         "    material.Ka = ({}, {}, {})",
        //         m.ambient[0], m.ambient[1], m.ambient[2]
        //     );
        //     println!(
        //         "    material.Kd = ({}, {}, {})",
        //         m.diffuse[0], m.diffuse[1], m.diffuse[2]
        //     );
        //     println!(
        //         "    material.Ks = ({}, {}, {})",
        //         m.specular[0], m.specular[1], m.specular[2]
        //     );
        //     println!("    material.Ns = {}", m.shininess);
        //     println!("    material.d = {}", m.dissolve);
        //     println!("    material.map_Ka = {}", m.ambient_texture);
        //     println!("    material.map_Kd = {}", m.diffuse_texture);
        //     println!("    material.map_Ks = {}", m.specular_texture);
        //     println!("    material.map_Ns = {}", m.shininess_texture);
        //     println!("    material.map_Bump = {}", m.normal_texture);
        //     println!("    material.map_d = {}", m.dissolve_texture);

        //     for (k, v) in &m.unknown_param {
        //         println!("    material.{} = {}", k, v);
        //     }
        // }
    

        // Mesh::new(vertices, allocator)
    }
}