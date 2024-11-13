use std::{collections::{HashMap, VecDeque}, fs};

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "dusty.pest"]
pub struct DustyParser;

#[derive(Debug)]
struct QuadrupleUnit {
    name: String,
    memory: u32,
}

impl QuadrupleUnit {
    fn new(name: String, memory: u32) -> Self {
        QuadrupleUnit {
            name,
            memory,
        }
    }
}

#[derive(Debug)]
struct VarInfo {
    var_type: String,
    location: u32,
}

impl VarInfo {
    fn new(var_type: String, location: u32) -> Self {
        VarInfo {
            var_type,
            location,
        }
    }
    fn to_string(&self) -> String {
        self.var_type.clone()
    }
}


#[derive(Debug)]
struct Resources {
    int_count: u32,
    float_count: u32,
    temporal_count: u32,
}

impl Resources {
    fn new() -> Self {
        Resources {
            int_count: 0,
            float_count: 0,
            temporal_count: 0,
        }
    }
}

#[derive(Debug)]
struct FunctionInfo {
    return_type: String,
    location: u32,
    resources: Resources,
    counters: [u32; 3],
    vars: HashMap<String, VarInfo>,
}

impl FunctionInfo {
    fn new(location: u32) -> Self {
        FunctionInfo {
            return_type: String::from("void"),
            location,
            resources: Resources::new(),
            counters: [0, 0, 0],
            vars: HashMap::new(),
        }
    }

    fn contains_key(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }

    fn get(&self, key: &str) -> Option<&VarInfo> {
        self.vars.get(key)
    }

    fn insert(&mut self, key: String, var_type: String) {
       self.vars.insert(key, VarInfo::new(var_type, 0));
    }
}

#[derive(Debug)]
enum Stage {
    Before,
    During,
    After,
    Finished,
}

#[derive(Debug)]
struct SemanticCube {
    cube: [[[String; 9]; 2]; 2],
    string_to_usize: HashMap<String, usize>,
}

impl SemanticCube {
    fn new() -> Self {
        SemanticCube {
            cube: [
                // Left operand is int (0)
                [
                    [// Right operand int (0) for...
                        String::from("int"),   // +
                        String::from("int"),   // -
                        String::from("int"),   // *
                        String::from("float"), // /
                        String::from("int"),   // <
                        String::from("int"),   // >
                        String::from("int"),   // ==
                        String::from("int"),   // !=
                        String::from("int"),   // =
                    ],  
                    [// Right operand float (1) for +, -, *, /
                        String::from("float"), // +
                        String::from("float"), // -
                        String::from("float"), // *
                        String::from("float"), // /
                        String::from("error"),   // <
                        String::from("error"),   // >
                        String::from("error"),   // ==
                        String::from("error"),   // !=
                        String::from("error"),   // = 
                    ],
                ],
                // Left operand is float (1)
                [
                    [// Right operand int (0) for...
                        String::from("float"), // +
                        String::from("float"), // -
                        String::from("float"), // *
                        String::from("float"), // /
                        String::from("error"),   // <
                        String::from("error"),   // >
                        String::from("error"),   // ==
                        String::from("error"),   // !=
                        String::from("error"),   // =
                    ],  
                    [// Right operand float (1) for +, -, *, /
                        String::from("float"), // +
                        String::from("float"), // -
                        String::from("float"), // *
                        String::from("float"), // /
                        String::from("int"),   // <
                        String::from("int"),   // >
                        String::from("int"),   // ==
                        String::from("int"),   // !=
                        String::from("int"),   // =
                    ],
                ],
            ],
            string_to_usize: {
                let mut map = HashMap::new();
                map.insert(String::from("int"), 0);
                map.insert(String::from("float"), 1);
                map.insert(String::from("+"), 0);
                map.insert(String::from("-"), 1);
                map.insert(String::from("*"),2);
                map.insert(String::from("/"), 3);
                map.insert(String::from("<"), 4);
                map.insert(String::from(">"), 5);
                map.insert(String::from("=="), 6);
                map.insert(String::from("!="), 7);
                map.insert(String::from("="), 8);
                map
            }
        }
    }

