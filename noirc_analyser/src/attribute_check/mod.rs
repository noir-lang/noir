// Currently, attributes are being used to link opcodes in Noir to ACIR
// We therefore need to check if the attribute provided is available

// We also want to check that the function body of the attribute is empty.
// There is currently no need to have anything written in it

// We also need to check that the return type matches the return type for the opcode defined in ACIR 
use noirc_frontend::ast::{Type, ArraySize};
use acir::opcode::OutputSize;
use noirc_frontend::ast::FunctionDefinition;
use noirc_frontend::parser::Program;

pub struct AttributeChecker;

impl AttributeChecker {
    pub fn check(ast : &Program) {

        // Main function should have no attributes attached
        match &ast.main {
            Some(main_func) =>    {    
                if main_func.attribute.is_some() {
                    panic!("The main function contains an attribute, this is not allowed")
                }
            },
            None => {}
        };

        AttributeChecker::check_ast(&ast);
        
        for (_, ast) in ast.modules.iter() {
            AttributeChecker::check_ast(&ast);
        }
    }

    fn check_ast(ast : &Program) {
        for func_def in ast.functions.iter() {
            AttributeChecker::check_func_def(func_def)
        }
    }

    // Check that all functions with attributes are correct 
    fn check_func_def(func : &FunctionDefinition) {
        // check if the function has an attribute present
        // If so, we check if it is valid, or return if otherwise
        let attribute = match &func.attribute{
            None => return,
            Some(attr) => attr
        };

        // If attribute is present, the function should not have a body
        // This is because we do not support this feature, and would be confusing to the user as to whether the body was being called
        if func.body.0.len() > 0 {
            panic!("Functions marked with an attribute should have an empty body")
        }

        // Check that the attribute matches an ACIR opcode
        let opcode = match acir::OPCODE::lookup(attribute.into()) {
            Some(opcode) => opcode,
            None => panic!("Currently function attributes are only used as an interface to specialised opcodes. {} is not a specialised opcode", attribute) 
        };

        // Check that the function signature, matches the opcode definition
        let opcode_definition = opcode.definition();


        // First check the return type
        let declared_return_type = &func.return_type;
        
        // Slight problem, the input is arbitrary which the array syntax does not account for
        let (num_output_elements, typ) = match declared_return_type {
            Type::Array(num_elements, typ) => (num_elements, typ),
            _=> panic!("Attributed functions can only return Arrays")
        };
        assert_eq!(**typ, Type::Witness);

        // Check output consistency
        let num_output_elements = array_size_to_output_size(&num_output_elements);
        assert_eq!(opcode_definition.output_size, num_output_elements);

    }

}

fn array_size_to_output_size(input_size : &ArraySize) -> OutputSize {
    match input_size {
        ArraySize::Fixed(integer) => OutputSize(*integer),
        ArraySize::Variable => panic!("Currently variable output size is not supported")
    }
}


