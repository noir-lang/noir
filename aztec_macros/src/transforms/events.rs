use noirc_frontend::ast::{ItemVisibility, NoirFunction, NoirTraitImpl, TraitImplItem};
use noirc_frontend::macros_api::{NodeInterner, StructId};
use noirc_frontend::token::SecondaryAttribute;
use noirc_frontend::{
    graph::CrateId,
    macros_api::{FileId, HirContext},
    parse_program,
    parser::SortedModule,
};

use crate::utils::hir_utils::collect_crate_structs;
use crate::utils::{ast_utils::is_custom_attribute, errors::AztecMacroError};

// Automatic implementation of most of the methods in the EventInterface trait, guiding the user with meaningful error messages in case some
// methods must be implemented manually.
pub fn generate_event_impls(module: &mut SortedModule) -> Result<(), AztecMacroError> {
    // Find structs annotated with #[aztec(event)]
    // Why doesn't this work ? Events are not tagged and do not appear, it seems only going through the submodule works
    // let annotated_event_structs = module
    //     .types
    //     .iter_mut()
    //     .filter(|typ| typ.attributes.iter().any(|attr: &SecondaryAttribute| is_custom_attribute(attr, "aztec(event)")));
    // This did not work because I needed the submodule itself to add the trait impl back in to, but it would be nice if it was tagged on the module level
    // let mut annotated_event_structs = module.submodules.iter_mut()
    //     .flat_map(|submodule| submodule.contents.types.iter_mut())
    //     .filter(|typ| typ.attributes.iter().any(|attr| is_custom_attribute(attr, "aztec(event)")));

    // To diagnose
    // let test = module.types.iter_mut();
    // for event_struct in test {
    //     print!("\ngenerate_event_interface_impl COUNT: {}\n", event_struct.name.0.contents);
    // }

    for submodule in module.submodules.iter_mut() {
        let annotated_event_structs = submodule.contents.types.iter_mut().filter(|typ| {
            typ.attributes.iter().any(|attr| is_custom_attribute(attr, "aztec(event)"))
        });

        for event_struct in annotated_event_structs {
            // event_struct.attributes.push(SecondaryAttribute::Abi("events".to_string()));
            // If one impl is pushed, this doesn't throw the "#[abi(tag)] attributes can only be used in contracts" error
            // But if more than one impl is pushed, we get an increasing amount of "#[abi(tag)] attributes can only be used in contracts" errors
            // We work around this by doing this addition in the HIR pass via transform_event_abi below.

            let event_type = event_struct.name.0.contents.to_string();
            let event_len = event_struct.fields.len() as u32;
            // event_byte_len = event fields * 32 + randomness (32) + event_type_id (32)
            let event_byte_len = event_len * 32 + 64;

            let mut event_fields = vec![];

            for (field_ident, field_type) in event_struct.fields.iter() {
                event_fields.push((
                    field_ident.0.contents.to_string(),
                    field_type.typ.to_string().replace("plain::", ""),
                ));
            }

            let mut event_interface_trait_impl =
                generate_trait_impl_stub_event_interface(event_type.as_str(), event_byte_len)?;
            event_interface_trait_impl.items.push(TraitImplItem::Function(
                generate_fn_get_event_type_id(event_type.as_str(), event_len)?,
            ));
            event_interface_trait_impl.items.push(TraitImplItem::Function(
                generate_fn_private_to_be_bytes(event_type.as_str(), event_byte_len)?,
            ));
            event_interface_trait_impl.items.push(TraitImplItem::Function(
                generate_fn_to_be_bytes(event_type.as_str(), event_byte_len)?,
            ));
            event_interface_trait_impl
                .items
                .push(TraitImplItem::Function(generate_fn_emit(event_type.as_str())?));
            submodule.contents.trait_impls.push(event_interface_trait_impl);

            let serialize_trait_impl =
                generate_trait_impl_serialize(event_type.as_str(), event_len, &event_fields)?;
            submodule.contents.trait_impls.push(serialize_trait_impl);

            let deserialize_trait_impl =
                generate_trait_impl_deserialize(event_type.as_str(), event_len, &event_fields)?;
            submodule.contents.trait_impls.push(deserialize_trait_impl);
        }
    }

    Ok(())
}

fn generate_trait_impl_stub_event_interface(
    event_type: &str,
    byte_length: u32,
) -> Result<NoirTraitImpl, AztecMacroError> {
    let byte_length_without_randomness = byte_length - 32;
    let trait_impl_source = format!(
        "
impl dep::aztec::event::event_interface::EventInterface<{byte_length}, {byte_length_without_randomness}> for {event_type} {{
    }}
    "
    )
    .to_string();

    let (parsed_ast, errors) = parse_program(&trait_impl_source);
    if !errors.is_empty() {
        dbg!(errors);
        return Err(AztecMacroError::CouldNotImplementEventInterface {
            secondary_message: Some(format!("Failed to parse Noir macro code (trait impl of {event_type} for EventInterface). This is either a bug in the compiler or the Noir macro code")),
        });
    }

    let mut sorted_ast = parsed_ast.into_sorted();
    let event_interface_impl = sorted_ast.trait_impls.remove(0);

    Ok(event_interface_impl)
}

