// Currently, attributes are being used to link opcodes in Noir to ACIR
// and to define Builtin methods
// We therefore need to check if the attribute provided is available

// We also want to check that the function body of the attribute is empty.

// We also need to check that the return type matches the return type for the opcode defined in ACIR 
use crate::{FunctionKind, ast::{Type, ArraySize}, hir::{basic_data::FunctionBasicData, crate_graph::CrateId}, token::Attribute};
use acir::opcode::OutputSize;
use super::errors::AnalyserError;
use crate::hir::Context;
pub struct AttributeChecker;

impl AttributeChecker {

    // Since Checking for attributes is not contextual, we can iterate over all of the functions 
    // irregardless of crate and check their attributes
    pub fn check_crates(context: &Context) -> Result<(), AnalyserError>{
        
        for func_id in context.functions() {
            let func_basic_data = context.function_basic_data(func_id);
            AttributeChecker::check_func_def(func_basic_data)
        }
        
        Ok(())
    }

    fn check_binary_crates(context: &Context) -> Result<(), AnalyserError>{ 

        // First get all binary crates
        for crate_id in context.binary_crates() {
            AttributeChecker::check_binary_crate(context, crate_id)?
        }

        Ok(())

    } 

    fn check_binary_crate(context: &Context, crate_id : CrateId) -> Result<(), AnalyserError>{ 
        // Get that crate from the context
        let krate = context.def_map(crate_id).expect("expected a CrateDefMap. This is an ice.");

        // Locate the root module, this is the file which has the main method inside
        // Implicit unwrap here is fine.
        let root_mod = &krate.modules[krate.root.0];
        let file_id = &context.crate_graph()[crate_id].root_file_id;

        // Find main method
        let main_func = match root_mod.scope.find_func_with_name("main"){
            None => {
                return Err(AnalyserError::Unstructured{file_id : file_id.as_usize(), span : Default::default(), message : "could not find a main function in binary crate".into()})
            },
            Some(func) => func,
        };

        let main_func_basic = context.function_basic_data(*main_func);
        if main_func_basic.attributes.is_some() {
            // XXX: functions do not have a span right now, so we use the default here
            return Err(AnalyserError::Unstructured{file_id : file_id.as_usize(), span : Default::default(), message : "The main function in main.nr cannot contain attributes".into()})
        }

        Ok(())

    }
        // Check that all functions with attributes are correct 
        // XXX: We may be able to move this check to the function_arena. Notice that it does not require the Context
        fn check_func_def(func : &FunctionBasicData) {

            // Check if it is a normal function
            // Normal functions do not contain attributes currently
            // ie any attributes that are attached to a function immediately make it non-normal
            if func.kind == FunctionKind::Normal {
                assert!(func.attributes.is_none(), "ice : currently all attributes make a function non-normal");
                return
            }

            // If attribute is present, the function should not have a body
            // This is because we do not support this feature currently, and would be confusing to the user as to whether the body was being called
            // If we had a test attribute, then we could modify this condition
            if func.body_id.is_some() {
                // XXX: This is a user error, remove panic in next iteration
                panic!("functions marked with an attribute should have an empty body")
            }
    
            // Checks below this point are for foreign/low-level functions 
            if func.kind == FunctionKind::LowLevel {
                AttributeChecker::check_foreign_func(func)
            }
            // Builtin functions are checked at runtime, if they are available
            // XXX: It would not be bad to remove the syntax to declare builtins in the code
            // The main reason we have them is so that users do not need to check the compiler code to
            // find builtins. In regular programming, it is fine since the builtins are usually types, while for us, builtins are methods
            // that we cannot describe in Noir just yet. Since Noir is not Turing Complete. An added benefit of describing builtins in code
            // is that we can also see their namespace clearly.
        }

    fn check_foreign_func(func : &FunctionBasicData) {
        
        let attribute = match &func.attributes {
            Some(Attribute::Foreign(attr)) => attr,
            _=> unreachable!("ice: this function is only called when we have a foreign function, so the foreign attribute should be present")
        };

        // Check that the attribute matches an ACIR opcode
        let opcode = match acir::OPCODE::lookup(&attribute) {
            Some(opcode) => opcode,
            None => panic!("This function has been tagged as a foreign function therefore it is used as an interface to specialised opcodes. {} is not a specialised opcode", attribute) 
        };
            
        // Check that the function signature, matches the opcode definition
        let opcode_definition = opcode.definition();

        // The input can be arbitrary which the array syntax does not account for
        // An example of this is SHA256.
        // Lets assume that the input is always arbitrary, so we do not need to check it
        
        // Check the return type semantically matches what ACIR expects
        let (num_output_elements, typ) = match &func.return_type {
            Type::Array(num_elements, typ) => (num_elements, typ),
            _=> panic!("ice: foreign functions can only return sized arrays. This is the fundamental abi between Noir and ACIR")
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


