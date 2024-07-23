use acvm::acir::AcirField;

use noirc_errors::Location;
use noirc_frontend::ast::{Ident, NoirFunction, UnresolvedTypeData};
use noirc_frontend::{
    graph::CrateId,
    macros_api::{FieldElement, FileId, HirContext, HirExpression, HirLiteral, HirStatement},
    parse_program,
    parser::SortedModule,
    Type,
};

use tiny_keccak::{Hasher, Keccak};

use crate::utils::{
    errors::AztecMacroError,
    hir_utils::{collect_crate_structs, get_contract_module_data, signature_of_type},
};

// Generates the stubs for contract functions as low level calls using <visibility>CallInterface, turning
//  #[aztec(public)] // also private
//  fn a_function(first_arg: Field, second_arg: Struct, third_arg: [Field; 4]) -> Field {
//      ...
//  }
//
// into
//
// pub fn a_function(self, first_arg: Field, second_arg: Struct, third_arg: [Field; 4]) -> PublicCallInterface {
//   let mut args_acc: [Field] = &[];
//   args_acc = args_acc.append(first_arg.serialize().as_slice());
//   args_acc = args_acc.append(second_arg.serialize().as_slice());
//   let hash_third_arg = third_arg.map(|x: Field| x.serialize());
//   for i in 0..third_arg.len() {
//     args_acc = args_acc.append(third_arg[i].serialize().as_slice());
//   }
//   let args_hash = aztec::hash::hash_args(args_acc);
//   assert(args_hash == aztec::oracle::arguments::pack_arguments(args_acc));
//   PublicCallInterface {
//     target_contract: self.target_contract,
//     selector: FunctionSelector::from_signature("SELECTOR_PLACEHOLDER"),
//     args_hash,
//     name: "a_function",
//     args_hash,
//     args: args_acc,
//     original: | inputs: dep::aztec::context::inputs::PublicContextInputs | -> Field {
//         a_function(inputs, first_arg, second_arg, third_arg)
//     },
//     is_static: false,
//     gas_opts: dep::aztec::context::gas::GasOpts::default()
//   }
// }
//
// The selector placeholder has to be replaced with the actual function signature after type checking in the next macro pass
pub fn stub_function(aztec_visibility: &str, func: &NoirFunction, is_static_call: bool) -> String {
    let fn_name = func.name().to_string();
    let fn_parameters = func
        .parameters()
        .iter()
        .map(|param| {
            format!(
                "{}: {}",
                param.pattern.name_ident().0.contents,
                param.typ.to_string().replace("plain::", "")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let fn_return_type: noirc_frontend::ast::UnresolvedType = func.return_type();

    let parameters = func.parameters();
    let is_void = if matches!(fn_return_type.typ, UnresolvedTypeData::Unit) { "Void" } else { "" };
    let is_static = if is_static_call { "Static" } else { "" };
    let return_type_hint = fn_return_type.typ.to_string().replace("plain::", "");
    let call_args = parameters
        .iter()
        .map(|arg| {
            let param_name = arg.pattern.name_ident().0.contents.clone();
            match &arg.typ.typ {
                UnresolvedTypeData::Array(_, typ) => {
                    format!(
                        "let serialized_{0} = {0}.map(|x: {1}| x.serialize());
                        for i in 0..{0}.len() {{
                            args_acc = args_acc.append(serialized_{0}[i].as_slice());
                        }}\n",
                        param_name,
                        typ.typ.to_string().replace("plain::", "")
                    )
                }
                UnresolvedTypeData::Named(_, _, _) | UnresolvedTypeData::String(_) => {
                    format!("args_acc = args_acc.append({}.serialize().as_slice());\n", param_name)
                }
                _ => {
                    format!("args_acc = args_acc.append(&[{}.to_field()]);\n", param_name)
                }
            }
        })
        .collect::<Vec<_>>()
        .join("");

    let param_types = if !parameters.is_empty() {
        parameters
            .iter()
            .map(|param| param.pattern.name_ident().0.contents.clone())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "".to_string()
    };

    let original = format!(
        "| inputs: dep::aztec::context::inputs::{}ContextInputs | -> {} {{
            {}(inputs{})
        }}",
        aztec_visibility,
        if aztec_visibility == "Private" {
            "dep::aztec::protocol_types::abis::private_circuit_public_inputs::PrivateCircuitPublicInputs".to_string()
        } else {
            return_type_hint.clone()
        },
        fn_name,
        if param_types.is_empty() { "".to_string() } else { format!(" ,{}  ", param_types) }
    );
    let arg_types = format!(
        "({}{})",
        parameters
            .iter()
            .map(|param| param.typ.typ.to_string().replace("plain::", ""))
            .collect::<Vec<_>>()
            .join(","),
        // In order to distinguish between a single element Tuple (Type,) and a single type with unnecessary parenthesis around it (Type),
        // The latter gets simplified to Type, that is NOT a valid env
        if parameters.len() == 1 { "," } else { "" }
    );

    let generics = if is_void == "Void" {
        format!("{}>", arg_types)
    } else {
        format!("{}, {}>", return_type_hint, arg_types)
    };

    let args = format!(
        "let mut args_acc: [Field] = &[];
        {}
        {}",
        call_args,
        if aztec_visibility == "Private" {
            "let args_hash = aztec::hash::hash_args(args_acc);"
        } else {
            ""
        }
    );

    let gas_opts = if aztec_visibility == "Public" {
        "gas_opts: dep::aztec::context::gas::GasOpts::default()"
    } else {
        ""
    };

    let fn_body = format!(
        "{}
            let selector = dep::aztec::protocol_types::abis::function_selector::FunctionSelector::from_field(0);
            dep::aztec::context::{}{}{}CallInterface {{
                target_contract: self.target_contract,
                selector,
                name: \"{}\",
                {}
                args: args_acc,
                original: {},
                is_static: {},
                {}
            }}",
        args,
        aztec_visibility,
        is_static,
        is_void,
        fn_name,
        if aztec_visibility == "Private" { "args_hash," } else { "" },
        original,
        is_static_call,
        gas_opts
    );

    format!(
        "pub fn {}(self, {}) -> dep::aztec::context::{}{}{}CallInterface<{},{} {{
                {}
        }}",
        fn_name,
        fn_parameters,
        aztec_visibility,
        is_static,
        is_void,
        fn_name.len(),
        generics,
        fn_body
    )
}

