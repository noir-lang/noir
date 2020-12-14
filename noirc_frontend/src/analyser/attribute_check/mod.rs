// Currently, attributes are being used to link opcodes in Noir to ACIR
// We therefore need to check if the attribute provided is available

// We also want to check that the function body of the attribute is empty.
// There is currently no need to have anything written in it

// We also need to check that the return type matches the return type for the opcode defined in ACIR 
use crate::{FunctionKind, NoirFunction, ast::{Type, ArraySize}};
use acir::opcode::OutputSize;
use crate::parser::Program;
use super::errors::AnalyserError;
use nargo::{CrateManager, CrateUnit};

pub struct AttributeChecker;

impl AttributeChecker {

    pub fn check_crates(crate_manager: &CrateManager<Program>) -> Result<(), AnalyserError>{
        
        AttributeChecker::check_main_crate(&crate_manager)?;

        // Get all other crates except for main
        // XXX: We can generilise by classing crates as libs and binarys
        // Here we only want to get libraries
        let lib_crates = crate_manager.get_all_libraries().expect("could not fetch all libraries");
        for lib_crate in lib_crates {
            AttributeChecker::check_crate(lib_crate)?
        }
        
        Ok(())
    }
    pub fn check_crate(lib_crate: &CrateUnit<Program>) -> Result<(), AnalyserError>{
        
        for module in lib_crate.modules() {
            AttributeChecker::check_ast(module);
        }

        Ok(())
    }
    fn check_ast(ast : &Program) {
        for func_def in ast.functions.iter() {
            AttributeChecker::check_func_def(func_def)
        }
    }

    fn check_main_crate(crate_manager: &CrateManager<Program>) -> Result<(), AnalyserError>{
        
        // Look for the main crate and convert in the crate manager
        let main_crate = match crate_manager.get_crate_with_name("main") {
            None => return Ok(())/*There is no main crate, this is just a library*/,
            Some(main_crate) => main_crate
        };

        // The main crate should only have one module.
        let modules = main_crate.modules();
        if modules.len() != 1 {
            return Err(AnalyserError::Unstructured{span : Default::default(), message : "could not process the main file. Please ensure that there is only one `file` with the name main.nr".into()})
        }
        let main_module = modules.first().unwrap();
        
        // Every main crate should have a main method
        for func in main_module.functions.iter() {
            dbg!(&func.name());
        }
        let main_func = main_module.find_function("main").ok_or(AnalyserError::Unstructured{span : Default ::default(), message : "could not find a main() method in main.nr".into()})?;

        // main function should have no attributes attached to it
        if main_func.attribute().is_some() {
            // XXX: functions do not have a span, so we use the default here
            return Err(AnalyserError::Unstructured{span : Default::default(), message : "The main function in main.nr cannot contain attributes".into()})
        }

        Ok(())
    }
        // Check that all functions with attributes are correct 
        fn check_func_def(func : &NoirFunction) {
            // Check if it is a normal function
            // Normal functions do not contain attributes currently
            if FunctionKind::Normal == func.kind {
                assert!(func.def().attribute.is_none());
                return
            };

            // If attribute is present, the function should not have a body
            // This is because we do not support this feature currently, and would be confusing to the user as to whether the body was being called
            // If we had a test attribute, then we could modify this condition
            if func.number_of_statements() > 0 {
                panic!("Functions marked with an attribute should have an empty body")
            }
    
            // Checks below this point are for foreign/low-level functions 
            // Return, as builtin functions are checked at runtime, if they are available
            let (func, attribute) = match func.foreign() {
                None => return, 
                Some(func) => (func, func.attribute.as_ref().unwrap())
            };

            
            
            // Check that the attribute matches an ACIR opcode
            let opcode = match acir::OPCODE::lookup(attribute.into()) {
                Some(opcode) => opcode,
                None => panic!("This function has been tagged as a foreign function therefore it is used as an interface to specialised opcodes. {} is not a specialised opcode", attribute) 
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


