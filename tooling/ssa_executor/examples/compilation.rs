use noir_ssa_executor::compiler::compile_to_bytecode_base64;
use noirc_driver::CompileOptions;
use noirc_evaluator::ssa::ssa_gen::Ssa;

/// Compiles the given SSA program into an Brillig program and prints the bytecode
fn main() {
    let ssa = "brillig(inline) predicate_pure fn main f0 {
      b0(v0: u16, v1: Field, v2: Field, v3: Field, v4: Field, v5: u1, v6: u1):
        v9 = call to_le_radix(v3, u32 256) -> [u8; 102]
        v10 = call to_le_radix(v4, u32 256) -> [u8; 16]
        v11 = call to_le_radix(v2, u32 256) -> [u8; 16]
        v12 = call to_le_radix(v1, u32 256) -> [u8; 16]
        v13 = call to_le_radix(v3, u32 256) -> [u8; 16]
        v14 = cast v5 as u8
        return v14
    }
    ";
    let program_bytecode =
        compile_to_bytecode_base64(Ssa::from_str(ssa).unwrap(), &CompileOptions::default())
            .unwrap();
    println!("Bytecode: {program_bytecode}");
}
