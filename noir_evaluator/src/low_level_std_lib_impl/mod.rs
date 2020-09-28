// Functions that are in the low level standard library
// Low level std library methods are gadgets which are assumed to be present in the underlying proof system
// This means that the underlying PLONK library must have some way to deal with these methods.
// Note that standard library refers to higher level methods that are exposed by the underlying plonk api
// Currently, we do not have a way to import rasa modules, but in the future, the std library will be
// a mixture of useful gadgets from the plonk library and also rasa functions
use crate::{CallExpression, Environment, Evaluator, Polynomial, Ident};
mod sha256;

pub use sha256::Sha256Gadget;
pub use low_level_std_lib::{LowLevelStandardLibrary, HashLibrary};

pub trait GadgetCaller {
    fn name() -> LowLevelStandardLibrary;
    fn call(
        evaluator: &mut Evaluator,
        env: &mut Environment,
        call_expr: CallExpression,
    ) -> Polynomial;
}

pub fn call_low_level(        
    evaluator: &mut Evaluator,
    env: &mut Environment,
    func_name: &Ident,
    call_expr: CallExpression) -> Polynomial 
{

    let std_lib_func = LowLevelStandardLibrary::find(func_name);
    
    let func = match std_lib_func {
        None => panic!("cannot find a function with that name in the low level standard library"),
        Some(func) => func
    };

    match func {
        LowLevelStandardLibrary::Hash(HashLibrary::SHA256) => Sha256Gadget::call(evaluator, env, call_expr),
        LowLevelStandardLibrary::Hash(HashLibrary::AES) => panic!("Currently the compiler does not have an implementation for aes"),
    }
}
