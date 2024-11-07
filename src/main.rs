use std::{collections::{HashMap, HashSet}, fs};

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "dusty.pest"]
pub struct DustyParser;

#[derive(Debug)]
enum Stage {
    Before,
    During,
    After
}

#[derive(Debug)]
struct DustyContext {
    func_dir: HashMap<String, HashMap<String, String>>, // Function-variable scope directory
    current_func: String, // Current function name (knowing the current scope)
    id_rules: HashSet<Rule>, // Main rules that have actions triggered by ids
    id_context: Vec<Rule>, // Know what is the parent rule
    id_stack: Vec<String> // Stack of ids to add type (vars)
}

impl DustyContext {
    fn new() -> Self {
        DustyContext {
            func_dir: HashMap::new(),
            current_func: String::new(),
            id_rules: [Rule::program, Rule::vars, Rule::funcs, Rule::funcs, Rule::parameters]
                .iter()
                .cloned()
                .collect(),
            id_context: Vec::new(),
            id_stack: Vec::new()
        }
    }

    fn contains_id(&self, id: &str) -> bool {
        self.func_dir.get(&self.current_func).unwrap().contains_key(id)
    }
}

fn process_pair(
    pair: pest::iterators::Pair<Rule>,
    stage: Stage,
    dusty_context: &mut DustyContext
) {
    // visualize current rule
    let current_rule = pair.as_rule();
    println!("{:#?}", current_rule);
    println!("{:#?}", pair.as_str());
    println!("{:#?}", dusty_context.id_context);

    // Push the current rule to the context stack, if it is a main rule
    if dusty_context.id_rules.contains(&current_rule) && dusty_context.id_context.last() != Some(&current_rule) {
        // println!("Rule: ++{:?}++ IS a main rule", current_rule);
        dusty_context.id_context.push(current_rule);
    } else {
        // println!("Rule: **{:?}** not a main rule", current_rule);
    }

    match (pair.as_rule(), &stage) {
        // Process program keyword
        (Rule::program, Stage::Before) => {
            println!("Adding the global function to the directory...");
            dusty_context.current_func = "global".to_string();
            process_pair(pair.clone(), Stage::During, dusty_context);
        }

        // Process identifiers
        (Rule::id, Stage::Before) => {
            println!("ID before...");
            process_pair(pair.clone(), Stage::During, dusty_context);
        }
        (Rule::id, Stage::During) => {
            if let Some(&parent_rule) = dusty_context.id_context.iter().rev().next() {
                match parent_rule {
                    Rule::program => {
                        println!("ID action during (adding the global function to the directory)...");
                        dusty_context.func_dir.insert(dusty_context.current_func.clone(), HashMap::new());
                    }
                    Rule::vars => {
                        println!("ID action during (adding the variable to the function)...");
                        // Look for the id in the current function
                        // If it is panic as it is already declared
                        if dusty_context.contains_id(pair.as_str()) {
                            panic!("ERROR: id {} already exists in current context (vars)", pair.as_str());
                        // otherwise insert the id to the context-ids stack (to later add them to the dir_func)
                        } else {
                            println!("ACTION: Add id-name to context-ids to later add them to dir_func \n");
                            dusty_context.id_stack.push(pair.as_str().to_string());
                        }
                    }
                    _ => {
                        println!("ID during...");
                    }
                }
            }
            process_pair(pair.clone(), Stage::After, dusty_context);
        }

        // Process typeVar (After a list of ids, or id)
        (Rule::typeVar, Stage::Before) => {
            println!("TypeVar before...");
            process_pair(pair.clone(), Stage::During, dusty_context);
        }
        (Rule::typeVar, Stage::During) => {
            println!("Add all pending ids to the current scope and set them to current type");
            while let Some(id) = dusty_context.id_stack.pop() {
                if dusty_context.contains_id(&id) {
                    panic!("ERROR: id {} already exists in current context (typevar)", id);
                } else {
                    dusty_context.func_dir.get_mut(&dusty_context.current_func).unwrap().insert(id, pair.as_str().to_string());
                }
            }
        }

        // Process vars
        (Rule::vars, Stage::Before) => {
            println!("Vars before...");
            process_pair(pair.clone(), Stage::During, dusty_context);
        }

        // Process begin keyword
        (Rule::beginKeyword, Stage::Before) => {
            println!("Begin before...");
            dusty_context.current_func = "global".to_string();
            process_pair(pair.clone(), Stage::During, dusty_context);
        }

        // Anything else (move on to the next pair)
        _ => {
            println!("...");
            for inner_pair in pair.into_inner() {
                process_pair(
                    inner_pair,
                    Stage::Before,
                    dusty_context
                );
            }
        }
    }
}

fn main() {
    // File path to read
    let path = "C:/Users/wetpe/OneDrive/Documents/_Manual/TEC 8/ducky-language-rust/src/tests/app2.dusty";
    let patito_file = fs::read_to_string(&path).expect("error reading file");

    let mut dusty_context = DustyContext::new();

    match DustyParser::parse(Rule::program, &patito_file) {
        Ok(pairs) => {
            // Go through the AST and trigger actions in certain parts
            for pair in pairs {
                process_pair(
                    pair,
                    Stage::Before,
                    &mut dusty_context
                );
            }
        }
        Err(e) => {
            println!("Error: {:#?}", e);
        }
    }

    println!("{:#?}", dusty_context.func_dir);
}
