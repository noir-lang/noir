use acvm::pwg::witness_to_value;
use nargo::package::Package;
use noirc_abi::{
    input_parser::{Format, InputValue},
    Abi, InputMap, MAIN_RETURN_NAME,
};
use std::collections::BTreeMap;

use acvm::brillig_vm::{brillig::Value, Registers};
use fm::FileManager;
use noirc_driver::{
    compile_brillig_main, compile_main, prepare_crate, CompileOptions, CompiledProgram,
};
use noirc_errors::FileDiagnostic;
use noirc_evaluator::brillig::brillig_ir::artifact::GeneratedBrillig;
use noirc_frontend::{graph::CrateGraph, hir::Context};
use std::path::Path;

/// Compile program from file using path.
pub(crate) fn compile(
    entry_point: &str,
) -> Result<(GeneratedBrillig, CompiledProgram), Vec<FileDiagnostic>> {
    let options = CompileOptions::default();

    let path = std::env::current_dir().expect("No current directory");
    let root = Path::new(&path);
    let fm = FileManager::new(root, Box::new(|path| std::fs::read_to_string(path)));
    let graph = CrateGraph::default();
    let mut context = Context::new(fm, graph);

    let path = Path::new(entry_point);
    let crate_id = prepare_crate(&mut context, path);

    let compiled_brillig = compile_brillig_main(&mut context, crate_id, &options, None)?;
    let compiled_acvm = compile_main(&mut context, crate_id, &options, None, false)?;
    Ok((compiled_brillig.0, compiled_acvm.0))
}

pub(crate) fn get_input_registers(
    package: &Package,
    program: &CompiledProgram,
    prover_name: &str,
) -> Registers {
    let inputs_map = if let Ok((inputs_map, _)) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &program.abi)
    {
        inputs_map
    } else {
        return Registers { inner: vec![] };
    };
    let witness_map = program.abi.encode(&inputs_map, None).unwrap();
    let wmap = witness_map.clone();
    let elements: Vec<_> =
        wmap.into_iter().map(|w| witness_to_value(&witness_map, w.0).unwrap()).collect();
    let registers: Vec<_> = elements.into_iter().map(|e| Value::from(e.to_u128())).collect();
    Registers { inner: registers }
}

pub(crate) fn read_inputs_from_file<P: AsRef<Path>>(
    path: P,
    file_name: &str,
    format: Format,
    abi: &Abi,
) -> Result<(InputMap, Option<InputValue>), String> {
    if abi.is_empty() {
        return Ok((BTreeMap::new(), None));
    }

    let file_path = path.as_ref().join(file_name).with_extension(format.ext());
    if !file_path.exists() {
        return Err("File not found".into());
    }

    let input_string = std::fs::read_to_string(file_path).unwrap();
    let mut input_map = format.parse(&input_string, abi).unwrap();
    let return_value = input_map.remove(MAIN_RETURN_NAME);

    Ok((input_map, return_value))
}
