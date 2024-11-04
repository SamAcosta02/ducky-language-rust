use std::{collections::{HashMap, HashSet, VecDeque}, fs};

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "patito.pest"]
pub struct PatitoParser;

fn generate_quads(
    semantic_cube: &[[[&str; 8]; 2]; 2],
    current_expr: &mut VecDeque<[&str; 4]>,
    poper: &mut Vec<char>,
    op: &mut Vec<[String; 2]>,
    func_dir: &HashMap<String, HashMap<String, String>>,
    current_func: &String,
    pair: &pest::iterators::Pair<Rule>
) {
    for inner_pair in pair.into_inner() {
        generate_quads(semantic_cube, current_expr, poper, op, func_dir, current_func, pair);
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
    current_expr: &mut VecDeque<[&str; 4]>
) {

    // visualize current rule
    let current_rule = pair.as_rule();
    println!("{:#?}", current_rule);
    println!("{:#?}", pair.as_str());

    // Push the current rule to the context stack, if it is a main rule
    if main_rules.contains(&current_rule) {
        println!("Rule: ++{:?}++ IS a main rule", current_rule);
        context_stack.push(current_rule);
    } else {
        println!("Rule: **{:?}** not a main rule", current_rule);
    }

    // match the current rule depending on the context
    match pair.as_rule() {
        Rule::programKeyword => {
            println!("ACTION: Create dir_func \n"); //1
            func_dir.insert(String::from("global"), HashMap::new());
            *current_func = String::from("global");
            println!("{:#?}", func_dir);
        }
        Rule::id => {
            if let Some(&parent_rule) = context_stack.iter().rev().next() {
                // Check in what context was id matched in
                match parent_rule {
                    Rule::program => {
                        println!("ACTION: Add id-name and type program to dir_func \n"); //2
                    },
                    Rule::vars => {
                        println!("ACTION: Search for id-name in current VarTable"); //5
                        // Look for the id in the current function
                        // If it is panic as it is already declared
                        if func_dir.get_mut(current_func).unwrap().contains_key(pair.as_str()) {
                            panic!("ERROR: id {} already exists in current context", pair.as_str());
                        // otherwise insert the id to the context-ids stack (to later add them to the dir_func)
                        } else {
                            println!("ACTION: Add id-name to context-ids to later add them to dir_func \n");
                            context_ids.push(pair.as_str().to_string());
                        }
                    },
                    Rule::funcs => {
                        // Update the current function id and add it to the func_dir if it doesn't already exist
                        println!("ACTION: Add id-name and type func to func_dir \n"); //2
                        *current_func = pair.as_str().to_string();

                        if !func_dir.contains_key(&current_func.to_string()) {
                            func_dir.insert(current_func.to_string(), HashMap::new());
                            println!("Function '{}' added to func_dir.", current_func);
                        } else {
                            panic!("ERROR: Function '{}' already exists.", current_func);
                        }
                        println!("{:#?}", func_dir);
                    },
                    Rule::parameters => {
                        // Add the parameter ID to the contex_ids stack
                        context_ids.push(pair.as_str().to_string());
                    },
                    Rule::statement => {
                        if func_dir.get_mut(current_func).unwrap().contains_key(pair.as_str()) {
                            println!("ACTION: Search for id-name in current VarTable");
                        } else {
                            panic!("Error: id '{}' not found in current context", pair.as_str());
                        }
                    }
                    _ => println!("id in another context."),
                }
            }
        }
        Rule::varsKeyword => {
            println!("ACTION: if current Func doesn't have a VarTable then create Var Table and link it to current func \n"); //3
        }
        Rule::typeVar => {
            println!("ACTION: update current-type to {} \n", pair.as_str().to_string()); //4
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
            println!("ACTION: Changing back to the global context \n");
            *current_func = "global".to_string();
        },
        Rule::statement => {
            let mut poper: Vec<char> = Vec::new();
            let mut op: Vec<[String; 2]> = Vec::new();
            let quads = generate_quads(semantic_cube, current_expr, &mut poper, &mut op, &func_dir, current_func, &pair);
            println!("{:#?}", quads);
        }
        _ => {
            println!("... \n");
        }
    }
    
    // Check current stack status
    println!("CONTEXT IDs: {:?}", context_ids);
    println!("CONTEXT STACK {:?}", &context_stack);
    println!("CURRENT FUNC {:?}", current_func);
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
            current_expr
        );
    }

    // Pop the current rule from the context stack, if it is a main rule (once the recursion is done)
    if main_rules.contains(&current_rule) {
        println!("Rule: --{:?}-- Remove context main Rule as we are done", current_rule);
        context_stack.pop();
    }
}

fn main() {
    let path = "C:/Users/wetpe/OneDrive/Documents/_Manual/TEC 8/ducky-language-rust/src/tests/app2.dusty";
    let patito_file = fs::read_to_string(&path).expect("error reading file");

    // Create semantic cube that will tell us what type of data will be returned when performing an operation
    let semantic_cube = [
        [ // Left operand is int (0)
            ["int", "float", "int", "float", "int", "int", "int", "int"],  // Right operand int (0) for +, -, *, /, >, <, ==, !=
            ["float", "float", "float", "float", "int", "int", "int", "int"], // Right operand float (1) for +, -, *, /, >, <, ==, !=
        ],
        [ // Left operand is float (1)
            ["float", "float", "float", "float", "int", "int", "int", "int"], // Right operand int (0) for +, -, *, /, >, <, ==, !=
            ["float", "float", "float", "float", "int", "int", "int", "int"], // Right operand float (1) for +, -, *, /, >, <, ==, !=
        ],
    ];

    // Create a directory to store the functions and their types
    let mut func_dir: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current_type = String::new();
    let mut context_stack: Vec<Rule> = Vec::new();
    let mut context_ids: Vec<String> = Vec::new();
    let mut current_func: String = String::new();
    let mut current_expr: VecDeque<[&str; 4]> = VecDeque::new();
    let main_rules: HashSet<Rule> = [Rule::program, Rule::vars, Rule::funcs, Rule::funcs, Rule::parameters, Rule::statement]
        .iter()
        .cloned()
        .collect();

    match PatitoParser::parse(Rule::program, &patito_file) {
        Ok(pairs) => {
            for pair in pairs {
                // println!("{:#?}", pair.into_inner());
                visualize_pair(
                    pair,
                    &mut context_stack,
                    &main_rules,
                    &mut current_type,
                    &mut func_dir,
                    &semantic_cube,
                    &mut context_ids,
                    &mut current_func,
                    &mut current_expr
                );
            }
            println!("{:#?}", func_dir);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
