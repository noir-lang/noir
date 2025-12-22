use std::collections::BTreeMap;
use std::rc::Rc;

use acvm::AcirField;
use fm::FileManager;
use iter_extended::vecmap;
use nargo::ops::report_errors;
use nargo::package::Package;
use nargo::workspace::Workspace;
use noirc_abi::InputMap;
use noirc_abi::input_parser::InputValue;
use noirc_driver::gen_abi;
use noirc_errors::{CustomDiagnostic, Location};
use noirc_evaluator::ssa::interpreter::value::{Fitted, NumericValue};
use noirc_evaluator::ssa::ir::types::NumericType;
use noirc_frontend::hir::comptime::Value;
use noirc_frontend::hir::def_collector::dc_crate::CompilationError;
use noirc_frontend::hir::{Context, ParsedFiles};
use noirc_frontend::hir_def::function::FuncMeta;
use noirc_frontend::hir_def::stmt::HirPattern;
use noirc_frontend::node_interner::NodeInterner;
use noirc_frontend::shared::Signedness;
use noirc_frontend::signed_field::SignedField;
use noirc_frontend::{Shared, Type};

use crate::cli::compile_cmd::parse_workspace;
use crate::cli::execute_cmd::ExecuteCommand;
use crate::errors::CliError;

pub(super) fn run_comptime(args: ExecuteCommand, workspace: Workspace) -> Result<(), CliError> {
    let (file_manager, parsed_files) = parse_workspace(&workspace, None);
    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());

    for package in binary_packages {
        run_package_comptime(package, &args, &workspace, &file_manager, &parsed_files)?;
    }

    Ok(())
}

fn run_package_comptime(
    package: &Package,
    args: &ExecuteCommand,
    workspace: &Workspace,
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
) -> Result<(), CliError> {
    let (mut context, crate_id) = nargo::prepare_package(file_manager, parsed_files, package);
    context.package_build_path = workspace.package_build_path(package);
    noirc_driver::link_to_debug_crate(&mut context, crate_id);
    let result = noirc_driver::check_crate(&mut context, crate_id, &args.compile_options);
    match report_errors(
        result,
        &context.file_manager,
        args.compile_options.deny_warnings,
        args.compile_options.silence_warnings,
    ) {
        Ok(()) => {}
        Err(err) => {
            return Err(CliError::CompileError(err));
        }
    }

    let Some(main_id) = context.get_main_function(&crate_id) else {
        return Err(CliError::Generic("Could not find main function".to_string()));
    };

    let func_meta = context.def_interner.function_meta(&main_id);

    let error_types = BTreeMap::default();
    let abi = gen_abi(&context, &main_id, func_meta.return_visibility, error_types);

    // Parse the inputs and convert them to what the interpreter expects.
    let prover_file = package.root_dir.join(&args.prover_name).with_extension("toml");
    let (prover_input, return_value) =
        noir_artifact_cli::fs::inputs::read_inputs_from_file(&prover_file, &abi)?;

    let location = func_meta.location;
    let func_args =
        input_values_to_comptime_values(&prover_input, func_meta, &context.def_interner);
    let return_value = return_value.map(|return_value| {
        input_value_to_comptime_value(&return_value, func_meta.return_type(), location)
    });

    match context.interpret_function(main_id, func_args) {
        Ok(result) => {
            let result_as_string = output_value_to_string(&result, &context);

            println!("[{}] Circuit witness successfully solved", package.name);
            if !matches!(result, Value::Unit) {
                println!("[{}] Circuit output: {result_as_string}", package.name);
            }

            if let Some(return_value) = return_value {
                if result != return_value {
                    let return_value_as_string =
                        return_value.display(&context.def_interner).to_string();
                    return Err(CliError::Generic(format!(
                        "Unexpected return value.\nExpected: {return_value_as_string}\nGot:      {result_as_string}"
                    )));
                }
            }

            Ok(())
        }
        Err(err) => {
            let err = CompilationError::InterpreterError(err);
            let diagnostic = CustomDiagnostic::from(&err);
            let errors = vec![diagnostic];
            report_errors(
                Ok(((), errors)),
                file_manager,
                args.compile_options.deny_warnings,
                args.compile_options.silence_warnings,
            )?;
            Err(CliError::Generic("Error interpreting main function".to_string()))
        }
    }
}

fn input_values_to_comptime_values(
    prover_input: &InputMap,
    func_meta: &FuncMeta,
    interner: &NodeInterner,
) -> Vec<(Value, Location)> {
    vecmap(func_meta.parameters.iter(), |(pattern, typ, _visibility)| {
        let location = pattern.location();
        let (pattern, typ) = if let HirPattern::Mutable(pattern, _) = pattern {
            (*pattern.clone(), typ.clone())
        } else {
            (pattern.clone(), typ.clone())
        };
        let HirPattern::Identifier(ident) = pattern else {
            panic!("only identifier patterns are supported in main");
        };
        let name = interner.definition_name(ident.id);
        let input = prover_input
            .get(name)
            .unwrap_or_else(|| panic!("Expected to find {name} in prover inputs"));
        let value = input_value_to_comptime_value(input, &typ, location);
        (value, location)
    })
}

