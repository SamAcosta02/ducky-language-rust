use std::collections::HashMap;

use super::structs::virtual_memory::{
  GlobalMemory,
  LocalMemory,
  MemorySegment
};
use super::structs::{
  dusty_context::DustyContext,
  function_info::FunctionInfo,
  var_info::VarInfo
};

fn map_address(address: usize) -> Option<(MemorySegment, usize)> {
  match address {
      1000..=2999 => Some((MemorySegment::Ints, address - 1000)),
      3000..=4999 => Some((MemorySegment::Floats, address - 3000)),
      5000..=6999 => Some((MemorySegment::IntTemps, address - 5000)),
      7000..=8999 => Some((MemorySegment::FloatTemps, address - 7000)),

      11000..=12999 => Some((MemorySegment::IntLocal, address - 11000)),
      13000..=14999 => Some((MemorySegment::FloatLocal, address - 13000)),
      15000..=16999 => Some((MemorySegment::IntLocalTemps, address - 15000)),
      17000..=18999 => Some((MemorySegment::FloatLocalTemps, address - 17000)),

      21000..=22999 => Some((MemorySegment::IntConsts, address - 21000)),
      23000..=24999 => Some((MemorySegment::FloatConsts, address - 23000)),
      25000..=26999 => Some((MemorySegment::StringConsts, address - 25000)),
      _ => None, // Address out of bounds
  }
}

fn get_value(memory: &GlobalMemory, address: usize) -> Option<(String, &'static str)> {
  if let Some((segment, offset)) = map_address(address) {
      match segment {
          MemorySegment::Ints => Some((memory.ints[offset].to_string(), "int")),
          MemorySegment::Floats => Some((memory.floats[offset].to_string(), "float")),
          MemorySegment::IntTemps => Some((memory.int_temps[offset].to_string(), "int")),
          MemorySegment::FloatTemps => Some((memory.float_temps[offset].to_string(), "float")),

          MemorySegment::IntLocal => Some((memory.memory_stack.last().unwrap().ints[offset].to_string(), "int")),
          MemorySegment::FloatLocal => Some((memory.memory_stack.last().unwrap().floats[offset].to_string(), "float")),
          MemorySegment::IntLocalTemps => Some((memory.memory_stack.last().unwrap().int_temps[offset].to_string(), "int")),
          MemorySegment::FloatLocalTemps => Some((memory.memory_stack.last().unwrap().float_temps[offset].to_string(), "float")),

          MemorySegment::IntConsts => Some((memory.int_consts[offset].to_string(), "int")),
          MemorySegment::FloatConsts => Some((memory.float_consts[offset].to_string(), "float")),
          MemorySegment::StringConsts => Some((memory.string_const[offset].to_string(), "string")),
      }
  } else {
      None // Invalid address
  }
}

fn set_value(memory: &mut GlobalMemory, address: usize, value: String) {
  if let Some((segment, offset)) = map_address(address) {
      match segment {
          MemorySegment::Ints => memory.ints[offset] = value.parse().unwrap(),
          MemorySegment::Floats => memory.floats[offset] = value.parse().unwrap(),
          MemorySegment::IntTemps => memory.int_temps[offset] = value.parse().unwrap(),
          MemorySegment::FloatTemps => memory.float_temps[offset] = value.parse().unwrap(),

          MemorySegment::IntLocal => memory.memory_stack.last_mut().unwrap().ints[offset] = value.parse().unwrap(),
          MemorySegment::FloatLocal => memory.memory_stack.last_mut().unwrap().floats[offset] = value.parse().unwrap(),
          MemorySegment::IntLocalTemps => memory.memory_stack.last_mut().unwrap().int_temps[offset] = value.parse().unwrap(),
          MemorySegment::FloatLocalTemps => memory.memory_stack.last_mut().unwrap().float_temps[offset] = value.parse().unwrap(),

          MemorySegment::IntConsts => {
              // IntConsts are usually immutable, handle this as an error if necessary
              panic!("Cannot modify constants");
          }
          MemorySegment::FloatConsts => {
              panic!("Cannot modify constants");
          }
          MemorySegment::StringConsts => memory.string_const[offset] = value,
      }
  } else {
      panic!("Invalid address");
  }
}

fn set_param_value(memory: &mut GlobalMemory, address: usize, value: String) {
  if address < 13000 {
      memory.memory_stack.last_mut().unwrap().ints[address] = value.parse().unwrap();
  } else {
      memory.memory_stack.last_mut().unwrap().floats[address] = value.parse().unwrap();
  }
}

