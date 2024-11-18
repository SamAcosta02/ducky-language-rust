use std::collections::HashMap;

use super::var_info::VarInfo;
use super::resources::Resources;

#[derive(Debug)]
pub struct FunctionInfo {
    // return_type: String,
    pub location: u32,
    pub resources: Resources,
    pub vars: HashMap<String, VarInfo>,
    pub params: Vec<String>
}

impl FunctionInfo {
    pub fn new(location: u32) -> Self {
        FunctionInfo {
            // return_type: String::from("void"),
            location,
            resources: Resources::new(),
            vars: HashMap::new(),
            params: Vec::new()
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&VarInfo> {
        self.vars.get(key)
    }

    pub fn get_counter(&self, var_type: &str, kind: &str) -> u32 {
        // println!("Getting counter for {} {}", var_type, kind);
        match (var_type, kind) {
            ("int", "regular") => self.resources.int_count,
            ("float", "regular") => self.resources.float_count,
            ("int", "temporal") => self.resources.temp_i_count,
            ("float", "temporal") => self.resources.temp_f_count,
            _ => 9999
        }
    }

    pub fn add_to_counter(&mut self, var_type: &str, kind: &str) {
        match (var_type, kind) {
            ("int", "regular") => self.resources.int_count += 1,
            ("float", "regular") => self.resources.float_count += 1,
            ("int", "temporal") => self.resources.temp_i_count += 1,
            ("float", "temporal") => self.resources.temp_f_count += 1,
            _ => {}
        }
    }

    pub fn insert(&mut self, key: String, var_type: String, memory: u32) {
       self.vars.insert(key.clone(), VarInfo::new(key, var_type, memory));
    }

    pub fn add_param(&mut self, param: String) {
        self.params.push(param);
    }
}