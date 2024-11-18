#[derive(Debug)]
pub struct GlobalMemory {
    pub ints: Vec<i32>,
    pub int_temps: Vec<i32>,
    pub floats: Vec<f32>,
    pub float_temps: Vec<f32>,
    pub int_consts: Vec<i32>,
    pub float_consts: Vec<f32>,
    pub string_const: Vec<String>,
    pub memory_stack: Vec<LocalMemory>,
    pub jump_stack: Vec<usize>,
}

impl GlobalMemory {
    pub fn new(i_size: usize, it_size: usize, f_size: usize, ft_size: usize, ic_size: usize, fc_size: usize, sc_size: usize) -> GlobalMemory {
        GlobalMemory {
            ints: vec![std::i32::MIN; i_size],
            int_temps: vec![std::i32::MIN; it_size],
            floats: vec![std::f32::MIN; f_size],
            float_temps: vec![std::f32::MIN; ft_size],
            int_consts: vec![std::i32::MIN; ic_size],
            float_consts: vec![std::f32::MIN; fc_size],
            string_const: vec!["".to_string(); sc_size],
            memory_stack: Vec::new(),
            jump_stack: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct LocalMemory {
    pub ints: Vec<i32>,
    pub int_temps: Vec<i32>,
    pub floats: Vec<f32>,
    pub float_temps: Vec<f32>,
}

impl LocalMemory {
    pub fn new(i_size: usize, it_size: usize, f_size: usize, ft_size: usize) -> LocalMemory {
        LocalMemory {
            ints: vec![std::i32::MIN; i_size],
            int_temps: vec![std::i32::MIN; it_size],
            floats: vec![std::f32::MIN; f_size],
            float_temps: vec![std::f32::MIN; ft_size],
        }
    }
}

#[derive(Debug)]
pub enum MemorySegment {
    Ints,
    IntTemps,
    Floats,
    FloatTemps,

    IntLocal,
    FloatLocal,
    IntLocalTemps,
    FloatLocalTemps,

    IntConsts,
    FloatConsts,
    StringConsts,
}