fn generate_trait_impl_serialize(
    event_type: &str,
    event_len: u32,
    event_fields: &[(String, String)],
) -> Result<NoirTraitImpl, AztecMacroError> {
    let field_names = event_fields
        .iter()
        .map(|field| {
            let field_type = field.1.as_str();
            match field_type {
                "Field" => format!("self.{}", field.0),
                "bool" | "u8" | "u32" | "u64" | "i8" | "i32" | "i64" => {
                    format!("self.{} as Field", field.0)
                }
                _ => format!("self.{}.to_field()", field.0),
            }
        })
        .collect::<Vec<String>>();
    let field_input = field_names.join(",");

    let trait_impl_source = format!(
        "
    impl dep::aztec::protocol_types::traits::Serialize<{event_len}> for {event_type} {{
        fn serialize(self: {event_type}) -> [Field; {event_len}] {{
            [{field_input}]
        }}
    }}
    "
    )
    .to_string();

    let (parsed_ast, errors) = parse_program(&trait_impl_source);
    if !errors.is_empty() {
        dbg!(errors);
        return Err(AztecMacroError::CouldNotImplementEventInterface {
            secondary_message: Some(format!("Failed to parse Noir macro code (trait impl of Serialize for {event_type}). This is either a bug in the compiler or the Noir macro code")),
        });
    }

    let mut sorted_ast = parsed_ast.into_sorted();
    let serialize_impl = sorted_ast.trait_impls.remove(0);

    Ok(serialize_impl)
}

fn generate_trait_impl_deserialize(
    event_type: &str,
    event_len: u32,
    event_fields: &[(String, String)],
) -> Result<NoirTraitImpl, AztecMacroError> {
    let field_names: Vec<String> = event_fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let field_type = field.1.as_str();
            match field_type {
                "Field" => format!("{}: fields[{}]", field.0, index),
                "bool" | "u8" | "u32" | "u64" | "i8" | "i32" | "i64" => {
                    format!("{}: fields[{}] as {}", field.0, index, field_type)
                }
                _ => format!("{}: {}::from_field(fields[{}])", field.0, field.1, index),
            }
        })
        .collect::<Vec<String>>();
    let field_input = field_names.join(",");

    let trait_impl_source = format!(
        "
    impl dep::aztec::protocol_types::traits::Deserialize<{event_len}> for {event_type} {{
        fn deserialize(fields: [Field; {event_len}]) -> {event_type} {{
            {event_type} {{ {field_input} }}
        }}
    }}
    "
    )
    .to_string();

    let (parsed_ast, errors) = parse_program(&trait_impl_source);
    if !errors.is_empty() {
        dbg!(errors);
        return Err(AztecMacroError::CouldNotImplementEventInterface {
            secondary_message: Some(format!("Failed to parse Noir macro code (trait impl of Deserialize for {event_type}). This is either a bug in the compiler or the Noir macro code")),
        });
    }

    let mut sorted_ast = parsed_ast.into_sorted();
    let deserialize_impl = sorted_ast.trait_impls.remove(0);

    Ok(deserialize_impl)
}

