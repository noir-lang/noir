use std::{collections::BTreeMap, path::PathBuf};

use clap::Args;
use color_eyre::eyre::{self, Context, bail};
use iter_extended::vecmap;
use noir_artifact_cli::{commands::parse_and_normalize_path, fs::artifact::write_to_file};
use noirc_abi::{
    Abi, AbiParameter, AbiReturnType, AbiType, AbiVisibility, InputMap, Sign,
    input_parser::InputValue,
};
use noirc_errors::println_to_stdout;
use noirc_evaluator::ssa::{
    interpreter::InterpreterOptions,
    ir::types::{NumericType, Type},
    ssa_gen::Ssa,
};
use tempfile::NamedTempFile;

const TOML_LINE_SEP: char = ';';

/// Parse the input SSA and it arguments, run the SSA interpreter,
/// then write the return values to stdout.
#[derive(Debug, Clone, Args)]
pub(super) struct InterpretCommand {
    /// Path to the input arguments to the SSA interpreter.
    ///
    /// Expected to be in TOML format or JSON, similar to `Prover.toml`.
    ///
    /// If empty, we assume the SSA has no arguments.
    #[clap(long, short, value_parser = parse_and_normalize_path, conflicts_with = "input_json", conflicts_with = "input_toml")]
    pub input_path: Option<PathBuf>,

    /// Verbatim inputs in JSON format.
    #[clap(long, conflicts_with = "input_path", conflicts_with = "input_toml")]
    pub input_json: Option<String>,

    /// Verbatim inputs in TOML format.
    ///
    /// Use ';' to separate what would normally be multiple lines.
    #[clap(long, conflicts_with = "input_path", conflicts_with = "input_json")]
    pub input_toml: Option<String>,

    /// Turn on tracing in the SSA interpreter.
    #[clap(long, default_value_t = false)]
    pub trace: bool,

    /// Optional limit for the interpreter.
    #[clap(long)]
    pub step_limit: Option<usize>,
}

pub(super) fn run(args: InterpretCommand, ssa: Ssa) -> eyre::Result<()> {
    // Construct an ABI, which we can then use to parse input values.
    let abi = abi_from_ssa(&ssa);

    let options =
        InterpreterOptions { trace: args.trace, step_limit: args.step_limit, ..Default::default() };

    let (input_map, return_value) = read_inputs_and_return(&abi, &args)?;
    let ssa_args = noir_ast_fuzzer::input_values_to_ssa(&abi, &input_map);

    let ssa_return =
        if let (Some(return_type), Some(return_value)) = (&abi.return_type, return_value) {
            Some(noir_ast_fuzzer::input_value_to_ssa(&return_type.abi_type, &return_value))
        } else {
            None
        };

    let result = ssa.interpret_with_options(ssa_args, options, std::io::stdout());

    // Mimicking the way `nargo interpret` presents its results.
    match &result {
        Ok(value) => {
            let value_as_string = vecmap(value, ToString::to_string).join(", ");
            println_to_stdout!("--- Interpreter result:\nOk({value_as_string})\n---");
        }
        Err(err) => {
            println_to_stdout!("--- Interpreter result:\nErr({err})\n---");
        }
    }
    let is_ok = result.is_ok();

    if let Some(return_value) = ssa_return {
        let return_value_as_string = vecmap(&return_value, ToString::to_string).join(", ");
        let Ok(result) = result else {
            bail!(
                "Interpreter produced an unexpected error.\nExpected result: {return_value_as_string}"
            );
        };
        if return_value != result {
            let result_as_string = vecmap(&result, ToString::to_string).join(", ");
            bail!(
                "Interpreter produced an unexpected result.\nExpected result: {return_value_as_string}\nActual result: {result_as_string}"
            )
        }
    }

    if is_ok { Ok(()) } else { bail!("The interpreter encountered an error.") }
}