fn get_memory_size_main(function_info: &FunctionInfo, const_count: [u32; 3]) -> [usize; 7] {
  [
      function_info.resources.int_count as usize,
      function_info.resources.temp_i_count as usize,
      function_info.resources.float_count as usize,
      function_info.resources.temp_f_count as usize,
      const_count[0] as usize,
      const_count[1] as usize,
      const_count[2] as usize,
  ]
}

fn fill_consts(const_dir: &HashMap<String, VarInfo>, virtual_memory: &mut GlobalMemory) {
  for (key, value) in const_dir {
      let memory = value.location as usize;
      match value.var_type.as_str() {
          "int" => {
              virtual_memory.int_consts[memory - 21000] = key.parse::<i32>().unwrap();
          }
          "float" => {
              virtual_memory.float_consts[memory - 23000] = key.parse::<f32>().unwrap();
          }
          "string" => {
              virtual_memory.string_const[memory - 25000] = key.clone();
          }
          _ => {}
      }
  }
}

fn bool_to_int(value: bool) -> i32 {
  if value {
      1
  } else {
      0
  }
}

fn allocate_to_stack(virtual_memory: &mut GlobalMemory, dusty_context: &DustyContext, func_name: &str) {
  virtual_memory.memory_stack.push(LocalMemory::new(
      dusty_context.func_dir.get(func_name).unwrap().resources.int_count as usize,
      dusty_context.func_dir.get(func_name).unwrap().resources.temp_i_count as usize,
      dusty_context.func_dir.get(func_name).unwrap().resources.float_count as usize,
      dusty_context.func_dir.get(func_name).unwrap().resources.temp_f_count as usize,
  ));
}

