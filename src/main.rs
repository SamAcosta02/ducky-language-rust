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
struct DustyContext {
    func_dir: HashMap<String, HashMap<String, String>>, // Function-variable scope directory
    parent_rule: Rule,
    current_type: String,
    current_func: String,
    id_stack: Vec<String>,
    syntax_flow: VecDeque<Rule>
}

impl DustyContext {
    fn new(syntax_tree: VecDeque<Rule>) -> Self {
        DustyContext {
            func_dir: HashMap::new(),
            id_stack: Vec::new(),
            parent_rule: Rule::program,
            current_func: String::new(),
            current_type: String::new(),
            syntax_flow: syntax_tree
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
    println!("Processing rule: {:#?} in stage {:#?}, parent rule: {:#?}, line: {:#?}, col: {:#?}",
        pair.as_rule(), stage, dusty_context.parent_rule,
        pair.as_span().start_pos().line_col().0,
        pair.as_span().start_pos().line_col().1
    );

    match (pair.as_rule(), &stage) {
        // Process ID --------------------------------------
        (Rule::id, Stage::Before) => {
            println!("Token ID found: {:#?}", pair.as_str());
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::id, Stage::During) => {
            match dusty_context.parent_rule {
                Rule::program => {
                    println!("Adding global scope to function directory"); // #1 Add global scope during program name
                    dusty_context.func_dir.insert("global".to_string(), HashMap::new());
                    dusty_context.current_func = "global".to_string();
                }
                Rule::vars => {
                    println!("Adding variable stack to add to directory after knowing its type"); // #2 Add variable to stack at ID in VARS
                    dusty_context.id_stack.push(pair.as_str().to_string());                  
                }
                _ => {}
            }
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::id, Stage::After) => {
            println!("\n");
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process ID --------------------------------------

        // Process Vars ------------------------------------
        (Rule::vars, Stage::Before) => {
            println!("Sintactic rule VARS found: {:#?}", pair.as_str());
            dusty_context.parent_rule = Rule::vars;
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
            println!("func_dir after vars: {:#?}", dusty_context.func_dir);
            println!("\n");
            dusty_context.parent_rule = Rule::program;
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process Vars ------------------------------------

        // Process typeVar ---------------------------------
        (Rule::typeVar, Stage::Before) => {
            println!("Sintactic rule TYPEVAR found: {:#?}", pair.as_str());
            dusty_context.current_type = pair.as_str().to_string();
            process_pair(pair, Stage::During, dusty_context);
        }
        (Rule::typeVar, Stage::During) => {
            println!("ID stack: {:#?}", dusty_context.id_stack);
            println!("Add all pending ids to the current scope and set them to current type");
            while let Some(id) = dusty_context.id_stack.pop() {
                if dusty_context.contains_id(&id) {
                    panic!("ERROR: id {} already exists in current context (typevar)", id);
                } else {
                    println!("Adding id {} to {} as {}", id, dusty_context.current_func, dusty_context.current_type);
                    dusty_context.func_dir.get_mut(&dusty_context.current_func).unwrap().insert(id.clone(), dusty_context.current_type.clone());
                }
            }
            println!("func_dir: {:#?}", dusty_context.func_dir);
            process_pair(pair, Stage::After, dusty_context);
        }
        (Rule::typeVar, Stage::After) => {
            println!("\n");
            process_pair(pair, Stage::Finished, dusty_context);
        }
        // Process typeVar ---------------------------------

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

    let mut dusty_context = DustyContext::new(VecDeque::new());

    match DustyParser::parse(Rule::program, &patito_file) {
        Ok(pairs) => {
            let test = pairs.collect::<Vec<_>>();
            println!("context_pairs: {:#?}", test);
            // dusty_context.syntax_flow = context_pairs;
            // Go through the AST and trigger actions in certain parts
            // for pair in pairs {
            //     process_pair(
            //         pair,
            //         Stage::Before,
            //         &mut dusty_context
            //     );
            // }
        }
        Err(e) => {
            println!("Error: {:#?}", e);
        }
    }

    println!("{:#?}", dusty_context.func_dir);
}
