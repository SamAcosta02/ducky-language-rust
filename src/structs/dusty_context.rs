use std::collections::{HashMap, VecDeque};

use super::function_info::FunctionInfo;
use super::quadruple_unit::QuadrupleUnit;
use super::var_info::VarInfo;
use super::parser::Rule;
use super::quad_data::QuadData;

#[derive(Debug)]
pub struct DustyContext {
    pub func_dir: HashMap<String, FunctionInfo>, // Function-variable scope directory
    pub const_dir: HashMap<String, VarInfo>, // Constant directory
    pub parent_rules: Vec<Rule>,
    pub current_type: String,
    pub current_func: String,
    pub current_call: String,
    pub id_stack: Vec<String>,
    pub quad_data: QuadData,
    pub quadruples: VecDeque<[QuadrupleUnit; 4]>,
    pub constants: [u32; 3]
}

#[derive(Debug)]
pub enum Stage {
    Before,
    During,
    After,
    Finished,
}

impl DustyContext {
    pub fn new() -> Self {
        DustyContext {
            func_dir: HashMap::new(),
            const_dir: HashMap::new(),
            id_stack: Vec::new(),
            parent_rules: vec![Rule::program],
            current_func: String::new(),
            current_call: String::new(),
            current_type: String::new(),
            quad_data: QuadData::new(),
            quadruples: VecDeque::new(),
            constants: [0,0,0]
        }
    }

    pub fn contains_id(&self, id: &str) -> bool {
        self.func_dir.get(&self.current_func).unwrap().contains_key(id)
    }

    pub fn id_in_global_scope(&self, id: &str) -> bool {
        self.func_dir.get("global").unwrap().contains_key(id)
    }
    
    pub fn top_is_multiplication_or_division(&self) -> bool {
        self.quad_data.operator_stack.last() == Some(&String::from("*")) || self.quad_data.operator_stack.last() == Some(&String::from("/"))
    }

    pub fn top_is_addition_or_subtraction(&self) -> bool {
        self.quad_data.operator_stack.last() == Some(&String::from("+")) || self.quad_data.operator_stack.last() == Some(&String::from("-"))
    }

    pub fn top_is_logical_operator(&self) -> bool {
        self.quad_data.operator_stack.last() == Some(&String::from("==")) || self.quad_data.operator_stack.last() == Some(&String::from("!="))
        || self.quad_data.operator_stack.last() == Some(&String::from(">")) || self.quad_data.operator_stack.last() == Some(&String::from("<"))
    }

    pub fn top_is_equals(&self) -> bool {
        self.quad_data.operator_stack.last() == Some(&String::from("="))
    }

    pub fn generate_full_quad(&mut self) {
        // Get Operands and Operator
        let right_operand = self.quad_data.operand_stack.pop()
            .expect("ERROR: Missing right operand");
        let left_operand = self.quad_data.operand_stack.pop()
            .expect("ERROR: Missing left operand");
        let operator = self.quad_data.operator_stack.pop()
            .expect("ERROR: Missing operator");

        // Check if types are compatible
        if self.quad_data.semantic_cube.get_result_type(&left_operand.var_type, &right_operand.var_type, &operator) == "error" {
            panic!("ERROR: Type mismatch. Cannot use {} with {} and {}.", operator, left_operand.var_type, right_operand.var_type);
        }

        // get result type
        let result_type = self.quad_data.semantic_cube.get_result_type(&left_operand.var_type, &right_operand.var_type, &operator);

        // Get temp variable information
        let name = format!("t{}", self.quad_data.temp_counter);
        let var_type = result_type.clone();
        let base = self.quad_data.get_memory_segment(&var_type, &self.current_func, "temporal");
        let counter = self.func_dir.get_mut(&self.current_func).unwrap().get_counter(&var_type, "temporal");
        // let kind: String = "temporal".to_string();

        let result = VarInfo::new(name.clone(), var_type.clone(), base+counter);

        self.quadruples.push_back([
            QuadrupleUnit::new(
                operator.clone(),
                self.quad_data.operator_config.get(&operator).unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                left_operand.name,
                left_operand.location
            ),
            QuadrupleUnit::new(
                right_operand.name,
                right_operand.location
            ),
            QuadrupleUnit::new(
                result.name.clone(),
                result.location.clone()
            )
        ]);
        self.quad_data.quad_counter += 1;
        self.quad_data.temp_counter += 1;
        
        self.quad_data.operand_stack.push(result.clone());
        self.func_dir.get_mut(&self.current_func).unwrap().add_to_counter(&self.current_type, "temporal");
    }

    pub fn generate_assign_quad(&mut self) {
        let right_operand = self.quad_data.operand_stack.pop()
            .expect("ERROR: Missing right operand");
        let left_operand = self.quad_data.operand_stack.pop()
            .expect("ERROR: Missing left operand");
        let operator = self.quad_data.operator_stack.pop()
            .expect("ERROR: Missing operator");

        if self.quad_data.semantic_cube.get_result_type(&left_operand.var_type, &right_operand.var_type, &operator) == "error" {
            panic!("ERROR: Type mismatch. Cannot assign {} to {}", right_operand.var_type, left_operand.var_type);
        }

        self.quadruples.push_back([
            QuadrupleUnit::new(
                operator.clone(),
                self.quad_data.operator_config.get(&operator).unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                right_operand.name,
                right_operand.location
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                left_operand.name,
                left_operand.location
            )
        ]);
        self.quad_data.quad_counter += 1;
    }

