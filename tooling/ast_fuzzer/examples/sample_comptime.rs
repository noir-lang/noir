//! Print a random comptime AST
//!
//! ```shell
//! cargo run -p noir_ast_fuzzer --example sample_comptime
//! ```
use arbitrary::Unstructured;
use noir_ast_fuzzer::{Config, DisplayAstAsNoirComptime, arb_program_comptime};
use rand::RngCore;

fn main() {
    let data = {
        let mut rng = rand::rng();
        let mut data = [0u8; 1024 * 1024];
        rng.fill_bytes(&mut data);
        data
    };
    let mut u = Unstructured::new(&data);

    let config = Config { max_globals: 0, ..Default::default() };

    let program = arb_program_comptime(&mut u, config).expect("arb_program");
    println!("{}", DisplayAstAsNoirComptime(&program));
}
