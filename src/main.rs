use std::{collections::HashSet, fs};

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "patito.pest"]
pub struct PatitoParser;

fn visualize_pair(
    pair: pest::iterators::Pair<Rule>,
    context_stack: &mut Vec<Rule>,
    main_rules: &HashSet<Rule>
) {
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

    match pair.as_rule() {
        Rule::programKeyword => {
            println!("ACTION: Create dir_func \n");
        }
        Rule::id => {
            if let Some(&parent_rule) = context_stack.iter().rev().next() {
                match parent_rule {
                    Rule::program => println!("ACTION: Add id-name and type program to dir_func \n"),
                    Rule::vars => println!("ACTION: Search for id-name in current VarTable \n"),
                    _ => println!("id in another context."),
                }
            }
        }
        Rule::varsKeyword => {
            println!("ACTION: if current Func doesn't have a VarTable then create Var Table and link it to current func \n");
        }
        _ => {
            println!("... \n");
        }
    }
    println!("CONTEXT STACK {:?}", &context_stack);
    for inner_pair in pair.into_inner() {
        visualize_pair(inner_pair, context_stack, main_rules);
    }
}

fn main() {
    let path = "C:/Users/wetpe/OneDrive/Documents/_Manual/TEC 8/ducky-language-rust/src/tests/app1.dky";
    let patito_file = fs::read_to_string(&path).expect("error reading file");

    // let mut dir_func: HashMap<String, String> = HashMap::new();
    let mut context_stack: Vec<Rule> = Vec::new();
    let main_rules: HashSet<Rule> = [Rule::program, Rule::vars]
        .iter()
        .cloned()
        .collect();

    match PatitoParser::parse(Rule::program, &patito_file) {
        Ok(pairs) => {
            for pair in pairs {
                // println!("{:#?}", pair.into_inner());
                visualize_pair(pair, &mut context_stack, &main_rules);
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