pub fn run_virtual_machine(dusty_context: &DustyContext) {
  let main_memory_size = get_memory_size_main(
      dusty_context.func_dir.get("global").unwrap(),
      dusty_context.constants,
  );
  let mut virtual_memory = GlobalMemory::new(
      main_memory_size[0], 
      main_memory_size[1], 
      main_memory_size[2], 
      main_memory_size[3], 
      main_memory_size[4], 
      main_memory_size[5], 
      main_memory_size[6]
  );
  fill_consts(&dusty_context.const_dir, &mut virtual_memory);

  let mut intruction_pointer = 0;

//   println!("{:#?}", virtual_memory);

//   println!("################### OUTPUT WINDOW ###################\n");
  while intruction_pointer < dusty_context.quadruples.len() {
      let quadruple = &dusty_context.quadruples[intruction_pointer];
      let operator = &quadruple[0].name;
      // println!("POINTER: {}", intruction_pointer);
      match operator.to_string().as_str() {
          "goto" => {
              intruction_pointer = quadruple[3].memory as usize - 1;
              // println!("GOTO: {}", quadruple[3].memory - 1);
          }
          "gotof" => {
              let (value, _) = get_value(&virtual_memory, quadruple[1].memory as usize).unwrap();
              if value == "0" {
                  intruction_pointer = quadruple[3].memory as usize;
                //   println!("GOTOF: {:#?}, Som {}", dusty_context.quadruples[intruction_pointer], intruction_pointer);
              } else {
                  intruction_pointer += 1;
              }
          }
          "era" => {
              allocate_to_stack(&mut virtual_memory, dusty_context, quadruple[3].name.as_str());
              intruction_pointer += 1;
          }
          "param" => {
              let (value, _) = get_value(&virtual_memory, quadruple[1].memory as usize).unwrap();
              set_param_value(&mut virtual_memory, quadruple[3].memory as usize, value);
              intruction_pointer += 1;
          }
          "gosub" => {
              let current_pointer = intruction_pointer + 1;
              virtual_memory.jump_stack.push(current_pointer);
              intruction_pointer = quadruple[3].memory as usize - 1;
            //   println!("GOSUB: {}", quadruple[3].memory - 1);
          }
          "endfunc" => {
              // println!("ENDFUNC");
              let return_pointer = virtual_memory.jump_stack.pop().unwrap();
              intruction_pointer = return_pointer;
          }
          "end" => {
              break;
          }
          "=" => {
              // println!("{:#?}", quadruple);
              let (value, _) = get_value(&virtual_memory, quadruple[1].memory as usize).unwrap();
              set_value(&mut virtual_memory, quadruple[3].memory as usize, value);
              intruction_pointer += 1;
          }
          "+" => {
              let (left_value, left_type) = get_value(&virtual_memory, quadruple[1].memory as usize).unwrap();
              let (right_value, right_type) = get_value(&virtual_memory, quadruple[2].memory as usize).unwrap();

              let result = if left_type == "float" || right_type == "float" {
                  // Coerce to float if either operand is a float
                  let left = left_value.parse::<f32>().unwrap();
                  let right = right_value.parse::<f32>().unwrap();
                  (left + right).to_string()
              } else {
                  // Both operands are integers
                  let left = left_value.parse::<i32>().unwrap();
                  let right = right_value.parse::<i32>().unwrap();
                  (left + right).to_string()
              };

              set_value(&mut virtual_memory, quadruple[3].memory as usize, (result).to_string());
              intruction_pointer += 1;
          }
          "-" => {
              let (left_value, left_type) = get_value(&virtual_memory, quadruple[1].memory as usize).unwrap();
              let (right_value, right_type) = get_value(&virtual_memory, quadruple[2].memory as usize).unwrap();

              let result = if left_type == "float" || right_type == "float" {
                  // Coerce to float if either operand is a float
                  let left = left_value.parse::<f32>().unwrap();
                  let right = right_value.parse::<f32>().unwrap();
                  (left - right).to_string()
              } else {
                  // Both operands are integers
                  let left = left_value.parse::<i32>().unwrap();
                  let right = right_value.parse::<i32>().unwrap();
                  (left - right).to_string()
              };

              set_value(&mut virtual_memory, quadruple[3].memory as usize, (result).to_string());
              intruction_pointer += 1;
          }
          "*" => {
              let (left_value, left_type) = get_value(&virtual_memory, quadruple[1].memory as usize).unwrap();
              let (right_value, right_type) = get_value(&virtual_memory, quadruple[2].memory as usize).unwrap();

              let result = if left_type == "float" || right_type == "float" {
                  // Coerce to float if either operand is a float
                  let left = left_value.parse::<f32>().unwrap();
                  let right = right_value.parse::<f32>().unwrap();
                  (left * right).to_string()
              } else {
                  // Both operands are integers
                  let left = left_value.parse::<i32>().unwrap();
                  let right = right_value.parse::<i32>().unwrap();
                  (left * right).to_string()
              };

              set_value(&mut virtual_memory, quadruple[3].memory as usize, (result).to_string());
              intruction_pointer += 1;
          }
          "/" => {
              let (left_value, _) = get_value(&virtual_memory, quadruple[1].memory as usize).unwrap();
              let (right_value, _) = get_value(&virtual_memory, quadruple[2].memory as usize).unwrap();

              // Both operands are integers
              let left = left_value.parse::<i32>().unwrap();
              let right = right_value.parse::<i32>().unwrap();
              let result = (left / right).to_string();

              set_value(&mut virtual_memory, quadruple[3].memory as usize, (result).to_string());
              intruction_pointer += 1;
          }
          ">" => {
              let (left_value, _) = get_value(&virtual_memory, quadruple[1].memory as usize).unwrap();
              let (right_value, _) = get_value(&virtual_memory, quadruple[2].memory as usize).unwrap();

              // Both operands are integers
              let left = left_value.parse::<i32>().unwrap();
              let right = right_value.parse::<i32>().unwrap();
              let result = (bool_to_int(left > right)).to_string();

              set_value(&mut virtual_memory, quadruple[3].memory as usize, (result).to_string());
              intruction_pointer += 1;
          }
          "<" => {
              let (left_value, _) = get_value(&virtual_memory, quadruple[1].memory as usize).unwrap();
              let (right_value, _) = get_value(&virtual_memory, quadruple[2].memory as usize).unwrap();

              // Both operands are integers
              let left = left_value.parse::<i32>().unwrap();
              let right = right_value.parse::<i32>().unwrap();
              let result = (bool_to_int(left < right)).to_string();

              set_value(&mut virtual_memory, quadruple[3].memory as usize, (result).to_string());
              intruction_pointer += 1;
          }
          "print" => {
              let (value, _) = get_value(&virtual_memory, quadruple[3].memory as usize).unwrap();
              println!("{}", value);
              intruction_pointer += 1;
          }
          _ => {intruction_pointer += 1;}
      }
  }
//   println!("\n################### OUTPUT WINDOW ###################\n");

//   println!("{:#?}", virtual_memory);
}