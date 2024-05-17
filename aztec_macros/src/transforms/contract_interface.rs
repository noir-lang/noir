use noirc_frontend::ast::{NoirFunction, UnresolvedTypeData};
use noirc_frontend::{
    graph::CrateId,
    macros_api::{FileId, HirContext, HirExpression, HirLiteral, HirStatement},
    parse_program,
    parser::SortedModule,
    Type,
};

use crate::utils::{
    constants::SELECTOR_PLACEHOLDER,
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
//   let args_hash = dep::aztec::hash::hash_args(args_acc);
//   assert(args_hash == dep::aztec::oracle::arguments::pack_arguments(args_acc));
//   PublicCallInterface {
//     target_contract: self.target_contract,
//     selector: FunctionSelector::from_signature("SELECTOR_PLACEHOLDER"),
//     args_hash
//   }
// }
//
// The selector placeholder has to be replaced with the actual function signature after type checking in the next macro pass
pub fn stub_function(aztec_visibility: &str, func: &NoirFunction) -> String {
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

    let fn_selector = format!("dep::aztec::protocol_types::abis::function_selector::FunctionSelector::from_signature(\"{}\")", SELECTOR_PLACEHOLDER);

    let parameters = func.parameters();
    let is_void = if matches!(fn_return_type.typ, UnresolvedTypeData::Unit) { "Void" } else { "" };
    let return_type_hint = if is_void == "Void" {
        "".to_string()
    } else {
        format!("<{}>", fn_return_type.typ.to_string().replace("plain::", ""))
    };
    let call_args = parameters
        .iter()
        .map(|arg| {
            let param_name = arg.pattern.name_ident().0.contents.clone();
            match &arg.typ.typ {
                UnresolvedTypeData::Array(_, typ) => {
                    format!(
                        "let hash_{0} = {0}.map(|x: {1}| x.serialize());
                for i in 0..{0}.len() {{
                    args_acc = args_acc.append(hash_{0}[i].as_slice());
                }}\n",
                        param_name,
                        typ.typ.to_string().replace("plain::", "")
                    )
                }
                _ => {
                    format!("args_acc = args_acc.append({}.serialize().as_slice());\n", param_name)
                }
            }
        })
        .collect::<Vec<_>>()
        .join("");
    if aztec_visibility != "Avm" {
        let args_hash = if !parameters.is_empty() {
            format!(
                "let mut args_acc: [Field] = &[];
                {}
                let args_hash = dep::aztec::hash::hash_args(args_acc);
                assert(args_hash == dep::aztec::oracle::arguments::pack_arguments(args_acc));",
                call_args
            )
        } else {
            "let args_hash = 0;".to_string()
        };

        let fn_body = format!(
            "{}
                dep::aztec::context::{}{}CallInterface {{
                    target_contract: self.target_contract,
                    selector: {},
                    args_hash,
                }}",
            args_hash, aztec_visibility, is_void, fn_selector,
        );
        format!(
            "pub fn {}(self, {}) -> dep::aztec::context::{}{}CallInterface{} {{
                    {}
                }}",
            fn_name, fn_parameters, aztec_visibility, is_void, return_type_hint, fn_body
        )
    } else {
        let args = format!(
            "let mut args_acc: [Field] = &[];
            {}
            ",
            call_args
        );
        let fn_body = format!(
            "{}
            dep::aztec::context::Avm{}CallInterface {{
                target_contract: self.target_contract,
                selector: {},
                args: args_acc,
                gas_opts: dep::aztec::context::gas::GasOpts::default(),
            }}",
            args, is_void, fn_selector,
        );
        format!(
            "pub fn {}(self, {}) -> dep::aztec::context::Avm{}CallInterface{} {{
                    {}
            }}",
            fn_name, fn_parameters, is_void, return_type_hint, fn_body
        )
    }
}

