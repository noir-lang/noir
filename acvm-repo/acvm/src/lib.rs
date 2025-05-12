#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

pub mod compiler;
pub mod pwg;

pub use acvm_blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError};
use pwg::OpcodeResolutionError;

// re-export acir
pub use acir;
pub use acir::{AcirField, FieldElement};
// re-export brillig vm
pub use brillig_vm;
// re-export blackbox solver
pub use acvm_blackbox_solver as blackbox_solver;

pub fn serialize_acir(acir_inp: String) -> Vec<String>{

    vec!["".to_string()]
}


#[test]
fn test_serialize_acir(){
    let input = 
    "func 0
current witness index : _1
private parameters indices : [_0]
public parameters indices : []
return value indices : [_1]
EXPR [ (-2, _0) (1, _1) 0 ]".to_string(); 
    let serialized_output = serialize_acir(input); 
    println!("serialized_output: {:?}", serialized_output); 
}