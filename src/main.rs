use std::fs;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "patito.pest"]
pub struct PatitoParser;

fn print_rules(pair: pest::iterators::Pair<Rule>, indent: usize) {
    // Print the current rule with indentation
    println!("{:indent$}{:?}", "", pair.as_rule(), indent = indent);

    // Iterate through all inner pairs and print their rules
    for inner_pair in pair.into_inner() {
        print_rules(inner_pair, indent + 2); // Increase indentation for inner rules
    }
}

fn main() {
    let path = "C:/Users/wetpe/Documents/Tec8/Testing Rust/patito-parser/src/patitos/app.pat";
    let patito_file = fs::read_to_string(&path).expect("error reading file");
    match PatitoParser::parse(Rule::program, &patito_file) {
        Ok(pairs) => {
            for pair in pairs {
                // println!("{:#?}", pair.as_rule());
                print_rules(pair, 0);
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
