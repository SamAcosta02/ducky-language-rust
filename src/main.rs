use std::{collections::{HashMap, VecDeque}, fs};

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "dusty.pest"]
pub struct DustyParser;

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
    temp_counter: usize,
    semantic_cube: SemanticCube,
}

impl QuadData {
    fn new() -> Self {
        QuadData {
            operator_stack: Vec::new(),
            operand_stack: Vec::new(),
            jump_stack: Vec::new(),
            quad_counter: 1,
            temp_counter: 1,
            semantic_cube: SemanticCube::new(),
        }
    }
}

#[derive(Debug)]
struct DustyContext {
    func_dir: HashMap<String, HashMap<String, String>>, // Function-variable scope directory
    parent_rules: Vec<Rule>,
    current_type: String,
    current_func: String,
    id_stack: Vec<String>,
    quad_data: QuadData,
    quadruples: VecDeque<[String; 4]>,
}

impl DustyContext {
    fn new() -> Self {
        DustyContext {
            func_dir: HashMap::new(),
            id_stack: Vec::new(),
            parent_rules: vec![Rule::program],
            current_func: String::new(),
            current_type: String::new(),
            quad_data: QuadData::new(),
            quadruples: VecDeque::new(),
        }
    }

    fn contains_id(&self, id: &str) -> bool {
        self.func_dir.get(&self.current_func).unwrap().contains_key(id)
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
            operator,
            left_operand[0].clone(),
            right_operand[0].clone(),
            result.clone()
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
            operator,
            right_operand[0].clone(),
            "_".to_string(),
            left_operand[0].clone()
        ]);
        self.quad_data.quad_counter += 1;
    }

    fn generate_print_quad(&mut self) {
        let element = self.quad_data.operand_stack.last();
        if element.is_none() {
            panic!("ERROR: Missing element to print");
        }
        self.quadruples.push_back([
            "print".to_string(),
            "_".to_string(),
            "_".to_string(),
            element.unwrap()[0].clone()
        ]);
        self.quad_data.quad_counter += 1;
    }

    fn generate_gotof_quad(&mut self) {
        self.quadruples.push_back([
            "gotof".to_string(),
            format!("t{}", self.quad_data.temp_counter - 1),
            "_".to_string(),
            "_".to_string()
        ]);
        self.quad_data.jump_stack.push(self.quad_data.quad_counter);
        self.quad_data.quad_counter += 1;
    }

    fn print_quadruples(&self) {
        for quad in &self.quadruples {
            println!("{:?}", quad);
        }
    }

    fn debug_quad_gen(&self) {
        println!("  Operator stack: {:?}", self.quad_data.operator_stack);
        println!("  Operand stack: {:?}", self.quad_data.operand_stack);
        println!("  Jump stack: {:?}", self.quad_data.jump_stack);
        println!("  temp counter: {}, quad_counter: {}", self.quad_data.temp_counter, self.quad_data.quad_counter);
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
        // Process ID --------------------------------------
        (Rule::id, Stage::Before) => {
            println!("  Token ID found: {:#?}", pair.as_str());
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::id, Stage::During) => {
            match dusty_context.parent_rules.last().unwrap() {
                Rule::program => {
                    println!("  Adding global scope to function directory"); // #1 Add global scope during program name
                    dusty_context.func_dir.insert("global".to_string(), HashMap::new());
                    dusty_context.current_func = "global".to_string();
                }
                Rule::vars => {
                    println!("  Adding variable stack to add to directory after knowing its type"); // #2 Add variable to stack at ID in VARS
                    dusty_context.id_stack.push(pair.as_str().to_string());                  
                }
                Rule::funcs => {
                    println!("  Adding function scope to function directory"); // #3 Add function scope during function name
                    dusty_context.func_dir.insert(pair.as_str().to_string(), HashMap::new());
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
                _ => {
                    if !dusty_context.contains_id(pair.as_str()) {
                        panic!("ERROR: ID \"{}\" not found in current context", pair.as_str());
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
                    println!("  (#1) Adding ID and type to operand stack in factor"); // #1.1 Add ID and type to operand stack in FACTOR
                    dusty_context.quad_data.operand_stack.push([
                        pair.as_str().to_string(),
                        dusty_context.func_dir.get(&dusty_context.current_func).unwrap().get(pair.as_str()).unwrap().to_string()
                        ]);
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
                    // println!("Adding id {} to {} as {}", id, dusty_context.current_func, dusty_context.current_type);
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
                }
                _ => {}
            }
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process delimiter -------------------------------


        // Anything else (move on to the next pair)
        _ => {
            println!("...");
        }
    }
}

fn main() {
    // File path to read
    let path = "C:/Users/wetpe/OneDrive/Documents/_Manual/TEC 8/ducky-language-rust/src/tests/app4.dusty";
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
    dusty_context.print_quadruples();
}
