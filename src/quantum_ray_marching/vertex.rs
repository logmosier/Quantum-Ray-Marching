pub struct Vertex{
    pub position: [f32; 3],
    pub emissive: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}
impl Vertex{
    pub fn new(position: [f32; 3], normal: [f32; 3], color: [f32; 3], emissive: [f32; 3]) -> Vertex{
        Vertex{
            position,
            emissive,
            normal,
            color
        }
    }
}