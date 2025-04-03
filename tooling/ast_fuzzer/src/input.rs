use acir::FieldElement;
use arbitrary::Unstructured;
use noir_fuzzer::{dictionary::build_dictionary_from_program, strategies};
use noirc_abi::{Abi, InputMap};
use proptest::{
    prelude::Strategy,
    test_runner::{Config, RngAlgorithm, TestRng, TestRunner},
};

/// Generate an arbitrary input according to the ABI.
pub fn arb_inputs(
    u: &mut Unstructured,
    program: &acir::circuit::Program<FieldElement>,
    abi: &Abi,
) -> arbitrary::Result<InputMap> {
    // Reuse the proptest strategy in `noir_fuzzer` to generate random inputs.
    let dictionary = build_dictionary_from_program(program);
    let strategy = strategies::arb_input_map(abi, &dictionary);
    // The strategy needs a runner, although all it really uses is the RNG from it.
    let seed: [u8; 16] = u.arbitrary()?;
    let rng = TestRng::from_seed(RngAlgorithm::XorShift, &seed);
    let mut runner = TestRunner::new_with_rng(Config::default(), rng);
    let tree = strategy.new_tree(&mut runner).map_err(|_| arbitrary::Error::IncorrectFormat)?;
    Ok(tree.current())
}