fn input_value_to_comptime_value(input: &InputValue, typ: &Type, location: Location) -> Value {
    match typ {
        Type::Unit => Value::Unit,
        Type::Bool => {
            let InputValue::Field(value) = input else {
                panic!("expected field input for bool type");
            };
            Value::Bool(!value.is_zero())
        }
        Type::Integer(signedness, bit_size) => {
            let InputValue::Field(value) = input else {
                panic!("expected field input for bool type");
            };

            // Here we reuse the logic of converting an input into an SSA numeric value,
            // which is not ideal but avoids duplicating the conversion logic.
            let bit_size = u32::from(bit_size.bit_size());
            let numeric_type = match signedness {
                Signedness::Unsigned => NumericType::Unsigned { bit_size },
                Signedness::Signed => NumericType::Signed { bit_size },
            };
            let numeric_value = NumericValue::from_constant(*value, numeric_type)
                .expect("Could not convert field value to integer");
            match numeric_value {
                NumericValue::Field(_) => panic!("Field should not happen here"),
                NumericValue::U1(value) => Value::U1(value),
                NumericValue::U8(fitted) => match fitted {
                    Fitted::Fit(value) => Value::U8(value),
                    Fitted::Unfit(..) => panic!("input value does not fit in u8"),
                },
                NumericValue::U16(fitted) => match fitted {
                    Fitted::Fit(value) => Value::U16(value),
                    Fitted::Unfit(..) => panic!("input value does not fit in u16"),
                },
                NumericValue::U32(fitted) => match fitted {
                    Fitted::Fit(value) => Value::U32(value),
                    Fitted::Unfit(..) => panic!("input value does not fit in u32"),
                },
                NumericValue::U64(fitted) => match fitted {
                    Fitted::Fit(value) => Value::U64(value),
                    Fitted::Unfit(..) => panic!("input value does not fit in u64"),
                },
                NumericValue::U128(fitted) => match fitted {
                    Fitted::Fit(value) => Value::U128(value),
                    Fitted::Unfit(..) => panic!("input value does not fit in u128"),
                },
                NumericValue::I8(fitted) => match fitted {
                    Fitted::Fit(value) => Value::I8(value),
                    Fitted::Unfit(..) => panic!("input value does not fit in i8"),
                },
                NumericValue::I16(fitted) => match fitted {
                    Fitted::Fit(value) => Value::I16(value),
                    Fitted::Unfit(..) => panic!("input value does not fit in i16"),
                },
                NumericValue::I32(fitted) => match fitted {
                    Fitted::Fit(value) => Value::I32(value),
                    Fitted::Unfit(..) => panic!("input value does not fit in i32"),
                },
                NumericValue::I64(fitted) => match fitted {
                    Fitted::Fit(value) => Value::I64(value),
                    Fitted::Unfit(..) => panic!("input value does not fit in i64"),
                },
            }
        }
        Type::FieldElement => {
            let InputValue::Field(value) = input else {
                panic!("expected field input for field element type");
            };
            Value::Field(SignedField::positive(*value))
        }
        Type::Array(length, element_typ) => {
            let length =
                length.evaluate_to_u32(location).expect("Could not evaluate array length to u32");
            let InputValue::Vec(inputs) = input else {
                panic!("expected vec input for array type");
            };
            assert_eq!(inputs.len(), length as usize, "Array length does not match input length");
            let array = inputs
                .iter()
                .map(|input| input_value_to_comptime_value(input, element_typ, location))
                .collect();
            Value::Array(array, typ.clone())
        }
        Type::String(length) => {
            let InputValue::String(string) = input else {
                panic!("expected string input for string type");
            };
            let length =
                length.evaluate_to_u32(location).expect("Could not evaluate string length to u32");
            assert_eq!(string.len(), length as usize, "String length does not match input length");
            Value::String(Rc::new(string.clone()))
        }
        Type::Tuple(types) => {
            let InputValue::Vec(inputs) = input else {
                panic!("expected vec input for tuple type");
            };
            assert_eq!(inputs.len(), types.len(), "Tuple length does not match input length");
            let tuple = vecmap(inputs.iter().zip(types.iter()), |(input, typ)| {
                let value = input_value_to_comptime_value(input, typ, location);
                Shared::new(value)
            });
            Value::Tuple(tuple)
        }
        Type::DataType(data_type, generics) => {
            let fields = data_type
                .borrow()
                .get_fields(generics)
                .expect("Enums as inputs are not yet supported");
            let InputValue::Struct(inputs) = input else {
                panic!("expected struct input for data type");
            };
            let fields = fields
                .into_iter()
                .map(|(name, typ, _)| {
                    let input = inputs
                        .get(&name)
                        .unwrap_or_else(|| panic!("Expected to find field {name} in input"));
                    let value = input_value_to_comptime_value(input, &typ, location);
                    (Rc::new(name), Shared::new(value))
                })
                .collect();
            Value::Struct(fields, typ.clone())
        }
        Type::Alias(alias, generics) => {
            let typ = alias.borrow().get_type(generics);
            input_value_to_comptime_value(input, &typ, location)
        }
        Type::Vector(_)
        | Type::FmtString(_, _)
        | Type::TypeVariable(..)
        | Type::TraitAsType(..)
        | Type::NamedGeneric(..)
        | Type::CheckedCast { .. }
        | Type::Function(..)
        | Type::Reference(_, _)
        | Type::Forall(..)
        | Type::Constant(..)
        | Type::Quoted(..)
        | Type::InfixExpr(..)
        | Type::Error => panic!("Unexpected type in comptime input value conversion"),
    }
}

