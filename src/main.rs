use std::{collections::{HashMap, HashSet, VecDeque}, fs, result};

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "patito.pest"]
pub struct PatitoParser;

#[derive(Debug)]
struct QuadStack {
    o: Vec<[String; 2]>,
    oper: Vec<String>,
    jumps: Vec<i32>
}

impl QuadStack {
    fn new() -> Self {
        QuadStack {
            o: Vec::new(),
            oper: Vec::new(),
            jumps: Vec::new()
        }
    }
}

fn consulut_id_type(func_dir: &mut HashMap<String, HashMap<String, String>>, current_func: &String, pair: &pest::iterators::Pair<Rule>) -> String {
    // Consult the inner value
    if let Some(inner_map) = func_dir.get(current_func) {
        if let Some(value) = inner_map.get(pair.as_str()) {
            println!("ID type: {:?}", value);
            value.to_string()
        } else {
            "ERROR".to_string()
        }
    } else {
        "ERROR".to_string()
    }
}

fn get_type_int(type_str: &str) -> usize {
    match type_str {
        "int" => 0,
        "float" => 1,
        "+" => 0,
        "-" => 1,
        "*" => 2,
        "/" => 3,
        ">" => 4,
        "<" => 5,
        "==" => 6,
        "!=" => 7,
        _ => 10
    }
}

fn visualize_pair(
    pair: pest::iterators::Pair<Rule>,
    context_stack: &mut Vec<Rule>,
    main_rules: &HashSet<Rule>,
    current_type: &mut String,
    func_dir: &mut HashMap<String, HashMap<String, String>>,
    semantic_cube: &[[[&str; 8]; 2]; 2],
    context_ids: &mut Vec<String>,
    current_func: &mut String,
    quadruples: &mut VecDeque<[String; 4]>,
    pending_statements: &mut Vec<String>,
    quad_stack: &mut QuadStack
) {

    // visualize current rule
    let current_rule = pair.as_rule();
    println!("Rule: {:#?}", current_rule);
    // println!("{:#?}", pair.as_str());

    // Push the current rule to the context stack, if it is a main rule
    if main_rules.contains(&current_rule) {
        // println!("Rule: ++{:?}++ IS a main rule", current_rule);
        context_stack.push(current_rule);
    } else {
        // println!("Rule: **{:?}** not a main rule", current_rule);
    }

    // match the current rule depending on the context
    match pair.as_rule() {
        Rule::programKeyword => {
            // println!("ACTION: Create dir_func \n"); //1
            func_dir.insert(String::from("global"), HashMap::new());
            *current_func = String::from("global");
            println!("{:#?}", func_dir);
        }
        Rule::id => {
            if let Some(&parent_rule) = context_stack.iter().rev().next() {
                // Check in what context was id matched in
                match parent_rule {
                    Rule::program => {
                        // println!("ACTION: Add id-name and type program to dir_func \n"); //2
                    },
                    Rule::vars => {
                        // println!("ACTION: Search for id-name in current VarTable"); //5
                        // Look for the id in the current function
                        // If it is panic as it is already declared
                        if func_dir.get_mut(current_func).unwrap().contains_key(pair.as_str()) {
                            panic!("ERROR: id {} already exists in current context", pair.as_str());
                        // otherwise insert the id to the context-ids stack (to later add them to the dir_func)
                        } else {
                            // println!("ACTION: Add id-name to context-ids to later add them to dir_func \n");
                            context_ids.push(pair.as_str().to_string());
                        }
                    },
                    Rule::funcs => {
                        // Update the current function id and add it to the func_dir if it doesn't already exist
                        // println!("ACTION: Add id-name and type func to func_dir \n"); //2
                        *current_func = pair.as_str().to_string();

                        if !func_dir.contains_key(&current_func.to_string()) {
                            func_dir.insert(current_func.to_string(), HashMap::new());
                            // println!("Function '{}' added to func_dir.", current_func);
                        } else {
                            panic!("ERROR: Function '{}' already exists.", current_func);
                        }
                        println!("{:#?}", func_dir);
                    },
                    Rule::parameters => {
                        // Add the parameter ID to the contex_ids stack
                        context_ids.push(pair.as_str().to_string());
                    },
                    Rule::factor => {
                        // Add the factor ID to the quad operand stack
                        println!(" - - expression action #1");
                        quad_stack.o.push([pair.as_str().to_string(), consulut_id_type(func_dir, current_func, &pair)]);
                    }
                    _ => {
                        // if the id is not found in any of these contexts, check if the id exists in the current context
                        println!("Rule is: {:?}", parent_rule);
                        if func_dir.get_mut(current_func).unwrap().contains_key(pair.as_str()) {
                            // println!("ACTION: Search for id-name in current VarTable");
                        } else {
                            panic!("Error: id '{}' not found in current context", pair.as_str());
                        }
                    },
                }
            }
        }
        Rule::varsKeyword => {
            // println!("ACTION: if current Func doesn't have a VarTable then create Var Table and link it to current func \n"); //3
        }
        Rule::typeVar => {
            // println!("ACTION: update current-type to {} \n", pair.as_str().to_string()); //4
            // Get the currenty type
            *current_type = pair.as_str().to_string();

            // Assign all variables of this type, checking if they already exist
            while let Some(id) = context_ids.pop() {
                if let Some(context) = func_dir.get_mut(current_func) {
                    if context.contains_key(&id) {
                        panic!("ERROR: id '{}' already exists in the current function context '{}'", id, current_func);
                    } else {
                        context.insert(id, current_type.to_string());
                    }
                }
            }
            println!("{:#?}", func_dir);
        },
        Rule::beginKeyword => {
            // println!("ACTION: Changing back to the global context \n");
            *current_func = "global".to_string();
        },
        Rule::operator => {
            println!(" - - expression action #2");
            quad_stack.oper.push(pair.as_str().to_string());
        }
        Rule::sign => {
            println!(" - - expression action #3");
            quad_stack.oper.push(pair.as_str().to_string());
        }
        Rule::term => {
            println!(" - - expression action #4");
            // println!("qud stack: {:?}", &quad_stack);
            // let right_operand = quad_stack.o.pop().unwrap();
            // println!("- - - - right operand: {:?}", &right_operand);
            // let left_operand = quad_stack.o.pop().unwrap();
            // println!("- - - - right operand: {:?}", &left_operand);
            // let operator = quad_stack.oper.pop().unwrap();
            // println!("- - - - operator: {:?}", &operator);
            // let result_type = semantic_cube[get_type_int(&left_operand[1])][get_type_int(&right_operand[1])][get_type_int(&operator)];
            // println!("- - - - result type: {:?}", &result_type);
            // if result_type != "err" {
            //     quadruples.push_back([right_operand[0].clone(), left_operand[0].clone(), operator.clone(), "_".to_string()]);
            // }
        }
        _ => {
            println!("... \n");
        }
    }
    
    // Check current stack status
    // println!("CONTEXT IDs: {:?}", context_ids);
    println!("CONTEXT STACK {:?}", &context_stack);
    // println!("CURRENT FUNC {:?}", current_func);
    // println!("PENDING STATEMENTS {:?}", &pending_statements);
    println!("QUAD STACK {:?}", &quad_stack);
    println!("QUADRUPLES {:?}", &quadruples);
    for inner_pair in pair.into_inner() {
        visualize_pair(
            inner_pair,
            context_stack,
            main_rules,
            current_type,
            func_dir,
            semantic_cube,
            context_ids,
            current_func,
            quadruples,
            pending_statements,
            quad_stack
        );
    }
    
    // Pop the current rule from the context stack, if it is a main rule (once the recursion is done)
    if main_rules.contains(&current_rule) {
        println!("Rule: --{:?}-- Remove context main Rule as we are done", current_rule);
        context_stack.pop();
    }
}

