#![cfg_attr(not(test), forbid(unsafe_code))] // `std::env::set_var` is used in tests.
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

#[doc = include_str!("../README.md")]
pub mod circuit;
pub mod native_types;
mod proto;
mod serialization;

pub use acir_field;
pub use acir_field::{AcirField, FieldElement};
pub use brillig;
pub use circuit::black_box_functions::BlackBoxFunc;
pub use circuit::opcodes::InvalidInputBitSize;

#[cfg(test)]
mod reflection {
    //! Getting test failures? You've probably changed the ACIR serialization format.
    //!
    //! These tests generate C++ deserializers for [`ACIR bytecode`][super::circuit::Circuit]
    //! and the [`WitnessMap`] structs. These get checked against the C++ files committed to the `codegen` folder
    //! to see if changes have been to the serialization format. These are almost always a breaking change!
    //!
    //! If you want to make a breaking change to the ACIR serialization format, then just comment out the assertions
    //! that the file hashes must match and rerun the tests. This will overwrite the `codegen` folder with the new
    //! logic. Make sure to uncomment these lines afterwards and to commit the changes to the `codegen` folder.

    use std::{
        collections::BTreeMap,
        fs::File,
        io::Write,
        path::{Path, PathBuf},
    };

    use acir_field::FieldElement;
    use brillig::{
        BinaryFieldOp, BinaryIntOp, BitSize, BlackBoxOp, HeapValueType, IntegerBitSize,
        MemoryAddress, Opcode as BrilligOpcode, ValueOrArray,
    };
    use prost::Message;
    use serde_generate::CustomCode;
    use serde_reflection::{
        ContainerFormat, Format, Named, Registry, Tracer, TracerConfig, VariantFormat,
    };

    use crate::{
        circuit::{
            AssertionPayload, Circuit, ExpressionOrMemory, ExpressionWidth, Opcode, OpcodeLocation,
            Program,
            brillig::{BrilligInputs, BrilligOutputs},
            opcodes::{BlackBoxFuncCall, BlockType, ConstantOrWitnessEnum, FunctionInput},
        },
        native_types::{Witness, WitnessMap, WitnessStack},
    };

    #[test]
    fn serde_acir_cpp_codegen() {
        let path = PathBuf::from("./codegen/acir.cpp");

        let old_hash = if path.is_file() {
            let old_source = std::fs::read(&path).unwrap();
            Some(fxhash::hash64(&old_source))
        } else {
            None
        };

        let mut tracer = Tracer::new(TracerConfig::default());
        tracer.trace_simple_type::<BlockType>().unwrap();
        tracer.trace_simple_type::<Program<FieldElement>>().unwrap();
        tracer.trace_simple_type::<Program<FieldElement>>().unwrap();
        tracer.trace_simple_type::<Circuit<FieldElement>>().unwrap();
        tracer.trace_simple_type::<ExpressionWidth>().unwrap();
        tracer.trace_simple_type::<Opcode<FieldElement>>().unwrap();
        tracer.trace_simple_type::<OpcodeLocation>().unwrap();
        tracer.trace_simple_type::<BinaryFieldOp>().unwrap();
        tracer.trace_simple_type::<ConstantOrWitnessEnum<FieldElement>>().unwrap();
        tracer.trace_simple_type::<FunctionInput<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BlackBoxFuncCall<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BrilligInputs<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BrilligOutputs>().unwrap();
        tracer.trace_simple_type::<BrilligOpcode<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BinaryIntOp>().unwrap();
        tracer.trace_simple_type::<BlackBoxOp>().unwrap();
        tracer.trace_simple_type::<ValueOrArray>().unwrap();
        tracer.trace_simple_type::<HeapValueType>().unwrap();
        tracer.trace_simple_type::<AssertionPayload<FieldElement>>().unwrap();
        tracer.trace_simple_type::<ExpressionOrMemory<FieldElement>>().unwrap();
        tracer.trace_simple_type::<BitSize>().unwrap();
        tracer.trace_simple_type::<IntegerBitSize>().unwrap();
        tracer.trace_simple_type::<MemoryAddress>().unwrap();

        let registry = tracer.registry().unwrap();

        let module_name = "Program";
        let msgpack_code = MsgPackCodeGenerator::generate(module_name, &registry);
        // Create C++ class definitions.
        let mut source = Vec::new();
        let config = serde_generate::CodeGeneratorConfig::new(module_name.to_string())
            .with_encodings(vec![serde_generate::Encoding::Bincode])
            .with_custom_code(msgpack_code);
        let generator = serde_generate::cpp::CodeGenerator::new(&config);
        generator.output(&mut source, &registry).unwrap();

        let source = MsgPackCodeGenerator::add_preamble(source);

        // Comment this out to write updated C++ code to file.
        if let Some(old_hash) = old_hash {
            let new_hash = fxhash::hash64(&source);
            assert_eq!(new_hash, old_hash, "Serialization format has changed");
        }

        write_to_file(&source, &path);
    }