    pub fn generate_print_quad(&mut self) {
        let element = self.quad_data.operand_stack.last();
        if element.is_none() {
            panic!("ERROR: Missing element to print");
        }
        self.quadruples.push_back([
            QuadrupleUnit::new(
                "print".to_string(),
                self.quad_data.operator_config.get("print").unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                element.unwrap().name.clone(),
                element.unwrap().location
            )
        ]);
        self.quad_data.quad_counter += 1;
    }

    pub fn generate_gotof_quad(&mut self) {
        // Check if top of operand stack is a int
        if self.quad_data.operand_stack.last().unwrap().var_type != "int" {
            panic!("ERROR: Expected int but got {}", self.quad_data.operand_stack.last().unwrap().var_type);
        }

        self.quadruples.push_back([
            QuadrupleUnit::new(
                "gotof".to_string(),
                self.quad_data.operator_config.get("gotof").unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                format!("t{}", self.quad_data.temp_counter - 1),
                self.quad_data.operand_stack.last().unwrap().location
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            )
        ]);
        self.quad_data.jump_stack.push(self.quad_data.quad_counter);
        self.quad_data.quad_counter += 1;
    }

    pub fn generate_goto_quad(&mut self) {
        self.quadruples.push_back([
            QuadrupleUnit::new(
                "goto".to_string(),
                self.quad_data.operator_config.get("goto").unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            )
        ]);
        self.quad_data.jump_stack.push(self.quad_data.quad_counter);
        self.quad_data.quad_counter += 1;
    }

    pub fn generate_gotow_quad(&mut self) {
        self.quadruples.push_back([
            QuadrupleUnit::new(
                "goto".to_string(),
                self.quad_data.operator_config.get("goto").unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            )
        ]);
        self.quad_data.quad_counter += 1;
    }

    pub fn generate_endfunc_quad(&mut self) {
        self.quadruples.push_back([
            QuadrupleUnit::new(
                "endfunc".to_string(),
                self.quad_data.operator_config.get("endfunc").unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            )
        ]);
        self.quad_data.quad_counter += 1;
    }

    pub fn generate_era_quad(&mut self, func_name: &str) {
        self.quadruples.push_back([
            QuadrupleUnit::new(
                "era".to_string(),
                self.quad_data.operator_config.get("era").unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                func_name.to_string(),
                self.func_dir.get(&self.current_call).unwrap().location
            )
        ]);
        self.quad_data.quad_counter += 1;
    }

    pub fn generate_param_quad(&mut self) { 
        let param = self.quad_data.operand_stack.last().unwrap();

        // Check for parameter overflow
        if self.quad_data.param_counter > self.func_dir.get(&self.current_call).unwrap().params.len() {
            panic!("ERROR: Too many parameters for function \"{}\"", self.current_call);
        }

        // Check current parameter type
        let var_type = &self.func_dir.get(&self.current_call).unwrap().params[self.quad_data.param_counter];
        if param.var_type != *var_type {
            panic!("ERROR: Type mismatch. Expected {} but got {}", var_type, param.var_type);
        }

        self.quadruples.push_back([
            QuadrupleUnit::new(
                "param".to_string(),
                self.quad_data.operator_config.get("param").unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                param.name.clone(),
                param.location.clone()
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                format!("param{}", self.quad_data.param_counter),
                self.quad_data.param_counter as u32
            )
        ]);
        self.quad_data.quad_counter += 1;
        self.quad_data.param_counter += 1;
    }

    pub fn generate_gosub_quad(&mut self) {
        self.quadruples.push_back([
            QuadrupleUnit::new(
                "gosub".to_string(),
                self.quad_data.operator_config.get("gosub").unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                self.current_call.to_string(),
                self.func_dir.get(&self.current_call).unwrap().location
            )
        ]);
        self.quad_data.quad_counter += 1;
    }

    pub fn generate_end_quad(&mut self) {
        self.quadruples.push_back([
            QuadrupleUnit::new(
                "end".to_string(),
                self.quad_data.operator_config.get("end").unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
        ]);
        self.quad_data.quad_counter += 1;
    }

    pub fn fill_jump(&mut self) {
        let jump = self.quad_data.jump_stack.pop().unwrap();
        self.quadruples[jump - 1][3] = QuadrupleUnit::new(
            format!("{}", self.quad_data.quad_counter), 
            self.quad_data.quad_counter as u32
        );
        // self.quadruples[jump - 1][3] = format!("{}", self.quad_data.quad_counter);
    }

    pub fn fill_while_start(&mut self) {
        let jump = self.quad_data.jump_stack.pop().unwrap();
        self.quadruples[jump - 1][3] = QuadrupleUnit::new(
            format!("{}", self.quad_data.quad_counter+1), 
            (self.quad_data.quad_counter+1) as u32
        );
        // self.quadruples[jump - 1][3] = format!("{}", self.quad_data.quad_counter+1);
    }

    pub fn fill_while_end(&mut self) {
        let jump = self.quad_data.jump_stack.pop().unwrap();
        self.quadruples[self.quad_data.quad_counter - 2][3] = QuadrupleUnit::new(
            format!("{}", jump), 
            jump as u32
        );
        // self.quadruples[self.quad_data.quad_counter - 2][3] = format!("{}", jump);
    }

    // pub fn print_quadruples_as_name(&self) {
    //     let mut counter = 1;
    //     for quad in &self.quadruples {
    //         print!("{}) [", counter);
    //         for unit in quad {
    //             print!("\"{}\", ", unit.name);
    //         }
    //         print!("] \n");
    //         counter += 1;
    //     }
    // }

    // pub fn print_quadruples_as_memmory(&self) {
    //     let mut counter = 1;
    //     for quad in &self.quadruples {
    //         print!("{}) [", counter);
    //         for unit in quad {
    //             print!("{}, ", unit.memory);
    //         }
    //         print!("] \n");
    //         counter += 1;
    //     }
    // }
}