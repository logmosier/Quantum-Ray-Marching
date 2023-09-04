use serde::Serialize;

pub type SamplePath = Vec<(glm::Vec3, glm::Vec3)>;
#[derive(Clone, Debug, Serialize)]
pub struct QuantumSample {
    pub r: glm::Vec3,
    pub value: Option<glm::Vec3>,
    pub path: SamplePath, 
}

impl QuantumSample {
    pub fn new() -> Self {
        QuantumSample {
            r: glm::Vec3::from_element(1.0) ,
            value: None,
            path: vec![],
        }
    }
    pub fn update(&mut self, difuse: glm::Vec3, emited: glm::Vec3){
        self.r = self.r.component_mul(&difuse);
        if let Some(value)= self.value{
            self.value = Some(value + self.r.component_mul(&emited));
        }
        else{
            self.value = Some(emited);
        }
        self.path.push((difuse, emited));
    }
}