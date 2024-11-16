pub struct FunctionMemory {
    pub ints: Vec<i32>,
    pub floats: Vec<f32>,
    pub temp_ints: Vec<i32>,
    pub temp_floats: Vec<f32>,
    pub return_to: usize,
}