use colored::*;

use super::structs::{
  var_info::VarInfo,
  dusty_context::{DustyContext, Stage},
  parser::Rule,
  function_info::FunctionInfo,
};

pub fn generate_quadruples(
  pair: pest::iterators::Pair<Rule>,
  stage: Stage,
  dusty_context: &mut DustyContext
) {
  // println!("Processing rule: {:#?} in stage {:#?}, parent rule: {:#?}, currrent func: {:#?}, line: {:#?}, col: {:#?}",
  //     pair.as_rule(), stage, dusty_context.parent_rules.last().unwrap(), dusty_context.current_func,
  //     pair.as_span().start_pos().line_col().0,
  //     pair.as_span().start_pos().line_col().1
  // );

  match (pair.as_rule(), &stage) {
      // Process beginKeyword ----------------------------
      (Rule::beginKeyword, Stage::Before) => {
          // println!("  token BEGIN found:");
        //   println!("  Filling initial GOTO quad");
          dusty_context.quadruples[0][3].memory = dusty_context.quad_data.quad_counter as u32;
          dusty_context.quadruples[0][3].name = dusty_context.quad_data.quad_counter.to_string();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process beginKeyword ----------------------------

      // Process ID --------------------------------------
      (Rule::id, Stage::Before) => {
          // println!("  Token ID found: {:#?}", pair.as_str());
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::id, Stage::During) => {
          match dusty_context.parent_rules.last().unwrap() {
              Rule::program => {
                  // println!("  Adding global scope to function directory"); // #1 Add global scope during program name
                  dusty_context.func_dir.insert("global".to_string(), FunctionInfo::new(0));
                  dusty_context.current_func = "global".to_string();
              }
              Rule::vars => {
                  // println!("  Adding variable stack to add to directory after knowing its type"); // #2 Add variable to stack at ID in VARS
                  dusty_context.id_stack.push(pair.as_str().to_string());                  
              }
              Rule::funcs => {
                  // println!("  Adding function scope to function directory"); // #3 Add function scope during function name
                  dusty_context.func_dir.insert(pair.as_str().to_string(), FunctionInfo::new(0));
                  dusty_context.current_func = pair.as_str().to_string();
              }
              Rule::id_type_list => {
                  // println!("  Adding ID to stack to add to directory after knowing its type"); // #4 Add ID to stack at ID_LIST
                  dusty_context.id_stack.push(pair.as_str().to_string());
              }
              Rule::assign => {
                  // Quad generation
                  if dusty_context.contains_id(pair.as_str()) {
                      let var = dusty_context.func_dir.get(&dusty_context.current_func).unwrap().get(pair.as_str()).unwrap();

                      dusty_context.quad_data.operand_stack.push(var.clone());

                      // println!("{:#?}", dusty_context.quad_data.operand_stack);
                  } else if dusty_context.id_in_global_scope(pair.as_str()) {
                      let var = dusty_context.func_dir.get("global").unwrap().get(pair.as_str()).unwrap();

                      dusty_context.quad_data.operand_stack.push(var.clone());

                      // println!("{:#?}", dusty_context.quad_data.operand_stack);
                  } else {
                      panic!("ERROR: ID \"{}\" not found in current context", pair.as_str());
                  }
              }
              Rule::func_call => {
                  // println!("  Generate GOSUB quad to call function"); // #8 Generate GOSUB quad to call function
                  if !dusty_context.func_dir.contains_key(pair.as_str()) {
                      let function_name = pair.as_str();
                      let error_message = format!("ERROR: Function \"{}\" was not declared", function_name.red());
                      panic!("{:}", error_message.red());
                  }
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
                      // println!("  ID \"{}\" was found in current context", pair.as_str());
                  }
              }
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::id, Stage::After) => {
          match dusty_context.parent_rules.last().unwrap() {
              Rule::value => {
                  if dusty_context.contains_id(pair.as_str()) {
                      // println!("  (#1) Adding ID and type to operand stack in factor"); // #1.1 Add ID and type to operand stack in FACTOR
                      let var = dusty_context.func_dir.get(&dusty_context.current_func).unwrap().get(pair.as_str()).unwrap().clone();
                      dusty_context.quad_data.operand_stack.push(var);
                  } else {
                      // println!("  (#1) Adding global ID and type to operand stack in factor"); // #1.1 Add ID and type to operand stack in FACTOR
                      let var = dusty_context.func_dir.get("global").unwrap().get(pair.as_str()).unwrap().clone();
                      dusty_context.quad_data.operand_stack.push(var);
                  }
                  // println!("  Operand stack: {:?}", dusty_context.quad_data.operand_stack);
              }
              _ => {}
          }
          // println!("\n");
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process ID --------------------------------------


      // Process Vars ------------------------------------
      (Rule::vars, Stage::Before) => {
          // println!("  Sintactic rule VARS found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::vars);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::vars, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::vars, Stage::After) => {
          // println!("func_dir after vars: {:#?}", dusty_context.func_dir);
          // println!("\n");
          dusty_context.parent_rules.pop();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process Vars ------------------------------------
      

      // Process typeVar ---------------------------------
      (Rule::typeVar, Stage::Before) => {
          // println!("  Sintactic rule TYPEVAR found: {:#?}", pair.as_str());
          dusty_context.current_type = pair.as_str().to_string();
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::typeVar, Stage::During) => {
          // println!("ID stack: {:#?}", dusty_context.id_stack);
          // println!("Add all pending ids to the current scope and set them to current type");
          while let Some(id) = dusty_context.id_stack.pop() {
              if dusty_context.contains_id(&id) {
                  panic!("ERROR: id {} already exists in current context (typevar)", id);
              } else {
                  // Create variable Info
                  let var_type = dusty_context.current_type.clone();
                  let base = dusty_context.quad_data.get_memory_segment(&var_type, &dusty_context.current_func, "regular");
                  let counter = dusty_context.func_dir.get_mut(&dusty_context.current_func).unwrap().get_counter(&var_type, "regular");
                  
                  // println!("Adding id {} to {} as {} in {}", id, dusty_context.current_func, dusty_context.current_type, base+counter);
                  
                  // Insert variable to function directory
                  dusty_context.func_dir
                      .get_mut(&dusty_context.current_func)
                      .unwrap()
                      .insert(id.clone(), dusty_context.current_type.clone(), base+counter);

                  // Increase counter
                  dusty_context.func_dir
                      .get_mut(&dusty_context.current_func)
                      .unwrap()
                      .add_to_counter(&dusty_context.current_type, "regular");
              }
          }
          // println!("func_dir: {:#?}", dusty_context.func_dir);
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::typeVar, Stage::After) => {
          // println!("\n");
          match dusty_context.parent_rules.last().unwrap() {
              Rule::id_type_list => {
                 dusty_context.func_dir.get_mut(&dusty_context.current_func).unwrap().add_param(dusty_context.current_type.clone());
              }
              _ => {}
          }
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process typeVar ---------------------------------


      // Process id_list ---------------------------------
      (Rule::id_list, Stage::Before) => {
          // println!("  Sintactic rule ID_LIST found: {:#?}", pair.as_str());
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process id_list ---------------------------------


      // Process Functions -------------------------------
      (Rule::funcs, Stage::Before) => {
          // println!("  Sintactic rule FUNCTION found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::funcs);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::funcs, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::funcs, Stage::After) => {
          // println!("\n");
          dusty_context.parent_rules.pop();
          dusty_context.current_func = "global".to_string();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process Functions -------------------------------


      // Process parameters ------------------------------
      (Rule::parameters, Stage::Before) => {
          // println!("  Sintactic rule PARAMETERS found: {:#?}", pair.as_str());
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process parameters ------------------------------


      // Process func_body -------------------------------
      (Rule::func_body, Stage::Before) => {
          // println!("  Sintactic rule FUNC_BODY found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::func_body);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::func_body, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::func_body, Stage::After) => {
          // println!("\n");
          dusty_context.parent_rules.pop();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process func_body -------------------------------


      // Process func_call -------------------------------
      (Rule::func_call, Stage::Before) => {
          // println!("  Sintactic rule FUNC_CALL found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::func_call);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::func_call, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::func_call, Stage::After) => {
          // println!("\n");
          dusty_context.parent_rules.pop();
          dusty_context.quad_data.param_counter = 0;
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process func_call -------------------------------


      // Process id_type_list ----------------------------
      (Rule::id_type_list, Stage::Before) => {
          // println!("  Sintactic rule ID_TYPE_LIST found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::id_type_list);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::id_type_list, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::id_type_list, Stage::After) => {
          // println!("\n");
          dusty_context.parent_rules.pop();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process id_type_list ----------------------------


      // Process body ------------------------------------
      (Rule::body, Stage::Before) => {
          // println!("  Sintactic rule BODY found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::body);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::body, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::body, Stage::After) => {
          // println!("\n");
          dusty_context.parent_rules.pop();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process body ------------------------------------


      // Process statement -------------------------------
      (Rule::statement, Stage::Before) => {
          // println!("\n");
          // println!("  Sintactic rule STATEMENT found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::statement);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::statement, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::statement, Stage::After) => {
          dusty_context.parent_rules.pop();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process statement -------------------------------


      // Process while -----------------------------------
      (Rule::while_loop, Stage::Before) => {
          // println!("  Sintactic rule WHILE found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::while_loop);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::while_loop, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::while_loop, Stage::After) => {
          dusty_context.parent_rules.pop();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process while -----------------------------------


      // Process doKeyword -------------------------------
      (Rule::doKeyword, Stage::Before) => {
          // println!("  token DO found:");
          // println!("  (#?) Generate GOTO quad to start of while loop");
          dusty_context.generate_gotof_quad();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process doKeyword -------------------------------
      

      // Process if --------------------------------------
      (Rule::condition, Stage::Before) => {
          // println!("  Sintactic rule CONDITION found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::condition);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::condition, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::condition, Stage::After) => {
          dusty_context.parent_rules.pop();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process if --------------------------------------


      // Process elseKeyword -----------------------------
      (Rule::elseKeyword, Stage::Before) => {
          // println!("  token rule ELSE found: {:#?}", pair.as_str());
        //   println!("filling jump...");
          dusty_context.fill_jump();
          dusty_context.quad_data.jump_stack.push(dusty_context.quad_data.quad_counter);
          dusty_context.generate_goto_quad();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process elseKeyword -----------------------------


      // Process print -----------------------------------
      (Rule::print, Stage::Before) => {
          // println!("  Sintactic rule PRINT found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::print);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::print, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::print, Stage::After) => {
          dusty_context.parent_rules.pop();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process print -----------------------------------


      // Process print_element ---------------------------
      (Rule::print_element, Stage::Before) => {
          // println!("  Sintactic rule PRINT_ELEMENT found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::print_element);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::print_element, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::print_element, Stage::After) => {
          dusty_context.parent_rules.pop();
          dusty_context.generate_print_quad();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process print_element ---------------------------


      // Process assignment ------------------------------
      (Rule::assign, Stage::Before) => {
          // println!("  Sintactic rule ASSIGNMENT found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::assign);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::assign, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::assign, Stage::After) => {
          dusty_context.parent_rules.pop();
          if dusty_context.top_is_equals() {
              // println!("  (#7) Execute #4 with =");
              dusty_context.generate_assign_quad();
          }
          // dusty_context.debug_quad_gen();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process assignment ------------------------------


      // Process expression ------------------------------
      (Rule::expression, Stage::Before) => {
          // println!("  Sintactic rule EXPRESSION found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::expression);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::expression, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::expression, Stage::After) => {
          dusty_context.parent_rules.pop();
          if dusty_context.top_is_logical_operator() {
              // println!("  (#6) Execute #4 with >, <, == or !=");
              dusty_context.generate_full_quad();
          }

          // Parameters for function call
          match dusty_context.parent_rules.last().unwrap() {
              Rule::func_call => {
                  // println!("  (#?) Generate PARAM quad for function call");
                  dusty_context.generate_param_quad();
              }
              _ => {}
          }

          // dusty_context.debug_quad_gen();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process expression ------------------------------


      // Process exp -------------------------------------
      (Rule::exp, Stage::Before) => {
          // println!("  Sintactic rule EXP found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::exp);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::exp, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::exp, Stage::After) => {
          dusty_context.parent_rules.pop();
          if dusty_context.top_is_addition_or_subtraction() {
              // println!("  (#4) Execute #4 with + or -");
              dusty_context.generate_full_quad();
          }
          // dusty_context.debug_quad_gen();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process exp -------------------------------------


      // Process term ------------------------------------
      (Rule::term, Stage::Before) => {
          // println!("  Sintactic rule TERM found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::term);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::term, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::term, Stage::After) => {
          if dusty_context.top_is_multiplication_or_division() {
              // println!("  (#5) Execute #4 with * or /");
              dusty_context.generate_full_quad();
          }
          // dusty_context.debug_quad_gen();
          dusty_context.parent_rules.pop();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process term ------------------------------------
      

      // Process factor ----------------------------------
      (Rule::factor, Stage::Before) => {
          // println!("  Sintactic rule FACTOR found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::factor);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::factor, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::factor, Stage::After) => {
          dusty_context.parent_rules.pop();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process factor ----------------------------------


      // Process value -----------------------------------
      (Rule::value, Stage::Before) => {
          // println!("  Sintactic rule VALUE found: {:#?}", pair.as_str());
          dusty_context.parent_rules.push(Rule::value);
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::value, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::value, Stage::After) => {
          dusty_context.parent_rules.pop();
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process value -----------------------------------


      // Process operator --------------------------------
      (Rule::operator, Stage::Before) => {
          // println!("  token OPERATOR found: {:#?}", pair.as_str());
          if dusty_context.top_is_multiplication_or_division() {
              // println!("  (#10) (Encountered * or / but there is at least 1 that needs to be executed before... Execute #4 with * or /");
              dusty_context.generate_full_quad();
          }
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::operator, Stage::During) => {
          // println!("  (#2) Push operator to operator stack");
          dusty_context.quad_data.operator_stack.push(pair.as_str().to_string());
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process operator --------------------------------


      // Process sign ------------------------------------
      (Rule::sign, Stage::Before) => {
          // println!("  token SIGN found: {:#?}", pair.as_str());
          if dusty_context.top_is_addition_or_subtraction() {
              // println!("  (#11) (Encountered + or - but there is at least 1 that needs to be executed before... Execute #4 with + or -");
              dusty_context.generate_full_quad();
          }
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::sign, Stage::During) => {
          // println!("  (#3) Push sign to operator stack");
          dusty_context.quad_data.operator_stack.push(pair.as_str().to_string());
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process sign ------------------------------------


      // Process logical_operator -------------------------
      (Rule::comparator, Stage::Before) => {
          // println!("  token COMPARATOR found: {:#?}", pair.as_str());
          // println!("  (#4) Push logical operator to operator stack");
          dusty_context.quad_data.operator_stack.push(pair.as_str().to_string());
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process logical_operator -------------------------

      
      // Process equals ----------------------------------
      (Rule::equals, Stage::Before) => {
          // println!("  token EQUALS found: {:#?}", pair.as_str());
          // println!("  (#5) Push equals to operator stack");
          dusty_context.quad_data.operator_stack.push(pair.as_str().to_string());
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process equals ----------------------------------


      // Process open_parenthesis -------------------------
      (Rule::openP, Stage::Before) => {
          // println!("  token OPEN_PARENTHESIS found: {:#?}", pair.as_str());
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::openP, Stage::During) => {
          match dusty_context.parent_rules.last().unwrap() {
              Rule::factor => {
                  // println!("  (#6) Push open parenthesis to operator stack");
                  dusty_context.quad_data.operator_stack.push(pair.as_str().to_string());
              }
              Rule::while_loop => {
                  // println!("  (#?) Push to jump stack");
                  dusty_context.quad_data.jump_stack.push(dusty_context.quad_data.quad_counter);
              }
              _ => {}
          }
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process open_parenthesis -------------------------


      // Process close_parenthesis ------------------------
      (Rule::closeP, Stage::Before) => {
          // println!("  token CLOSE_PARENTHESIS found: {:#?}", pair.as_str());
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::closeP, Stage::During) => {
          match dusty_context.parent_rules.last().unwrap() {
              Rule::factor => {
                  // println!("  (#9) pop stack");
                  dusty_context.quad_data.operator_stack.pop();
                  println!("  {:#?}", dusty_context.quad_data.operator_stack);
              }
              Rule::condition => {
                  // println!("  (#12) Generate incomplete GOTOF quad and push to jump stack");
                  dusty_context.generate_gotof_quad();
              }
              Rule::func_call => {
                  // println!("  (#?) Generate GOSUB quad to call function");
                  // Check for correct number of parameters
                  if dusty_context.quad_data.param_counter != dusty_context.func_dir.get(&dusty_context.current_call).unwrap().params.len() {
                      let error_message = format!("ERROR: Function \"{}\" was called with {} parameters, expected {}. Line: {}, Col: {}",
                          dusty_context.current_call.red(),
                          dusty_context.quad_data.param_counter,
                          dusty_context.func_dir.get(&dusty_context.current_call).unwrap().params.len(),
                          pair.as_span().start_pos().line_col().0,
                          pair.as_span().start_pos().line_col().1
                      );
                      panic!("{:}", error_message.red());
                  } 
                  dusty_context.generate_gosub_quad();
              }
              Rule::funcs => {
                  // println!("  (#?) Assign quadruple location to the start of the function");
                  dusty_context.func_dir.get_mut(&dusty_context.current_func).unwrap().location = dusty_context.quad_data.quad_counter as u32;
              }
              _ => {}
          }
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process close_parenthesis ------------------------


      // Process cte -------------------------------------
      (Rule::cte, Stage::Before) => {
          // println!("  Sintactic rule CTE found: {:#?}", pair.as_str());
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::cte, Stage::During) => {
          let inner_pairs = pair.clone().into_inner();
          for inner_pair in inner_pairs {
              generate_quadruples(
                  inner_pair,
                  Stage::Before,
                  dusty_context
              );
          }
          generate_quadruples(pair, Stage::After, dusty_context);
      }
      (Rule::cte, Stage::After) => {
          // dusty_context.parent_rule = Rule::value;
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process cte -------------------------------------

      
      // Process cte_int ---------------------------------
      (Rule::cte_int, Stage::Before) => {
          // println!("  token CTE found: {:#?}", pair.as_str());
          // println!("  (#1) Adding CTE to operand stack in factor");
          if dusty_context.const_dir.contains_key(pair.as_str()) {
              let const_var = dusty_context.const_dir.get(pair.as_str()).unwrap().clone();
              dusty_context.quad_data.operand_stack.push(const_var);
          } else {
              let const_var = VarInfo::new(
                  pair.as_str().to_string(),
                  "int".to_string(),
                  dusty_context.quad_data.get_memory_segment("int", "global", "constant") + dusty_context.constants[0],
              );
              dusty_context.const_dir.insert(pair.as_str().to_string(), const_var.clone());
              dusty_context.quad_data.operand_stack.push(const_var);
              dusty_context.constants[0] += 1;
          }
          // println!("  Operand stack: {:?}", dusty_context.quad_data.operand_stack);
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process cte_int ---------------------------------


      // Process cte_float -------------------------------
      (Rule::cte_float, Stage::Before) => {
          // println!("  token CTE_FLOAT found: {:#?}", pair.as_str());
          // println!("  (#1) Adding CTE_FLOAT to operand stack in factor");
          if dusty_context.const_dir.contains_key(pair.as_str()) {
              let const_var = dusty_context.const_dir.get(pair.as_str()).unwrap().clone();
              dusty_context.quad_data.operand_stack.push(const_var);
          } else {
              let const_var = VarInfo::new(
                  pair.as_str().to_string(),
                  "float".to_string(),
                  dusty_context.quad_data.get_memory_segment("float", "global", "constant") + dusty_context.constants[1],
              );
              dusty_context.const_dir.insert(pair.as_str().to_string(), const_var.clone());
              dusty_context.quad_data.operand_stack.push(const_var);
              dusty_context.constants[1] += 1;
          }
          // println!("  Operand stack: {:?}", dusty_context.quad_data.operand_stack);
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process cte_float -------------------------------


      // Process string ----------------------------------
      (Rule::string, Stage::Before) => {
        //   println!("  token STRING found: {:#?}", pair.as_str());
          if dusty_context.const_dir.contains_key(pair.as_str()) {
              let const_var = dusty_context.const_dir.get(pair.as_str()).unwrap().clone();
              dusty_context.quad_data.operand_stack.push(const_var);
          } else {
              let const_var = VarInfo::new(
                  pair.as_str().to_string().trim_matches('\"').to_string(),
                  "string".to_string(),
                  dusty_context.quad_data.get_memory_segment("string", "global", "constant") + dusty_context.constants[2],
              );
              dusty_context.const_dir.insert(pair.as_str().to_string().trim_matches('\"').to_string(), const_var.clone());
              dusty_context.quad_data.operand_stack.push(const_var);
              dusty_context.constants[2] += 1;
          }
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process string ----------------------------------


      // Process delimiter -------------------------------
      (Rule::delimiter, Stage::Before) => {
          // println!("  token DELIMITER found: {:#?}", pair.as_str());
          generate_quadruples(pair, Stage::During, dusty_context);
      }
      (Rule::delimiter, Stage::During) => {
          match dusty_context.parent_rules.last().unwrap() {
              Rule::condition => {
                  // println!("  (#13) Complete GOTOF quad");
                //   println!("filling jump...");
                  dusty_context.fill_jump();
              }
              Rule::while_loop => {
                  // println!("  (#?) Generate GOTO quad to start of while loop");
                //   println!("filling jump while...");
                  dusty_context.fill_while_start();
                  dusty_context.generate_gotow_quad();
                //   println!("filling jump while end...");
                  dusty_context.fill_while_end();
              }
              Rule::funcs => {
                  // println!("  (#?) Generate ENDFUNC to indicate functions end");
                  dusty_context.generate_endfunc_quad();
              }
              Rule::program => {
                  // println!("  (#?) Generate first GOTO quad to start of program");
                  dusty_context.generate_goto_quad();
              }
              _ => {}
          }
          generate_quadruples(pair, Stage::Finished, dusty_context);
      }
      // Process delimiter -------------------------------


      // Process endKeyword ------------------------------
      (Rule::endKeyword, Stage::Before) => {
          // println!("  token END found");
          // println!("  Generating END quad");
          dusty_context.generate_end_quad();
      }
      // Process endKeyword ------------------------------


      // Anything else (move on to the next pair)
      _ => {
          // println!("...");
      }
  }
}