/// Derive an ABI description from the SSA parameters.
fn abi_from_ssa(ssa: &Ssa) -> Abi {
    let main = &ssa.functions[&ssa.main_id];

    // We ignore visibility and treat everything as public, because visibility
    // is only available in the Program with the monomorphized AST from which
    // we normally generate the SSA. The SSA itself doesn't carry information
    // about the databus, for example.
    let visibility = AbiVisibility::Public;

    let parameters = main
        .view()
        .parameter_types()
        .iter()
        .enumerate()
        .map(|(i, typ)| AbiParameter {
            name: format!("v{i}"),
            typ: abi_type_from_ssa(typ),
            visibility,
        })
        .collect();

    let return_type = main
        .view()
        .return_types()
        .filter(|ts| !ts.is_empty())
        .map(|types| AbiReturnType { abi_type: abi_type_from_multi_ssa(&types), visibility });

    Abi { parameters, return_type, error_types: Default::default() }
}

/// Create an ABI type from multiple SSA types, for example when multiple values are returned, or appear in arrays.
fn abi_type_from_multi_ssa(types: &[Type]) -> AbiType {
    match types.len() {
        0 => unreachable!("cannot construct ABI type from 0 types"),
        1 => abi_type_from_ssa(&types[0]),
        _ => AbiType::Tuple { fields: vecmap(types, abi_type_from_ssa) },
    }
}

/// Create an ABI type from a single SSA type.
fn abi_type_from_ssa(typ: &Type) -> AbiType {
    match typ {
        Type::Numeric(numeric_type) => match numeric_type {
            NumericType::NativeField => AbiType::Field,
            NumericType::Unsigned { bit_size: 1 } => AbiType::Boolean,
            NumericType::Unsigned { bit_size } => {
                AbiType::Integer { sign: Sign::Unsigned, width: *bit_size }
            }
            NumericType::Signed { bit_size } => {
                AbiType::Integer { sign: Sign::Signed, width: *bit_size }
            }
        },
        Type::Array(items, length) => {
            AbiType::Array { length: length.0, typ: Box::new(abi_type_from_multi_ssa(items)) }
        }
        Type::Reference(_) => unreachable!("refs do not appear in SSA ABI"),
        Type::Function => unreachable!("functions do not appear in SSA ABI"),
        Type::Vector(_) => unreachable!("vectors do not appear in SSA ABI"),
    }
}

fn write_to_temp_file(content: &str, extension: &str) -> eyre::Result<NamedTempFile> {
    let tmp = NamedTempFile::with_suffix(format!("ssa.input.{extension}"))?;

    write_to_file(content.as_bytes(), tmp.path()).wrap_err_with(|| {
        format!("failed to write {extension} to temp file at {}", tmp.path().to_string_lossy())
    })?;

    Ok(tmp)
}

fn read_inputs_and_return(
    abi: &Abi,
    args: &InterpretCommand,
) -> eyre::Result<(InputMap, Option<InputValue>)> {
    let (input_path, _guard) = if let Some(ref json) = args.input_json {
        let tmp = write_to_temp_file(json, "json")?;
        (Some(tmp.path().to_path_buf()), Some(tmp))
    } else if let Some(ref toml) = args.input_toml {
        // Split along the line separator and rejoin into a file.
        let lines = toml.split(TOML_LINE_SEP).map(|s| s.trim_start()).collect::<Vec<_>>();
        let toml = lines.join("\n");
        let tmp = write_to_temp_file(&toml, "toml")?;
        (Some(tmp.path().to_path_buf()), Some(tmp))
    } else {
        (args.input_path.clone(), None)
    };

    let (input_map, return_value) = match input_path {
        Some(path) => noir_artifact_cli::fs::inputs::read_inputs_from_file(&path, abi)
            .wrap_err_with(|| format!("failed to read inputs from {}", path.to_string_lossy()))?,
        None => (BTreeMap::default(), None),
    };

    Ok((input_map, return_value))
}
