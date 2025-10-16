//! Print a random AST
//!
//! ```shell
//! cargo run -p noir_ast_fuzzer --example sample
//! ```
use arbitrary::Unstructured;
use noir_ast_fuzzer::{Config, DisplayAstAsNoir, arb_program};
use rand::RngCore;

fn main() {
    let data = {
        let mut rng = rand::rng();
        let mut data = [0u8; 1024 * 1024];
        rng.fill_bytes(&mut data);
        data
    };
    let mut u = Unstructured::new(&data);

    let program = arb_program(&mut u, Config::default()).expect("arb_program");

    println!("{}", DisplayAstAsNoir(&program));
}