// Generates the contract interface as a struct with an `at` function that holds the stubbed functions and provides
// them with a target contract address. The struct has the same name as the contract (which is technically a module)
// so imports look nice. The `at` function is also exposed as a contract library method for external use.
pub fn generate_contract_interface(
    module: &mut SortedModule,
    module_name: &str,
    stubs: &[String],
) -> Result<(), AztecMacroError> {
    let contract_interface = format!(
        "
        struct {0} {{
            target_contract: dep::aztec::protocol_types::address::AztecAddress
        }}

        impl {0} {{
            {1}

            pub fn at(
                target_contract: dep::aztec::protocol_types::address::AztecAddress
            ) -> Self {{
                Self {{ target_contract }}
            }}
        }}

        #[contract_library_method]
        pub fn at(
            target_contract: dep::aztec::protocol_types::address::AztecAddress
        ) -> {0} {{
            {0} {{ target_contract }}
        }}
    ",
        module_name,
        stubs.join("\n"),
    );

    let (contract_interface_ast, errors) = parse_program(&contract_interface);
    if !errors.is_empty() {
        dbg!(errors);
        return Err(AztecMacroError::CouldNotGenerateContractInterface { secondary_message: Some("Failed to parse Noir macro code during contract interface generation. This is either a bug in the compiler or the Noir macro code".to_string()), });
    }

    let mut contract_interface_ast = contract_interface_ast.into_sorted();
    module.types.push(contract_interface_ast.types.pop().unwrap());
    module.impls.push(contract_interface_ast.impls.pop().unwrap());
    module.functions.push(contract_interface_ast.functions.pop().unwrap());

    Ok(())
}

fn compute_fn_signature(fn_name: &str, parameters: &[Type]) -> String {
    format!(
        "{}({})",
        fn_name,
        parameters.iter().map(signature_of_type).collect::<Vec<_>>().join(",")
    )
}

// Updates the function signatures in the contract interface with the actual ones, replacing the placeholder.
// This is done by locating the contract interface struct, its functions (stubs) and assuming the last statement of each
// is the constructor for a <visibility>CallInterface. This constructor has a selector field that holds a
// FunctionSelector::from_signature function that receives the signature as a string literal.
pub fn update_fn_signatures_in_contract_interface(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    if let Some((name, _, file_id)) = get_contract_module_data(context, crate_id) {
        let maybe_interface_struct =
            collect_crate_structs(crate_id, context).iter().find_map(|struct_id| {
                let r#struct = context.def_interner.get_struct(*struct_id);
                if r#struct.borrow().name.0.contents == name {
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

                if name == "at" {
                    continue;
                }

                let fn_signature = compute_fn_signature(
                    name,
                    &fn_parameters
                        .iter()
                        .skip(1)
                        .map(|(_, typ, _)| typ.clone())
                        .collect::<Vec<Type>>(),
                );
                let hir_func = context.def_interner.function(func_id).block(&context.def_interner);
                let call_interface_constructor_statement = context.def_interner.statement(
                    hir_func
                        .statements()
                        .last()
                        .ok_or((AztecMacroError::AztecDepNotFound, file_id))?,
                );
                let call_interface_constructor_expression =
                    match call_interface_constructor_statement {
                        HirStatement::Expression(expression_id) => {
                            match context.def_interner.expression(&expression_id) {
                        HirExpression::Constructor(hir_constructor_expression) => {
                            Ok(hir_constructor_expression)
                        }
                        _ => Err((
                            AztecMacroError::CouldNotGenerateContractInterface {
                                secondary_message: Some(
                                    "CallInterface constructor statement must be a constructor expression"
                                        .to_string(),
                                ),
                            },
                            file_id,
                        )),
                    }
                        }
                        _ => Err((
                            AztecMacroError::CouldNotGenerateContractInterface {
                                secondary_message: Some(
                                    "CallInterface constructor statement must be an expression"
                                        .to_string(),
                                ),
                            },
                            file_id,
                        )),
                    }?;
                let (_, function_selector_expression_id) =
                    call_interface_constructor_expression.fields[1];
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
                    HirExpression::Literal(HirLiteral::Str(signature)) => {
                        if signature != SELECTOR_PLACEHOLDER {
                            Err((
                                AztecMacroError::CouldNotGenerateContractInterface {
                                    secondary_message: Some(format!(
                                        "Function signature argument must be a placeholder: {}",
                                        SELECTOR_PLACEHOLDER
                                    )),
                                },
                                file_id,
                            ))
                        } else {
                            Ok(())
                        }
                    }
                    _ => Err((
                        AztecMacroError::CouldNotAssignStorageSlots {
                            secondary_message: Some(
                                "Function signature argument must be a literal string".to_string(),
                            ),
                        },
                        file_id,
                    )),
                }?;

                context
                    .def_interner
                    .update_expression(current_fn_signature_expression_id, |expr| {
                        *expr = HirExpression::Literal(HirLiteral::Str(fn_signature))
                    });
            }
        }
    }
    Ok(())
}