// Generates the contract interface as a struct with an `at` function that holds the stubbed functions and provides
// them with a target contract address. The struct has the same name as the contract (which is technically a module)
// so imports look nice. The `at` function is also exposed as a contract library method for external use.
pub fn generate_contract_interface(
    module: &mut SortedModule,
    module_name: &str,
    stubs: &[(String, Location)],
    has_storage_layout: bool,
) -> Result<(), AztecMacroError> {
    let storage_layout_getter = format!(
        "#[contract_library_method]
        pub fn storage() -> StorageLayout {{
            {}_STORAGE_LAYOUT
        }}",
        module_name,
    );
    let contract_interface = format!(
        "
        struct {0} {{
            target_contract: aztec::protocol_types::address::AztecAddress
        }}

        impl {0} {{
            {1}

            pub fn at(
                target_contract: aztec::protocol_types::address::AztecAddress
            ) -> Self {{
                Self {{ target_contract }}
            }}

            pub fn interface() -> Self {{
                Self {{ target_contract: dep::aztec::protocol_types::address::AztecAddress::zero() }}
            }}

            {2}
        }}

        #[contract_library_method]
        pub fn at(
            target_contract: aztec::protocol_types::address::AztecAddress
        ) -> {0} {{
            {0} {{ target_contract }}
        }}

        #[contract_library_method]
        pub fn interface() -> {0} {{
            {0} {{ target_contract: dep::aztec::protocol_types::address::AztecAddress::zero() }}
        }}

        {3}
    ",
        module_name,
        stubs.iter().map(|(src, _)| src.to_owned()).collect::<Vec<String>>().join("\n"),
        if has_storage_layout { storage_layout_getter.clone() } else { "".to_string() },
        if has_storage_layout { format!("#[contract_library_method]\n{}", storage_layout_getter) } else { "".to_string() } 
    );

    let (contract_interface_ast, errors) = parse_program(&contract_interface);
    if !errors.is_empty() {
        dbg!(errors);
        return Err(AztecMacroError::CouldNotGenerateContractInterface { secondary_message: Some("Failed to parse Noir macro code during contract interface generation. This is either a bug in the compiler or the Noir macro code".to_string()), });
    }

    let mut contract_interface_ast = contract_interface_ast.into_sorted();
    let mut impl_with_locations = contract_interface_ast.impls.pop().unwrap();

    impl_with_locations.methods = impl_with_locations
        .methods
        .iter()
        .enumerate()
        .map(|(i, (method, orig_span))| {
            if method.name() == "at" || method.name() == "interface" || method.name() == "storage" {
                (method.clone(), *orig_span)
            } else {
                let (_, new_location) = stubs[i];
                let mut modified_method = method.clone();
                modified_method.def.name =
                    Ident::new(modified_method.name().to_string(), new_location.span);
                (modified_method, *orig_span)
            }
        })
        .collect();

    module.types.push(contract_interface_ast.types.pop().unwrap());
    module.impls.push(impl_with_locations);
    for function in contract_interface_ast.functions {
        module.functions.push(function);
    }

    Ok(())
}