fn main() {
    // File path to read
    let path = "C:/Users/wetpe/Documents/Tec8/compiladores/ducky-language-rust/src/tests/app2.dusty";
    let patito_file = fs::read_to_string(&path).expect("error reading file");

    // Create semantic cube that will tell us what type of data will be returned when performing an operation
    let semantic_cube = [
        [ // Left operand is int (0)
            ["int", "int", "int", "float", "int", "int", "int", "int"],  // Right operand int (0) for +, -, *, /, >, <, ==, !=
            ["float", "float", "float", "float", "err", "err", "err", "err"], // Right operand float (1) for +, -, *, /, >, <, ==, !=
        ],
        [ // Left operand is float (1)
            ["float", "float", "float", "float", "err", "err", "err", "err"], // Right operand int (0) for +, -, *, /, >, <, ==, !=
            ["float", "float", "float", "float", "int", "int", "int", "int"], // Right operand float (1) for +, -, *, /, >, <, ==, !=
        ],
    ];

    // Check what variables are declared in what scope
    let mut func_dir: HashMap<String, HashMap<String, String>> = HashMap::new();
    // Check the current type of the variable, to assign it once it/they are declared
    let mut current_type = String::new();
    // Check the current context of the program (main rules)
    let mut context_stack: Vec<Rule> = Vec::new();
    // Check what and how many ids to assign a certain type to once a type is declared
    let mut context_ids: Vec<String> = Vec::new();
    // Check what scope we are currently in (check for ids in this scope) 
    let mut current_func: String = String::new();
    // TBD
    let mut pending_statements: Vec<String> = Vec::new();
    // TBD
    let mut quad_stack: QuadStack = QuadStack::new();
    // What main rules to consider to trigger specific actions
    let main_rules: HashSet<Rule> = [Rule::program, Rule::vars, Rule::funcs, Rule::funcs, Rule::parameters, Rule::factor, Rule::expression]
        .iter()
        .cloned()
        .collect();

    let mut quadruples: VecDeque<[String; 4]> = VecDeque::new();

    match PatitoParser::parse(Rule::program, &patito_file) {
        Ok(pairs) => {
            // Go through the AST and trigger actions in certain parts
            for pair in pairs {
                visualize_pair(
                    pair,
                    &mut context_stack,
                    &main_rules,
                    &mut current_type,
                    &mut func_dir,
                    &semantic_cube,
                    &mut context_ids,
                    &mut current_func,
                    &mut quadruples,
                    &mut pending_statements,
                    &mut quad_stack
                );
            }
            println!("{:#?}", func_dir);
        }
        Err(e) => {
            println!("Error: {:#?}", e);
        }
    }
}
