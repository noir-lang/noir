//! The tests in the following files are simple tests checking the result of generating brillig bytecode
//! from a simple brillig function in SSA form. The brillig function is doing one elementary operation to
//! show how it is generated on the Brillig bytecode level.
//!
//! Every unit test will be checking against against an individual Brillig artifact
//! that has not yet undergone linking. This means any calls, to both external functions or procedures,
//! and jumps are expected to be unresolved. Thus any call/jump is expected to still have a label of `0`.

use crate::{
    brillig::{Brillig, BrilligOptions},
    ssa::ssa_gen::Ssa,
};

mod binary;
mod black_box;
mod call;
mod memory;

pub(crate) fn ssa_to_brillig_artifacts(src: &str) -> Brillig {
    let ssa = Ssa::from_str(src).unwrap();
    ssa.to_brillig(&BrilligOptions::default())
}

#[macro_export]
macro_rules! assert_artifact_snapshot {
    ($artifact:expr, $($arg:tt)*) => {
        #[allow(unused_mut)]
        let artifact_string = $artifact.to_string();
        insta::assert_snapshot!(artifact_string, $($arg)*)
    };
}