fn compute_fn_signature_hash(fn_name: &str, parameters: &[Type]) -> u32 {
    let signature = format!(
        "{}({})",
        fn_name,
        parameters.iter().map(signature_of_type).collect::<Vec<_>>().join(",")
    );
    let mut keccak = Keccak::v256();
    let mut result = [0u8; 32];
    keccak.update(signature.as_bytes());
    keccak.finalize(&mut result);
    // Take the first 4 bytes of the hash and convert them to an integer
    // If you change the following value you have to change NUM_BYTES_PER_NOTE_TYPE_ID in l1_note_payload.ts as well
    let num_bytes_per_note_type_id = 4;
    u32::from_be_bytes(result[0..num_bytes_per_note_type_id].try_into().unwrap())
}

// Updates the function signatures in the contract interface with the actual ones, replacing the placeholder.
// This is done by locating the contract interface struct, its functions (stubs) and assuming the second to last statement of each
// is a let statement initializing the selector with a FunctionSelector::from_field call.
pub fn update_fn_signatures_in_contract_interface(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    if let Some((struct_name, _, file_id)) = get_contract_module_data(context, crate_id) {
        let maybe_interface_struct =
            collect_crate_structs(crate_id, context).iter().find_map(|struct_id| {
                let r#struct = context.def_interner.get_struct(*struct_id);
                if r#struct.borrow().name.0.contents == struct_name {
                    Some(r#struct)
                } else {
                    None
                }
            });

        if let Some(interface_struct) = maybe_interface_struct {
            let methods = context.def_interner.get_struct_methods(interface_struct.borrow().id);

            for func_id in methods.iter().flat_map(|methods| methods.direct.iter()) {
                let name = context.def_interner.function_name(func_id);
                let fn_parameters = &context.def_interner.function_meta(func_id).parameters.clone();

                if name == "at" || name == "interface" || name == "storage" {
                    continue;
                }

                let fn_signature_hash = compute_fn_signature_hash(
                    name,
                    &fn_parameters
                        .iter()
                        .skip(1)
                        .map(|(_, typ, _)| typ.clone())
                        .collect::<Vec<Type>>(),
                );
                let hir_func = context.def_interner.function(func_id).block(&context.def_interner);

                let function_selector_statement = context.def_interner.statement(
                    hir_func.statements().get(hir_func.statements().len() - 2).ok_or((
                        AztecMacroError::CouldNotGenerateContractInterface {
                            secondary_message: Some(
                                "Function signature statement not found, invalid body length"
                                    .to_string(),
                            ),
                        },
                        file_id,
                    ))?,
                );
                let function_selector_expression_id = match function_selector_statement {
                    HirStatement::Let(let_statement) => Ok(let_statement.expression),
                    _ => Err((
                        AztecMacroError::CouldNotGenerateContractInterface {
                            secondary_message: Some(
                                "Function selector statement must be an expression".to_string(),
                            ),
                        },
                        file_id,
                    )),
                }?;
                let function_selector_expression =
                    context.def_interner.expression(&function_selector_expression_id);

                let current_fn_signature_expression_id = match function_selector_expression {
                    HirExpression::Call(call_expr) => Ok(call_expr.arguments[0]),
                    _ => Err((
                        AztecMacroError::CouldNotGenerateContractInterface {
                            secondary_message: Some(
                                "Function selector argument expression must be call expression"
                                    .to_string(),
                            ),
                        },
                        file_id,
                    )),
                }?;

                let current_fn_signature_expression =
                    context.def_interner.expression(&current_fn_signature_expression_id);

                match current_fn_signature_expression {
                    HirExpression::Literal(HirLiteral::Integer(value, _)) => {
                        if !value.is_zero() {
                            Err((
                                AztecMacroError::CouldNotGenerateContractInterface {
                                    secondary_message: Some(
                                        "Function signature argument must be a placeholder with value 0".to_string()),
                                },
                                file_id,
                            ))
                        } else {
                            Ok(())
                        }
                    }
                    _ => Err((
                        AztecMacroError::CouldNotGenerateContractInterface {
                            secondary_message: Some(
                                "Function signature argument must be a literal field element"
                                    .to_string(),
                            ),
                        },
                        file_id,
                    )),
                }?;

                context.def_interner.update_expression(
                    current_fn_signature_expression_id,
                    |expr| {
                        *expr = HirExpression::Literal(HirLiteral::Integer(
                            FieldElement::from(fn_signature_hash as u128),
                            false,
                        ))
                    },
                );
            }
        }
    }
    Ok(())
}