    fn get_result_type(&self, left: &str, right: &str, operator: &str) -> String {
        let left_usize = self.string_to_usize.get(left).unwrap();
        let right_usize = self.string_to_usize.get(right).unwrap();
        let operator_usize = self.string_to_usize.get(operator).unwrap();
        self.cube[*left_usize][*right_usize][*operator_usize].clone()
    }
}

#[derive(Debug)]
struct QuadData {
    operator_stack: Vec<String>,
    operand_stack: Vec<[String; 2]>,
    jump_stack: Vec<usize>,
    quad_counter: usize,
    param_counter: usize,
    temp_counter: usize,
    semantic_cube: SemanticCube,
    memmory_config: [[u32; 2]; 9],
    operator_config: HashMap<String, usize>
}

impl QuadData {
    fn new() -> Self {
        QuadData {
            operator_stack: Vec::new(),
            operand_stack: Vec::new(),
            jump_stack: Vec::new(),
            quad_counter: 1,
            param_counter: 1,
            temp_counter: 1,
            semantic_cube: SemanticCube::new(),
            memmory_config: [
                // ---- Global ----
                [1000,2999],  // 0. Ints
                [3000, 4999], // 1. Floats
                [5000, 6999], // 2. Temporal
                // ---- Local ----
                [11000, 12999], // 3. Ints
                [13000, 14999], // 4. Floats
                [15000, 16999], // 5. Temporal
                // ---- Constants ----
                [20000, 21999], // 6. Ints
                [22000, 23999], // 7. Floats
                [24000, 25999], // 8. Strings
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
}

#[derive(Debug)]
struct DustyContext {
    func_dir: HashMap<String, FunctionInfo>, // Function-variable scope directory
    parent_rules: Vec<Rule>,
    current_type: String,
    current_func: String,
    current_call: String,
    id_stack: Vec<String>,
    quad_data: QuadData,
    quadruples: VecDeque<[QuadrupleUnit; 4]>,
}

impl DustyContext {
    fn new() -> Self {
        DustyContext {
            func_dir: HashMap::new(),
            id_stack: Vec::new(),
            parent_rules: vec![Rule::program],
            current_func: String::new(),
            current_call: String::new(),
            current_type: String::new(),
            quad_data: QuadData::new(),
            quadruples: VecDeque::new(),
        }
    }

    fn contains_id(&self, id: &str) -> bool {
        self.func_dir.get(&self.current_func).unwrap().contains_key(id)
    }

    fn id_in_global_scope(&self, id: &str) -> bool {
        self.func_dir.get("global").unwrap().contains_key(id)
    }
    
    fn top_is_multiplication_or_division(&self) -> bool {
        self.quad_data.operator_stack.last() == Some(&String::from("*")) || self.quad_data.operator_stack.last() == Some(&String::from("/"))
    }

    fn top_is_addition_or_subtraction(&self) -> bool {
        self.quad_data.operator_stack.last() == Some(&String::from("+")) || self.quad_data.operator_stack.last() == Some(&String::from("-"))
    }

    fn top_is_logical_operator(&self) -> bool {
        self.quad_data.operator_stack.last() == Some(&String::from("==")) || self.quad_data.operator_stack.last() == Some(&String::from("!="))
        || self.quad_data.operator_stack.last() == Some(&String::from(">")) || self.quad_data.operator_stack.last() == Some(&String::from("<"))
    }

    fn top_is_equals(&self) -> bool {
        self.quad_data.operator_stack.last() == Some(&String::from("="))
    }

    fn generate_full_quad(&mut self) {
        // println!("{:?}",self.quad_data.operand_stack);
        let right_operand = self.quad_data.operand_stack.pop()
            .expect("ERROR: Missing right operand");
        let left_operand = self.quad_data.operand_stack.pop()
            .expect("ERROR: Missing left operand");
        let operator = self.quad_data.operator_stack.pop()
            .expect("ERROR: Missing operator");

        if self.quad_data.semantic_cube.get_result_type(&left_operand[1], &right_operand[1], &operator) == "error" {
            panic!("ERROR: Type mismatch. Cannot use {} with {} and {}.", operator, left_operand[1], right_operand[1]);
        }

        let result = format!("t{}", self.quad_data.temp_counter);
        self.quadruples.push_back([
            QuadrupleUnit::new(
                operator.clone(),
                self.quad_data.operator_config.get(&operator).unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                left_operand[0].clone(),
                0
            ),
            QuadrupleUnit::new(
                right_operand[0].clone(),
                0
            ),
            QuadrupleUnit::new(
                result.clone(),
                0
            )
        ]);
        self.quad_data.quad_counter += 1;
        self.quad_data.temp_counter += 1;
        self.quad_data.operand_stack.push([result.clone(), right_operand[1].clone()]);
    }

    fn generate_assign_quad(&mut self) {
        let right_operand = self.quad_data.operand_stack.pop()
            .expect("ERROR: Missing right operand");
        let left_operand = self.quad_data.operand_stack.pop()
            .expect("ERROR: Missing left operand");
        let operator = self.quad_data.operator_stack.pop()
            .expect("ERROR: Missing operator");

        if self.quad_data.semantic_cube.get_result_type(&left_operand[1], &right_operand[1], &operator) == "error" {
            panic!("ERROR: Type mismatch. Cannot assign {} to {}", right_operand[1], left_operand[1]);
        }

        self.quadruples.push_back([
            QuadrupleUnit::new(
                operator.clone(),
                self.quad_data.operator_config.get(&operator).unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                right_operand[0].clone(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                left_operand[0].clone(),
                0
            )
        ]);
        self.quad_data.quad_counter += 1;
    }

    fn generate_print_quad(&mut self) {
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
                element.unwrap()[0].clone(),
                0
            )
        ]);
        self.quad_data.quad_counter += 1;
    }

    fn generate_gotof_quad(&mut self) {
        self.quadruples.push_back([
            QuadrupleUnit::new(
                "gotof".to_string(),
                self.quad_data.operator_config.get("gotof").unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                format!("t{}", self.quad_data.temp_counter - 1),
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

    fn generate_goto_quad(&mut self) {
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

    fn generate_gotow_quad(&mut self) {
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

    fn generate_endfunc_quad(&mut self) {
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

    fn generate_era_quad(&mut self, func_name: &str) {
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
                0
            )
        ]);
        self.quad_data.quad_counter += 1;
    }

    fn generate_param_quad(&mut self) {
        let param = self.quad_data.operand_stack.last().unwrap();
        self.quadruples.push_back([
            QuadrupleUnit::new(
                "param".to_string(),
                self.quad_data.operator_config.get("param").unwrap().clone() as u32
            ),
            QuadrupleUnit::new(
                param[0].clone(),
                0
            ),
            QuadrupleUnit::new(
                "_".to_string(),
                0
            ),
            QuadrupleUnit::new(
                format!("param{}", self.quad_data.param_counter),
                0
            )
        ]);
        self.quad_data.quad_counter += 1;
        self.quad_data.param_counter += 1;
    }

    fn generate_gosub_quad(&mut self) {
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
                0
            )
        ]);
        self.quad_data.quad_counter += 1;
    }

    fn generate_end_quad(&mut self) {
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

    fn fill_jump(&mut self) {
        let jump = self.quad_data.jump_stack.pop().unwrap();
        self.quadruples[jump - 1][3] = QuadrupleUnit::new(
            format!("{}", self.quad_data.quad_counter), 
            self.quad_data.quad_counter as u32
        );
        // self.quadruples[jump - 1][3] = format!("{}", self.quad_data.quad_counter);
    }

    fn fill_while_start(&mut self) {
        let jump = self.quad_data.jump_stack.pop().unwrap();
        self.quadruples[jump - 1][3] = QuadrupleUnit::new(
            format!("{}", self.quad_data.quad_counter+1), 
            (self.quad_data.quad_counter+1) as u32
        );
        // self.quadruples[jump - 1][3] = format!("{}", self.quad_data.quad_counter+1);
    }

    fn fill_while_end(&mut self) {
        let jump = self.quad_data.jump_stack.pop().unwrap();
        self.quadruples[self.quad_data.quad_counter - 2][3] = QuadrupleUnit::new(
            format!("{}", jump), 
            jump as u32
        );
        // self.quadruples[self.quad_data.quad_counter - 2][3] = format!("{}", jump);
    }

    fn print_quadruples_as_name(&self) {
        let mut counter = 1;
        for quad in &self.quadruples {
            print!("{}) [", counter);
            for unit in quad {
                print!("\"{}\", ", unit.name);
            }
            print!("] \n");
            counter += 1;
        }
    }

    fn print_quadruples_as_memmory(&self) {
        let mut counter = 1;
        for quad in &self.quadruples {
            print!("{}) [", counter);
            for unit in quad {
                print!("{}, ", unit.memory);
            }
            print!("] \n");
            counter += 1;
        }
    }

    fn debug_quad_gen(&self) {
        println!("  Operator stack: {:?}", self.quad_data.operator_stack);
        println!("  Operand stack: {:?}", self.quad_data.operand_stack);
        println!("  Jump stack: {:?}", self.quad_data.jump_stack);
        println!("  temp counter: {}, quad_counter: {}", self.quad_data.temp_counter, self.quad_data.quad_counter);
        println!("  param counter: {}", self.quad_data.param_counter);
    }
}

fn process_pair(
    pair: pest::iterators::Pair<Rule>,
    stage: Stage,
    dusty_context: &mut DustyContext
) {
    println!("Processing rule: {:#?} in stage {:#?}, parent rule: {:#?}, currrent func: {:#?}, line: {:#?}, col: {:#?}",
        pair.as_rule(), stage, dusty_context.parent_rules.last().unwrap(), dusty_context.current_func,
        pair.as_span().start_pos().line_col().0,
        pair.as_span().start_pos().line_col().1
    );

    match (pair.as_rule(), &stage) {
        // Process beginKeyword ----------------------------
        (Rule::beginKeyword, Stage::Before) => {
            println!("  token BEGIN found:");
            println!("  Filling initial GOTO quad");
            dusty_context.fill_jump();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process beginKeyword ----------------------------

        // Process ID --------------------------------------
        (Rule::id, Stage::Before) => {
            println!("  Token ID found: {:#?}", pair.as_str());
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::id, Stage::During) => {
            match dusty_context.parent_rules.last().unwrap() {
                Rule::program => {
                    println!("  Adding global scope to function directory"); // #1 Add global scope during program name
                    dusty_context.func_dir.insert("global".to_string(), FunctionInfo::new(0));
                    dusty_context.current_func = "global".to_string();
                }
                Rule::vars => {
                    println!("  Adding variable stack to add to directory after knowing its type"); // #2 Add variable to stack at ID in VARS
                    dusty_context.id_stack.push(pair.as_str().to_string());                  
                }
                Rule::funcs => {
                    println!("  Adding function scope to function directory"); // #3 Add function scope during function name
                    dusty_context.func_dir.insert(pair.as_str().to_string(), FunctionInfo::new(0));
                    dusty_context.current_func = pair.as_str().to_string();
                }
                Rule::id_type_list => {
                    println!("  Adding ID to stack to add to directory after knowing its type"); // #4 Add ID to stack at ID_LIST
                    dusty_context.id_stack.push(pair.as_str().to_string());
                }
                Rule::assign => {
                    // Quad generation
                    if dusty_context.contains_id(pair.as_str()) {
                        dusty_context.quad_data.operand_stack.push([
                            pair.as_str().to_string(),
                            dusty_context.func_dir.get(&dusty_context.current_func).unwrap().get(pair.as_str()).unwrap().to_string()
                        ]);
                    } else {
                        panic!("ERROR: ID \"{}\" not found in current context", pair.as_str());
                    }
                }
                Rule::func_call => {
                    println!("  Generate GOSUB quad to call function"); // #8 Generate GOSUB quad to call function
                    dusty_context.current_call = pair.as_str().to_string();
                    dusty_context.generate_era_quad(pair.as_str());
                }
                _ => {
                    if !dusty_context.contains_id(pair.as_str()) && !dusty_context.id_in_global_scope(pair.as_str()) {
                        panic!("ERROR: ID \"{}\" not found in current context \"{}\", line: {}, col: {}",
                            pair.as_str(),
                            dusty_context.current_func,
                            pair.as_span().start_pos().line_col().0,
                            pair.as_span().start_pos().line_col().1
                        );
                    } else {
                        println!("  ID \"{}\" was found in current context", pair.as_str());
                    }
                }
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::id, Stage::After) => {
            match dusty_context.parent_rules.last().unwrap() {
                Rule::value => {
                    if dusty_context.contains_id(pair.as_str()) {
                        println!("  (#1) Adding ID and type to operand stack in factor"); // #1.1 Add ID and type to operand stack in FACTOR
                        dusty_context.quad_data.operand_stack.push([
                            pair.as_str().to_string(),
                            dusty_context.func_dir.get(&dusty_context.current_func).unwrap().get(pair.as_str()).unwrap().to_string()
                        ]);
                    } else {
                        println!("  (#1) Adding global ID and type to operand stack in factor"); // #1.1 Add ID and type to operand stack in FACTOR
                        dusty_context.quad_data.operand_stack.push([
                            pair.as_str().to_string(),
                            dusty_context.func_dir.get("global").unwrap().get(pair.as_str()).unwrap().to_string()
                        ]);
                    }
                    println!("  Operand stack: {:?}", dusty_context.quad_data.operand_stack);
                }
                _ => {}
            }
            println!("\n");
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process ID --------------------------------------


        // Process Vars ------------------------------------
        (Rule::vars, Stage::Before) => {
            println!("  Sintactic rule VARS found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::vars);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::vars, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::vars, Stage::After) => {
            // println!("func_dir after vars: {:#?}", dusty_context.func_dir);
            // println!("\n");
            dusty_context.parent_rules.pop();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process Vars ------------------------------------


        // Process typeVar ---------------------------------
        (Rule::typeVar, Stage::Before) => {
            println!("  Sintactic rule TYPEVAR found: {:#?}", pair.as_str());
            dusty_context.current_type = pair.as_str().to_string();
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::typeVar, Stage::During) => {
            // println!("ID stack: {:#?}", dusty_context.id_stack);
            // println!("Add all pending ids to the current scope and set them to current type");
            while let Some(id) = dusty_context.id_stack.pop() {
                if dusty_context.contains_id(&id) {
                    panic!("ERROR: id {} already exists in current context (typevar)", id);
                } else {
                    println!("Adding id {} to {} as {}", id, dusty_context.current_func, dusty_context.current_type);
                    dusty_context.func_dir.get_mut(&dusty_context.current_func).unwrap().insert(id.clone(), dusty_context.current_type.clone());
                }
            }
            // println!("func_dir: {:#?}", dusty_context.func_dir);
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::typeVar, Stage::After) => {
            println!("\n");
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process typeVar ---------------------------------


        // Process id_list ---------------------------------
        (Rule::id_list, Stage::Before) => {
            println!("  Sintactic rule ID_LIST found: {:#?}", pair.as_str());
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process id_list ---------------------------------


        // Process Functions -------------------------------
        (Rule::funcs, Stage::Before) => {
            println!("  Sintactic rule FUNCTION found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::funcs);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::funcs, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::funcs, Stage::After) => {
            println!("\n");
            dusty_context.parent_rules.pop();
            dusty_context.current_func = "global".to_string();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process Functions -------------------------------


        // Process parameters ------------------------------
        (Rule::parameters, Stage::Before) => {
            println!("  Sintactic rule PARAMETERS found: {:#?}", pair.as_str());
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process parameters ------------------------------


        // Process func_body -------------------------------
        (Rule::func_body, Stage::Before) => {
            println!("  Sintactic rule FUNC_BODY found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::func_body);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::func_body, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::func_body, Stage::After) => {
            println!("\n");
            dusty_context.parent_rules.pop();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process func_body -------------------------------


        // Process func_call -------------------------------
        (Rule::func_call, Stage::Before) => {
            println!("  Sintactic rule FUNC_CALL found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::func_call);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::func_call, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::func_call, Stage::After) => {
            println!("\n");
            dusty_context.parent_rules.pop();
            dusty_context.quad_data.param_counter = 1;
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process func_call -------------------------------


        // Process id_type_list ----------------------------
        (Rule::id_type_list, Stage::Before) => {
            println!("  Sintactic rule ID_TYPE_LIST found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::id_type_list);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::id_type_list, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::id_type_list, Stage::After) => {
            println!("\n");
            dusty_context.parent_rules.pop();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process id_type_list ----------------------------


        // Process body ------------------------------------
        (Rule::body, Stage::Before) => {
            println!("  Sintactic rule BODY found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::body);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::body, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::body, Stage::After) => {
            println!("\n");
            dusty_context.parent_rules.pop();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process body ------------------------------------


        // Process statement -------------------------------
        (Rule::statement, Stage::Before) => {
            println!("\n");
            println!("  Sintactic rule STATEMENT found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::statement);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::statement, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::statement, Stage::After) => {
            dusty_context.parent_rules.pop();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process statement -------------------------------


        // Process while -----------------------------------
        (Rule::while_loop, Stage::Before) => {
            println!("  Sintactic rule WHILE found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::while_loop);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::while_loop, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::while_loop, Stage::After) => {
            dusty_context.parent_rules.pop();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process while -----------------------------------


        // Process doKeyword -------------------------------
        (Rule::doKeyword, Stage::Before) => {
            println!("  token DO found:");
            println!("  (#?) Generate GOTO quad to start of while loop");
            dusty_context.generate_gotof_quad();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process doKeyword -------------------------------
        

        // Process if --------------------------------------
        (Rule::condition, Stage::Before) => {
            println!("  Sintactic rule CONDITION found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::condition);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::condition, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::condition, Stage::After) => {
            dusty_context.parent_rules.pop();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process if --------------------------------------


        // Process elseKeyword -----------------------------
        (Rule::elseKeyword, Stage::Before) => {
            println!("  token rule ELSE found: {:#?}", pair.as_str());
            dusty_context.fill_jump();
            dusty_context.quad_data.jump_stack.push(dusty_context.quad_data.quad_counter);
            dusty_context.generate_goto_quad();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process elseKeyword -----------------------------


        // Process print -----------------------------------
        (Rule::print, Stage::Before) => {
            println!("  Sintactic rule PRINT found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::print);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::print, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::print, Stage::After) => {
            dusty_context.parent_rules.pop();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process print -----------------------------------


        // Process print_element ---------------------------
        (Rule::print_element, Stage::Before) => {
            println!("  Sintactic rule PRINT_ELEMENT found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::print_element);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::print_element, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::print_element, Stage::After) => {
            dusty_context.parent_rules.pop();
            dusty_context.generate_print_quad();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process print_element ---------------------------


        // Process assignment ------------------------------
        (Rule::assign, Stage::Before) => {
            println!("  Sintactic rule ASSIGNMENT found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::assign);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::assign, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::assign, Stage::After) => {
            dusty_context.parent_rules.pop();
            if dusty_context.top_is_equals() {
                println!("  (#7) Execute #4 with =");
                dusty_context.generate_assign_quad();
            }
            dusty_context.debug_quad_gen();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process assignment ------------------------------


        // Process expression ------------------------------
        (Rule::expression, Stage::Before) => {
            println!("  Sintactic rule EXPRESSION found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::expression);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::expression, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::expression, Stage::After) => {
            dusty_context.parent_rules.pop();
            if dusty_context.top_is_logical_operator() {
                println!("  (#6) Execute #4 with >, <, == or !=");
                dusty_context.generate_full_quad();
            }

            // Parameters for function call
            match dusty_context.parent_rules.last().unwrap() {
                Rule::func_call => {
                    println!("  (#?) Generate PARAM quad for function call");
                    dusty_context.generate_param_quad();
                }
                _ => {}
            }

            dusty_context.debug_quad_gen();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process expression ------------------------------


        // Process exp -------------------------------------
        (Rule::exp, Stage::Before) => {
            println!("  Sintactic rule EXP found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::exp);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::exp, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::exp, Stage::After) => {
            dusty_context.parent_rules.pop();
            if dusty_context.top_is_addition_or_subtraction() {
                println!("  (#4) Execute #4 with + or -");
                dusty_context.generate_full_quad();
            }
            dusty_context.debug_quad_gen();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process exp -------------------------------------


        // Process term ------------------------------------
        (Rule::term, Stage::Before) => {
            println!("  Sintactic rule TERM found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::term);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::term, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::term, Stage::After) => {
            if dusty_context.top_is_multiplication_or_division() {
                println!("  (#5) Execute #4 with * or /");
                dusty_context.generate_full_quad();
            }
            dusty_context.debug_quad_gen();
            dusty_context.parent_rules.pop();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process term ------------------------------------
        

        // Process factor ----------------------------------
        (Rule::factor, Stage::Before) => {
            println!("  Sintactic rule FACTOR found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::factor);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::factor, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::factor, Stage::After) => {
            dusty_context.parent_rules.pop();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process factor ----------------------------------


        // Process value -----------------------------------
        (Rule::value, Stage::Before) => {
            println!("  Sintactic rule VALUE found: {:#?}", pair.as_str());
            dusty_context.parent_rules.push(Rule::value);
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::value, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::value, Stage::After) => {
            dusty_context.parent_rules.pop();
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process value -----------------------------------


        // Process operator --------------------------------
        (Rule::operator, Stage::Before) => {
            println!("  token OPERATOR found: {:#?}", pair.as_str());
            if dusty_context.top_is_multiplication_or_division() {
                println!("  (#10) (Encountered * or / but there is at least 1 that needs to be executed before... Execute #4 with * or /");
                dusty_context.generate_full_quad();
            }
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::operator, Stage::During) => {
            println!("  (#2) Push operator to operator stack");
            dusty_context.quad_data.operator_stack.push(pair.as_str().to_string());
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process operator --------------------------------


        // Process sign ------------------------------------
        (Rule::sign, Stage::Before) => {
            println!("  token SIGN found: {:#?}", pair.as_str());
            if dusty_context.top_is_addition_or_subtraction() {
                println!("  (#11) (Encountered + or - but there is at least 1 that needs to be executed before... Execute #4 with + or -");
                dusty_context.generate_full_quad();
            }
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::sign, Stage::During) => {
            println!("  (#3) Push sign to operator stack");
            dusty_context.quad_data.operator_stack.push(pair.as_str().to_string());
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process sign ------------------------------------


        // Process logical_operator -------------------------
        (Rule::comparator, Stage::Before) => {
            println!("  token COMPARATOR found: {:#?}", pair.as_str());
            println!("  (#4) Push logical operator to operator stack");
            dusty_context.quad_data.operator_stack.push(pair.as_str().to_string());
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process logical_operator -------------------------

        
        // Process equals ----------------------------------
        (Rule::equals, Stage::Before) => {
            println!("  token EQUALS found: {:#?}", pair.as_str());
            println!("  (#5) Push equals to operator stack");
            dusty_context.quad_data.operator_stack.push(pair.as_str().to_string());
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process equals ----------------------------------


        // Process open_parenthesis -------------------------
        (Rule::openP, Stage::Before) => {
            println!("  token OPEN_PARENTHESIS found: {:#?}", pair.as_str());
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::openP, Stage::During) => {
            match dusty_context.parent_rules.last().unwrap() {
                Rule::factor => {
                    println!("  (#6) Push open parenthesis to operator stack");
                    dusty_context.quad_data.operator_stack.push(pair.as_str().to_string());
                }
                Rule::while_loop => {
                    println!("  (#?) Push to jump stack");
                    dusty_context.quad_data.jump_stack.push(dusty_context.quad_data.quad_counter);
                }
                _ => {}
            }
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process open_parenthesis -------------------------


        // Process close_parenthesis ------------------------
        (Rule::closeP, Stage::Before) => {
            println!("  token CLOSE_PARENTHESIS found: {:#?}", pair.as_str());
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::closeP, Stage::During) => {
            match dusty_context.parent_rules.last().unwrap() {
                Rule::factor => {
                    println!("  (#9) pop stack");
                    dusty_context.quad_data.operator_stack.pop();
                    println!("  {:#?}", dusty_context.quad_data.operator_stack);
                }
                Rule::condition => {
                    println!("  (#12) Generate incomplete GOTOF quad and push to jump stack");
                    dusty_context.generate_gotof_quad();
                }
                Rule::func_call => {
                    println!("  (#?) Generate GOSUB quad to call function");
                    dusty_context.generate_gosub_quad();
                }
                _ => {}
            }
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process close_parenthesis ------------------------


        // Process cte -------------------------------------
        (Rule::cte, Stage::Before) => {
            println!("  Sintactic rule CTE found: {:#?}", pair.as_str());
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::cte, Stage::During) => {
            let inner_pairs = pair.clone().into_inner();
            for inner_pair in inner_pairs {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::cte, Stage::After) => {
            // dusty_context.parent_rule = Rule::value;
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process cte -------------------------------------

        
        // Process cte_int ---------------------------------
        (Rule::cte_int, Stage::Before) => {
            println!("  token CTE found: {:#?}", pair.as_str());
            println!("  (#1) Adding CTE to operand stack in factor");
            dusty_context.quad_data.operand_stack.push([
                pair.as_str().to_string(),
                "int".to_string()
            ]);
            println!("  Operand stack: {:?}", dusty_context.quad_data.operand_stack);
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process cte_int ---------------------------------


        // Process cte_float -------------------------------
        (Rule::cte_float, Stage::Before) => {
            println!("  token CTE_FLOAT found: {:#?}", pair.as_str());
            println!("  (#1) Adding CTE_FLOAT to operand stack in factor");
            dusty_context.quad_data.operand_stack.push([
                pair.as_str().to_string(),
                "float".to_string()
            ]);
            println!("  Operand stack: {:?}", dusty_context.quad_data.operand_stack);
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process cte_float -------------------------------


        // Process delimiter -------------------------------
        (Rule::delimiter, Stage::Before) => {
            println!("  token DELIMITER found: {:#?}", pair.as_str());
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::delimiter, Stage::During) => {
            match dusty_context.parent_rules.last().unwrap() {
                Rule::condition => {
                    println!("  (#13) Complete GOTOF quad");
                    dusty_context.fill_jump();
                }
                Rule::while_loop => {
                    println!("  (#?) Generate GOTO quad to start of while loop");
                    dusty_context.fill_while_start();
                    dusty_context.generate_gotow_quad();
                    dusty_context.fill_while_end();
                }
                Rule::funcs => {
                    println!("  (#?) Generate ENDFUNC to indicate functions end");
                    dusty_context.generate_endfunc_quad();
                }
                Rule::program => {
                    println!("  (#?) Generate first GOTO quad to start of program");
                    dusty_context.generate_goto_quad();
                }
                _ => {}
            }
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process delimiter -------------------------------


        // Process endKeyword ------------------------------
        (Rule::endKeyword, Stage::Before) => {
            println!("  token END found");
            println!("  Generating END quad");
            dusty_context.generate_end_quad();
        }
        // Process endKeyword ------------------------------


        // Anything else (move on to the next pair)
        _ => {
            println!("...");
        }
    }
}

fn main() {
    // File path to read
    let path = "C:/Users/wetpe/Documents/Tec8/compiladores/ducky-language-rust/src/tests/app6.dusty";
    let patito_file = fs::read_to_string(&path).expect("error reading file");

    let mut dusty_context = DustyContext::new();

    match DustyParser::parse(Rule::program, &patito_file) {
        Ok(pairs) => {
            // println!("{:#?}", pairs.into_iter().next().unwrap().into_inner());
            // dusty_context.syntax_flow = context_pairs;
            // Go through the AST and trigger actions in certain parts
            for pair in pairs.into_iter().next().unwrap().into_inner() {
                process_pair(
                    pair,
                    Stage::Before,
                    &mut dusty_context
                );
                // println!("{:#?}", pair.as_rule());
            }
        }
        Err(e) => {
            println!("Error: {:#?}", e);
        }
    }

    println!("{:#?}", dusty_context.func_dir);
    println!(" ---------- QUADRUPLES AS NAME ---------- ");
    dusty_context.print_quadruples_as_name();
    println!(" ---------- QUADRUPLES AS MEMORY ---------- ");
    dusty_context.print_quadruples_as_memmory();
}
