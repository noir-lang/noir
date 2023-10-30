#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{generate_doc, Info, get_map, Map};

    #[test]
    fn one_file() {
        assert!(generate_doc("input_files/another_module.nr").is_ok());
    }

    #[test]
    fn many_files() {
        assert!(generate_doc("input_files/prog.nr").is_ok());
    }

    #[test]
    fn function_output() {
        let mut map = HashMap::new();
        map.insert(Info::Function { signature: "fn main ( x : Field , y : pub Field ) ".to_string() }, "doc comment".to_string());

        let result = Map { 
            map
        };

        assert_eq!(get_map("input_files/function_example.nr"), result);
    }

    #[test]
    fn structure_output() {
        let mut map = HashMap::new();
        map.insert(
            Info::Struct { 
                signature: "struct MyStruct {\n/* private fields */\n}".to_string(), 
                additional_doc: "".to_string(), 
                implementations: vec![] 
            }, 
            "struct".to_string());

        let result = Map { 
            map
        };

        assert_eq!(get_map("input_files/struct_example.nr"), result);
    }
}
