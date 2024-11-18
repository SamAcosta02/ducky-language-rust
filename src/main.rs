mod structs;
mod virtual_machine;
mod quadruples;

use std::fs;
use pest::Parser;

use virtual_machine::run_virtual_machine;
use quadruples::generate_quadruples;
use structs::{
    dusty_context::{DustyContext, Stage},
    parser::{Rule, DustyParser},
};

fn main() {
    // File path to read
    let path = "C:/Users/wetpe/OneDrive/Documents/_Manual/TEC 8/ducky-language-rust/src/tests/app8.dusty";
    let patito_file = fs::read_to_string(&path).expect("error reading file");

    let mut dusty_context = DustyContext::new();

    match DustyParser::parse(Rule::program, &patito_file) {
        Ok(pairs) => {
            // Enter the Tree and generate quadruples
            for pair in pairs.into_iter().next().unwrap().into_inner() {
                generate_quadruples(
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
    run_virtual_machine(&dusty_context);
}
