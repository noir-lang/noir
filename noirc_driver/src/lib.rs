use std::path::PathBuf;
use noirc_errors::{FileMap, Reporter};
use noirc_errors::DiagnosableError;
use noirc_frontend::lexer::Lexer;
use noirc_frontend::parser::Parser;
use noir_evaluator::{Evaluator};
use noirc_frontend::analyser;
use noirc_frontend::ast::Type;
use acir::circuit::Circuit;

pub struct Driver {
    file_map : FileMap
}


pub struct CompiledProgram {
    pub circuit : Circuit,
    pub num_witnesses : usize, 
    pub num_public_inputs : usize, 
    pub abi : Option<Vec<(String, Type)>>
}

impl Driver{
    pub fn new() -> Self {
        Driver {
            file_map : FileMap::new()
        }
    }

    pub fn compile_file(&mut self, file_name : PathBuf, source: String) -> CompiledProgram {

        // First add the file to the system 
        let file_id = self.file_map.add_file(file_name.into(), source);

        let file = self.file_map.get_file(file_id).unwrap();

        // Lex the file using it's file id to generate error diagnostics (span)
        let lexer = Lexer::from_file(file_id.0, file);

        let mut parser = Parser::new(lexer);

        let program = match parser.parse_program() {
            Ok(program) => program,
            Err(errs) => {
                let diagnostics : Vec<_> = errs.into_iter().map(|err| err.to_diagnostic()).collect();
                // Parse errors and print diagnostic for them
                Reporter::with_diagnostics(file_id, &self.file_map, &diagnostics);
                std::process::exit(1);            }
        };


        let (checked_program, symbol_table) = match analyser::check(program) {
            Ok((checked_program, symbol_table)) => (checked_program, symbol_table),
            Err(errs) => {
                let diagnostics : Vec<_> = errs.into_iter().map(|err| err.to_diagnostic()).collect();
                Reporter::with_diagnostics(file_id, &self.file_map, &diagnostics);
                std::process::exit(1);
            }
        };

        let abi = checked_program.abi();
    
        let evaluator = Evaluator::new(checked_program, symbol_table);
    
        let (circuit, num_witnesses, num_public_inputs) = evaluator.evaluate();

        CompiledProgram {
            circuit,
            num_witnesses,
            num_public_inputs, 
            abi
        }
    }
}


#[test]
fn test_driver() {
    let _ = Driver::new();
}