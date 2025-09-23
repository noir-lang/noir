use acvm::acir::circuit::Program;
use base64::Engine;
use noir_ssa_executor::compiler::compile_from_ssa;
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
    let compiled_program =
        compile_from_ssa(Ssa::from_str(&ssa).unwrap(), &CompileOptions::default());

    let bytecode = Program::serialize_program(&compiled_program.unwrap().program);
    let bytecode_b64 = base64::engine::general_purpose::STANDARD.encode(bytecode);
    println!("Bytecode: {bytecode_b64}");
}