    #[test]
    fn serde_witness_map_cpp_codegen() {
        let path = PathBuf::from("./codegen/witness.cpp");

        let old_hash = if path.is_file() {
            let old_source = std::fs::read(&path).unwrap();
            Some(fxhash::hash64(&old_source))
        } else {
            None
        };

        let mut tracer = Tracer::new(TracerConfig::default());
        tracer.trace_simple_type::<Witness>().unwrap();
        tracer.trace_simple_type::<WitnessMap<FieldElement>>().unwrap();
        tracer.trace_simple_type::<WitnessStack<FieldElement>>().unwrap();

        let module_name = "WitnessStack";
        let registry = tracer.registry().unwrap();
        let msgpack_code = MsgPackCodeGenerator::generate(module_name, &registry);

        // Create C++ class definitions.
        let mut source = Vec::new();
        let config = serde_generate::CodeGeneratorConfig::new(module_name.to_string())
            .with_encodings(vec![serde_generate::Encoding::Bincode])
            .with_custom_code(msgpack_code);
        let generator = serde_generate::cpp::CodeGenerator::new(&config);
        generator.output(&mut source, &registry).unwrap();

        let source = MsgPackCodeGenerator::add_preamble(source);

        // Comment this out to write updated C++ code to file.
        if let Some(old_hash) = old_hash {
            let new_hash = fxhash::hash64(&source);
            assert_eq!(new_hash, old_hash, "Serialization format has changed");
        }

        write_to_file(&source, &path);
    }

    fn write_to_file(bytes: &[u8], path: &Path) -> String {
        let display = path.display();

        let parent_dir = path.parent().unwrap();
        if !parent_dir.is_dir() {
            std::fs::create_dir_all(parent_dir).unwrap();
        }

        let mut file = match File::create(path) {
            Err(why) => panic!("couldn't create {display}: {why}"),
            Ok(file) => file,
        };

        match file.write_all(bytes) {
            Err(why) => panic!("couldn't write to {display}: {why}"),
            Ok(_) => display.to_string(),
        }
    }

    /// Generate custom code for the msgpack machinery in Barretenberg.
    /// See https://github.com/AztecProtocol/aztec-packages/blob/master/barretenberg/cpp/src/barretenberg/serialize/msgpack.hpp
    #[derive(Default)]
    struct MsgPackCodeGenerator {
        namespace: Vec<String>,
        code: CustomCode,
    }

    impl MsgPackCodeGenerator {
        /// Add the import of the Barretenberg C++ header for msgpack
        fn add_preamble(source: Vec<u8>) -> Vec<u8> {
            match String::from_utf8(source) {
                Ok(mut source) => {
                    let inc = r#"#include "serde.hpp""#;
                    let pos = source.find(inc).expect("serde.hpp missing");
                    source.insert_str(pos + inc.len(), "\n#include \"msgpack.hpp\"");
                    source.as_bytes().to_vec()
                }
                Err(e) => e.into_bytes(),
            }
        }

        fn generate(namespace: &str, registry: &Registry) -> CustomCode {
            let mut g = Self::default();
            g.namespace.push(namespace.to_string());
            for (name, container) in registry {
                g.generate_container(name, container);
            }
            g.code
        }

        /// Append custom code of an item in the current namespace.
        fn add_code(&mut self, name: &str, code: &str) {
            let mut ns = self.namespace.clone();
            ns.push(name.to_string());
            let c = self.code.entry(ns).or_default();
            if !c.is_empty() && code.contains('\n') {
                c.push('\n');
            }
            c.push_str(code);
            c.push('\n');
        }

        fn generate_container(&mut self, name: &str, container: &ContainerFormat) {
            use serde_reflection::ContainerFormat::*;
            match container {
                UnitStruct => {
                    self.generate_unit_struct(name);
                }
                NewTypeStruct(_format) => {
                    self.generate_newtype(name);
                }
                TupleStruct(formats) => {
                    self.generate_tuple(name, formats);
                }
                Struct(fields) => {
                    self.generate_struct(name, fields);
                }
                Enum(variants) => {
                    self.generate_enum(name, variants);
                }
            }
        }

