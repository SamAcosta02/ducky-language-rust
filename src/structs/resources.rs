#[derive(Debug)]
pub struct Resources {
  pub int_count: u32,
  pub float_count: u32,
  pub temp_i_count: u32,
  pub temp_f_count: u32,
}

impl Resources {
    pub fn new() -> Self {
        Resources {
            int_count: 0,
            float_count: 0,
            temp_i_count: 0,
            temp_f_count: 0
        }
    }
}