//! Print a random AST
//!
//! ```shell
//! cargo run -p noir_ast_fuzzer --example sample
//! ```
use arbitrary::Unstructured;
use noir_ast_fuzzer::{Config, arb_program};
use rand::RngCore;

fn main() {
    let data = {
        let mut rng = rand::thread_rng();
        let mut data = [0u8; 1024 * 1024];
        rng.fill_bytes(&mut data);
        data
    };
    let mut u = Unstructured::new(&data);

    let config = Config { max_globals: 3, max_functions: 3, max_function_args: 3 };

    let (program, _abi) = arb_program(&mut u, config).unwrap();

    println!("{program}");
}
