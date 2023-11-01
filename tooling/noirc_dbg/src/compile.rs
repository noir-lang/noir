use acvm::FieldElement;
use nargo::package::Package;
use noirc_abi::{
    errors::AbiError,
    input_parser::{Format, InputValue},
    Abi, AbiType, InputMap, MAIN_RETURN_NAME,
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
pub(crate) fn compile_brillig(
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

/// Get inputs for a program. Dynamic data will be stored in memory, the simple variables will be
/// stored in registers.
pub(crate) fn get_input_registers_and_memory(
    package: &Package,
    program: &CompiledProgram,
    prover_name: &str,
) -> (Registers, Vec<Value>) {
    let inputs_map = if let Ok((inputs_map, _)) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &program.abi)
    {
        inputs_map
    } else {
        return (Registers { inner: vec![] }, vec![]);
    };

    let (r, m) = map_inputs_to_registers_and_memory(&inputs_map, &program.abi).unwrap_or_default();
    let inner: Vec<_> = r.iter().map(|e| Value::from(e.to_u128())).collect();
    let memory: Vec<_> = m.iter().map(|e| Value::from(e.to_u128())).collect();
    (Registers { inner }, memory)
}

/// Helper for reading inputs and converting in into brillig compatible values.
// Copy of function, the visibility problems.
fn read_inputs_from_file<P: AsRef<Path>>(
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

/// Transform map of inputs into vectors of field elements that represents registers and memory.
fn map_inputs_to_registers_and_memory(
    input_map: &BTreeMap<String, InputValue>,
    abi: &Abi,
) -> Result<(Vec<FieldElement>, Vec<FieldElement>), AbiError> {
    let encoded_input_map: BTreeMap<String, (Vec<FieldElement>, bool)> = abi
        .to_btree_map()
        .into_iter()
        .map(|(param_name, expected_type)| {
            let value = input_map
                .get(&param_name)
                .ok_or_else(|| AbiError::MissingParam(param_name.clone()))?
                .clone();

            if !value.matches_abi(&expected_type) {
                let param =
                    abi.parameters.iter().find(|param| param.name == param_name).unwrap().clone();
                return Err(AbiError::TypeMismatch { param, value });
            }

            encode_value(value, &expected_type).map(|v| (param_name, v))
        })
        .collect::<Result<_, _>>()?;

    let mut registers: Vec<FieldElement> = vec![];
    let mut memory: Vec<FieldElement> = vec![];

    encoded_input_map.into_iter().for_each(|(_k, v)| {
        if v.1 {
            memory.extend(v.0);
        } else {
            registers.extend(v.0);
        }
    });

    Ok((registers, memory))
}

/// Compile input value and decide the value storage (memory or registers).
fn encode_value(
    value: InputValue,
    abi_type: &AbiType,
) -> Result<(Vec<FieldElement>, bool), AbiError> {
    let mut encoded_value = Vec::new();
    let mut in_memory = true;
    match (value, abi_type) {
        (InputValue::Field(elem), _) => {
            encoded_value.push(elem);
            in_memory = false;
        }

        (InputValue::Vec(vec_elements), AbiType::Array { typ, .. }) => {
            for elem in vec_elements {
                encoded_value.extend(encode_value(elem, typ)?.0);
            }
        }

        (InputValue::String(string), _) => {
            let str_as_fields =
                string.bytes().map(|byte| FieldElement::from_be_bytes_reduce(&[byte]));
            encoded_value.extend(str_as_fields);
        }

        (InputValue::Struct(object), AbiType::Struct { fields, .. }) => {
            for (field, typ) in fields {
                encoded_value.extend(encode_value(object[field].clone(), typ)?.0);
            }
        }
        (InputValue::Vec(vec_elements), AbiType::Tuple { fields }) => {
            for (value, typ) in vec_elements.into_iter().zip(fields) {
                encoded_value.extend(encode_value(value, typ)?.0);
            }
        }
        _ => unreachable!("value should have already been checked to match abi type"),
    }
    Ok((encoded_value, in_memory))
}
