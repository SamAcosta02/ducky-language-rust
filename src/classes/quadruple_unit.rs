#[derive(Debug)]
pub struct QuadrupleUnit {
    pub name: String,
    pub memory: u32,
}

impl QuadrupleUnit {
    pub fn new(name: String, memory: u32) -> Self {
        QuadrupleUnit {
            name,
            memory,
        }
    }
}