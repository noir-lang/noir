// Functions that are in the low level standard library
// Low level std library methods are gadgets which are assumed to be present in the underlying proof system
// This means that the underlying PLONK library must have some way to deal with these methods.
// The standard library on the other hand, is a mixture of foreign and compiled functions.
use crate::{CallExpression, Environment, Evaluator, Object};
mod sha256;

pub use sha256::Sha256Gadget;
use acir::OPCODE;
use super::EvaluatorError;

pub trait GadgetCaller {
    fn name() -> acir::OPCODE;
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: CallExpression,
    ) -> Result<Object, EvaluatorError>;
}

pub fn call_low_level(        
    evaluator: &mut Evaluator,
    env: &mut Environment,
    opcode_name: &str,
    call_expr: CallExpression) -> Result<Object, EvaluatorError> 
{
   
    let func = match OPCODE::lookup(opcode_name) {
        None => panic!("cannot find a low level opcode with the name {} in the IR", opcode_name),
        Some(func) => func
    };

    match func {
        OPCODE::SHA256 => Sha256Gadget::call(evaluator, env, call_expr),
        k => panic!("The OPCODE {} exists, however, currently the compiler does not have a concrete implementation for it", k),
    }
}
