use std::collections::HashMap;

#[derive(Debug)]
pub struct SemanticCube {
    pub cube: [[[String; 9]; 2]; 2],
    pub string_to_usize: HashMap<String, usize>,
}

impl SemanticCube {
    pub fn new() -> Self {
        SemanticCube {
            cube: [
                // Left operand is int (0)
                [
                    [// Right operand int (0) for...
                        String::from("int"),   // +
                        String::from("int"),   // -
                        String::from("int"),   // *
                        String::from("float"), // /
                        String::from("int"),   // <
                        String::from("int"),   // >
                        String::from("int"),   // ==
                        String::from("int"),   // !=
                        String::from("int"),   // =
                    ],  
                    [// Right operand float (1) for +, -, *, /
                        String::from("float"), // +
                        String::from("float"), // -
                        String::from("float"), // *
                        String::from("float"), // /
                        String::from("error"),   // <
                        String::from("error"),   // >
                        String::from("error"),   // ==
                        String::from("error"),   // !=
                        String::from("error"),   // = 
                    ],
                ],
                // Left operand is float (1)
                [
                    [// Right operand int (0) for...
                        String::from("float"), // +
                        String::from("float"), // -
                        String::from("float"), // *
                        String::from("float"), // /
                        String::from("error"),   // <
                        String::from("error"),   // >
                        String::from("error"),   // ==
                        String::from("error"),   // !=
                        String::from("error"),   // =
                    ],  
                    [// Right operand float (1) for +, -, *, /
                        String::from("float"), // +
                        String::from("float"), // -
                        String::from("float"), // *
                        String::from("float"), // /
                        String::from("int"),   // <
                        String::from("int"),   // >
                        String::from("int"),   // ==
                        String::from("int"),   // !=
                        String::from("int"),   // =
                    ],
                ],
            ],
            string_to_usize: {
                let mut map = HashMap::new();
                map.insert(String::from("int"), 0);
                map.insert(String::from("float"), 1);
                map.insert(String::from("+"), 0);
                map.insert(String::from("-"), 1);
                map.insert(String::from("*"),2);
                map.insert(String::from("/"), 3);
                map.insert(String::from("<"), 4);
                map.insert(String::from(">"), 5);
                map.insert(String::from("=="), 6);
                map.insert(String::from("!="), 7);
                map.insert(String::from("="), 8);
                map
            }
        }
    }

    pub fn get_result_type(&self, left: &str, right: &str, operator: &str) -> String {
        let left_usize = self.string_to_usize.get(left).unwrap();
        let right_usize = self.string_to_usize.get(right).unwrap();
        let operator_usize = self.string_to_usize.get(operator).unwrap();
        self.cube[*left_usize][*right_usize][*operator_usize].clone()
    }
}