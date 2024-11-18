#[derive(Debug)]
#[derive(Clone)]
pub struct VarInfo {
    pub name: String,
    pub var_type: String,
    pub location: u32,
}

impl VarInfo {
    pub fn new(name:String, var_type: String, location: u32) -> Self {
        VarInfo {
            name,
            var_type,
            location,
        }
    }
}