fn generate_fn_get_event_type_id(
    event_type: &str,
    field_length: u32,
) -> Result<NoirFunction, AztecMacroError> {
    let from_signature_input =
        std::iter::repeat("Field").take(field_length as usize).collect::<Vec<_>>().join(",");
    let function_source = format!(
        "
        fn get_event_type_id() -> dep::aztec::protocol_types::abis::event_selector::EventSelector {{
           dep::aztec::protocol_types::abis::event_selector::EventSelector::from_signature(\"{event_type}({from_signature_input})\")
    }}
    ",
    )
    .to_string();

    let (function_ast, errors) = parse_program(&function_source);
    if !errors.is_empty() {
        dbg!(errors);
        return Err(AztecMacroError::CouldNotImplementEventInterface {
            secondary_message: Some(format!("Failed to parse Noir macro code (fn get_event_type_id, implemented for EventInterface of {event_type}). This is either a bug in the compiler or the Noir macro code")),
        });
    }

    let mut function_ast = function_ast.into_sorted();
    let mut noir_fn = function_ast.functions.remove(0);
    noir_fn.def.visibility = ItemVisibility::Public;
    Ok(noir_fn)
}

fn generate_fn_private_to_be_bytes(
    event_type: &str,
    byte_length: u32,
) -> Result<NoirFunction, AztecMacroError> {
    let function_source = format!(
        "
         fn private_to_be_bytes(self: {event_type}, randomness: Field) -> [u8; {byte_length}] {{
             let mut buffer: [u8; {byte_length}] = [0; {byte_length}];

             let randomness_bytes = randomness.to_be_bytes(32);
             let event_type_id_bytes = {event_type}::get_event_type_id().to_field().to_be_bytes(32);

             for i in 0..32 {{
                 buffer[i] = randomness_bytes[i];
                 buffer[32 + i] = event_type_id_bytes[i];
            }}

             let serialized_event = self.serialize();

             for i in 0..serialized_event.len() {{
                 let bytes = serialized_event[i].to_be_bytes(32);
                 for j in 0..32 {{
                     buffer[64 + i * 32 + j] = bytes[j];
                }}
            }}

             buffer
        }}
    "
    )
    .to_string();

    let (function_ast, errors) = parse_program(&function_source);
    if !errors.is_empty() {
        dbg!(errors);
        return Err(AztecMacroError::CouldNotImplementEventInterface {
            secondary_message: Some(format!("Failed to parse Noir macro code (fn private_to_be_bytes, implemented for EventInterface of {event_type}). This is either a bug in the compiler or the Noir macro code")),
        });
    }

    let mut function_ast = function_ast.into_sorted();
    let mut noir_fn = function_ast.functions.remove(0);
    noir_fn.def.visibility = ItemVisibility::Public;
    Ok(noir_fn)
}

fn generate_fn_to_be_bytes(
    event_type: &str,
    byte_length: u32,
) -> Result<NoirFunction, AztecMacroError> {
    let byte_length_without_randomness = byte_length - 32;
    let function_source = format!(
        "
         fn to_be_bytes(self: {event_type}) -> [u8; {byte_length_without_randomness}] {{
             let mut buffer: [u8; {byte_length_without_randomness}] = [0; {byte_length_without_randomness}];

             let event_type_id_bytes = {event_type}::get_event_type_id().to_field().to_be_bytes(32);

             for i in 0..32 {{
                 buffer[i] = event_type_id_bytes[i];
            }}

             let serialized_event = self.serialize();

             for i in 0..serialized_event.len() {{
                 let bytes = serialized_event[i].to_be_bytes(32);
                 for j in 0..32 {{
                     buffer[32 + i * 32 + j] = bytes[j];
                }}
            }}

             buffer
        }}
    ")
    .to_string();

    let (function_ast, errors) = parse_program(&function_source);
    if !errors.is_empty() {
        dbg!(errors);
        return Err(AztecMacroError::CouldNotImplementEventInterface {
            secondary_message: Some(format!("Failed to parse Noir macro code (fn to_be_bytes, implemented for EventInterface of {event_type}). This is either a bug in the compiler or the Noir macro code")),
        });
    }

    let mut function_ast = function_ast.into_sorted();
    let mut noir_fn = function_ast.functions.remove(0);
    noir_fn.def.visibility = ItemVisibility::Public;
    Ok(noir_fn)
}

fn generate_fn_emit(event_type: &str) -> Result<NoirFunction, AztecMacroError> {
    let function_source = format!(
        "
        fn emit<Env>(self: {event_type}, _emit: fn[Env](Self) -> ()) {{
            _emit(self);
        }}
    "
    )
    .to_string();

    let (function_ast, errors) = parse_program(&function_source);
    if !errors.is_empty() {
        dbg!(errors);
        return Err(AztecMacroError::CouldNotImplementEventInterface {
            secondary_message: Some(format!("Failed to parse Noir macro code (fn emit, implemented for EventInterface of {event_type}). This is either a bug in the compiler or the Noir macro code")),
        });
    }

    let mut function_ast = function_ast.into_sorted();
    let mut noir_fn = function_ast.functions.remove(0);
    noir_fn.def.visibility = ItemVisibility::Public;
    Ok(noir_fn)
}

// We do this pass in the HIR to work around the "#[abi(tag)] attributes can only be used in contracts" error
pub fn transform_event_abi(
    crate_id: &CrateId,
    context: &mut HirContext,
) -> Result<(), (AztecMacroError, FileId)> {
    for struct_id in collect_crate_structs(crate_id, context) {
        let attributes = context.def_interner.struct_attributes(&struct_id);
        if attributes.iter().any(|attr| is_custom_attribute(attr, "aztec(event)")) {
            transform_event(struct_id, &mut context.def_interner)?;
        }
    }
    Ok(())
}

fn transform_event(
    struct_id: StructId,
    interner: &mut NodeInterner,
) -> Result<(), (AztecMacroError, FileId)> {
    interner.update_struct_attributes(struct_id, |struct_attributes| {
        struct_attributes.push(SecondaryAttribute::Abi("events".to_string()));
    });

    Ok(())
}