/// Converts a [Value] into a `String`.
///
/// This is similar to `Value::display(..).to_string()` except that:
/// - only values that can be a circuit output are supported
/// - strings are quoted
/// - struct paths are fully-qualified
///
/// This is so the output matches the format produced by `nargo execute`.
fn output_value_to_string(value: &Value, context: &Context) -> String {
    match value {
        Value::Unit => "()".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Field(signed_field) => signed_field.to_field_element().to_short_hex(),
        Value::I8(value) => value.to_string(),
        Value::I16(value) => value.to_string(),
        Value::I32(value) => value.to_string(),
        Value::I64(value) => value.to_string(),
        Value::U1(false) => "0".to_string(),
        Value::U1(true) => "1".to_string(),
        Value::U8(value) => value.to_string(),
        Value::U16(value) => value.to_string(),
        Value::U32(value) => value.to_string(),
        Value::U64(value) => value.to_string(),
        Value::U128(value) => value.to_string(),
        Value::String(string) | Value::FormatString(string, _) | Value::CtString(string) => {
            format!("{string:?}")
        }
        Value::Tuple(values) => {
            let values = vecmap(values, |value| output_value_to_string(&value.borrow(), context));
            if values.len() == 1 {
                format!("({},)", values[0])
            } else {
                format!("({})", values.join(", "))
            }
        }
        Value::Array(values, _) => {
            let values = vecmap(values, |value| output_value_to_string(value, context));
            format!("[{}]", values.join(", "))
        }
        Value::Struct(fields, typ) => {
            let data_type = match typ.follow_bindings() {
                Type::DataType(def, _) => def,
                other => unreachable!("Expected data type, found {other}"),
            };
            let data_type = data_type.borrow();
            let typename =
                context.fully_qualified_struct_path(context.root_crate_id(), data_type.id);

            // Display fields in the order they are defined in the struct.
            // Some fields might not be there if they were missing in the constructor.
            let fields = data_type
                .fields_raw()
                .unwrap()
                .iter()
                .filter_map(|field| {
                    let name = field.name.as_string();
                    fields.get(name).map(|value| {
                        format!("{}: {}", name, output_value_to_string(&value.borrow(), context))
                    })
                })
                .collect::<Vec<_>>();
            format!("{typename} {{ {} }}", fields.join(", "))
        }
        Value::Enum(tag, args, typ) => {
            let args = vecmap(args, |arg| output_value_to_string(arg, context)).join(", ");

            match typ.follow_bindings_shallow().as_ref() {
                Type::DataType(def, _) => {
                    let def = def.borrow();
                    let variant = def.variant_at(*tag);
                    if variant.is_function {
                        format!("{}::{}({args})", def.name, variant.name)
                    } else {
                        format!("{}::{}", def.name, variant.name)
                    }
                }
                other => panic!("Expected a data type, got {other}"),
            }
        }
        Value::Function(..)
        | Value::Closure(..)
        | Value::Pointer(..)
        | Value::Vector(..)
        | Value::Quoted(..)
        | Value::TypeDefinition(..)
        | Value::TraitConstraint(..)
        | Value::TraitDefinition(..)
        | Value::TraitImpl(..)
        | Value::FunctionDefinition(..)
        | Value::ModuleDefinition(..)
        | Value::Type(_)
        | Value::Zeroed(_)
        | Value::Expr(..)
        | Value::TypedExpr(..)
        | Value::UnresolvedType(..) => {
            panic!("Unexpected output value: {}", value.display(&context.def_interner))
        }
    }
}
