use std::collections::HashMap;
use super::var_info::VarInfo;
use super::semantic_cube::SemanticCube;

#[derive(Debug)]
pub struct QuadData {
    pub operator_stack: Vec<String>,
    pub operand_stack: Vec<VarInfo>,
    pub jump_stack: Vec<usize>,
    pub quad_counter: usize,
    pub param_counter: usize,
    pub temp_counter: usize,
    pub semantic_cube: SemanticCube,
    pub memmory_config: [[u32; 2]; 11],
    pub operator_config: HashMap<String, usize>
}

impl QuadData {
    pub fn new() -> Self {
        QuadData {
            operator_stack: Vec::new(),
            operand_stack: Vec::new(),
            jump_stack: Vec::new(),
            quad_counter: 1,
            param_counter: 0,
            temp_counter: 1,
            semantic_cube: SemanticCube::new(),
            memmory_config: [
                // ---- Global ----
                [1000, 2999], // 0. Ints
                [3000, 4999], // 1. Floats
                [5000, 6999], // 2. Temporal Intss
                [7000, 8999], // 3. Temporal Floats
                // ---- Local ----
                [11000, 12999], // 4. Ints
                [13000, 14999], // 5. Floats
                [15000, 16999], // 6. Temporal Ints
                [17000, 18999], // 7. Temporal Floats
                // ---- Constants ----
                [21000, 22999], // 8. Ints
                [23000, 24999], // 9. Floats
                [25000, 26999], // 10. Strings
            ],
            operator_config: {
                let mut map = HashMap::new();
                map.insert(String::from("+"), 1);
                map.insert(String::from("-"), 2);
                map.insert(String::from("*"), 3);
                map.insert(String::from("/"), 4);
                map.insert(String::from("<"), 5);
                map.insert(String::from(">"), 6);
                map.insert(String::from("=="), 7);
                map.insert(String::from("!="), 8);
                map.insert(String::from("="), 9);
                map.insert(String::from("goto"), 10);
                map.insert(String::from("gotof"), 11);
                map.insert(String::from("era"), 12);
                map.insert(String::from("param"), 13);
                map.insert(String::from("gosub"), 14);
                map.insert(String::from("print"), 15);
                map.insert(String::from("end"), 16);
                map.insert(String::from("endfunc"), 17);
                map
            }
        }
    }

    pub fn get_memory_segment(&self, var_type: &str, current_func: &str, kind: &str) -> u32 {
        match(var_type, current_func, kind) {
            ("int", "global", "regular") => self.memmory_config[0][0],
            ("float", "global", "regular") => self.memmory_config[1][0],
            ("int", "global", "temporal") => self.memmory_config[2][0],
            ("float", "global", "temporal") => self.memmory_config[3][0],
            ("int", _, "regular") => self.memmory_config[4][0],
            ("float", _, "regular") => self.memmory_config[5][0],
            ("int", _, "temporal") => self.memmory_config[6][0],
            ("float", _, "temporal") => self.memmory_config[7][0],
            ("int", _, "constant") => self.memmory_config[8][0],
            ("float", _, "constant") => self.memmory_config[9][0],
            ("string", _, "constant") => self.memmory_config[10][0],
            _ => 999999
        }
    }
}