        /// Unit structs don't have fields to put into the data.
        fn generate_unit_struct(&mut self, name: &str) {
            self.msgpack_fields(name, std::iter::empty());
        }

        /// Regular structs pack into a map.
        fn generate_struct(&mut self, name: &str, fields: &[Named<Format>]) {
            self.msgpack_fields(name, fields.iter().map(|f| f.name.clone()));
        }

        /// Newtypes serialize as their underlying `value` that the C++ generator creates
        fn generate_newtype(&mut self, name: &str) {
            self.msgpack_pack(name, "value.msgpack_pack(packer);");
            self.msgpack_unpack(name, "value.msgpack_unpack(o);")
        }

        /// Tuples serialize as a vector of underlying data
        fn generate_tuple(&mut self, _name: &str, _formats: &[Format]) {
            todo!("Implement msgpack for tuples");
        }

        /// Enums serialize as a single element map keyed by the variant type name.
        fn generate_enum(&mut self, name: &str, variants: &BTreeMap<u32, Named<VariantFormat>>) {
            // Recurse into the variants
            self.namespace.push(name.to_string());
            for (_, variant) in variants {
                self.generate_variant(&variant.name, &variant.value);
            }
            self.namespace.pop();

            // Pack the enum itself
            let pack_body = {
                let cases = variants
                    .iter()
                    .map(|(i, v)| {
                        format!(
                            r#"
        case {i}:
            tag = "{}";
            break;"#,
                            v.name
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("");

                format!(
                    r#"
    string tag;
    switch value.index() {{
        {cases}
        default:
            throw_or_abort("unknown enum variant index: " + std::to_string(value.index()));
    }}
    std::visit([](const auto& arg) {{ packer.pack(tag, arg); }}, value);"#
                )
            };
            self.msgpack_pack(name, &pack_body);

            // Unpack the enum
            let unpack_body = {
                let mut body = "
    std::map<std::string, msgpack::type::variant> data = o.convert();
    auto entry = data.begin();
    string tag = entry->first;
    auto o = entry->second;"
                    .to_string();

                for (i, v) in variants.iter() {
                    let name = &v.name;
                    body.push_str(&format!(
                        r#"
    {} (tag == "{name}") {{
        {name} v;
        v.msgpack_unpack(o);
        value = v;
    }}"#,
                        if *i == 0 { "if" } else { "else if" }
                    ))
                }

                body
            };
            self.msgpack_unpack(name, &unpack_body);
        }

        /// Generate msgpack code for nested enum variants.
        fn generate_variant(&mut self, name: &str, variant: &VariantFormat) {
            match variant {
                VariantFormat::Variable(_) => {
                    unreachable!("internal construct")
                }
                VariantFormat::Unit => self.generate_unit_struct(name),
                VariantFormat::NewType(_format) => self.generate_newtype(name),
                VariantFormat::Tuple(formats) => self.generate_tuple(name, formats),
                VariantFormat::Struct(fields) => self.generate_struct(name, fields),
            }
        }

        /// Use the `MSGPACK_FIELDS` macro with a list of fields.
        /// This one takes care of serializing and deserializing as well.
        fn msgpack_fields<'a>(&mut self, name: &str, fields: impl Iterator<Item = String>) {
            let fields = fields.collect::<Vec<_>>().join(", ");
            let code = format!("MSGPACK_FIELDS({});", fields);
            self.add_code(name, &code);
        }

        /// Add a `msgpack_pack` implementation.
        fn msgpack_pack(&mut self, name: &str, body: &str) {
            let code = Self::make_fn("void msgpack_pack(auto& packer) const", body);
            self.add_code(name, &code);
        }

        /// Add a `msgpack_unpack` implementation.
        fn msgpack_unpack(&mut self, name: &str, body: &str) {
            let code = Self::make_fn("void msgpack_unpack(auto const& o)", body);
            self.add_code(name, &code);
        }

        fn make_fn(header: &str, body: &str) -> String {
            let body = body.trim_end();
            if body.is_empty() {
                format!("{header} {{}}")
            } else if !body.contains('\n') {
                format!("{header} {{ {body} }}")
            } else if body.starts_with('\n') {
                format!("{header} {{{body}\n}}")
            } else {
                format!("{header} {{\n{body}\n}}")
            }
        }
    }
}
