//! C++ code generation for ACIR format, to be used by Barretenberg.
//!
//! To regenerate code run the following command:
//! ```text
//! NOIR_CODEGEN_OVERWRITE=1 cargo test -p acir cpp_codegen
//! ```
#![cfg_attr(not(test), forbid(unsafe_code))] // `std::env::set_var` is used in tests.
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

#[doc = include_str!("../README.md")]
pub mod circuit;
pub mod native_types;
mod parser;
mod serialization;

pub use acir_field;
pub use acir_field::{AcirField, FieldElement};
pub use brillig;
pub use circuit::black_box_functions::BlackBoxFunc;
pub use circuit::opcodes::InvalidInputBitSize;
pub use parser::parse_opcodes;
pub use serialization::Format as SerializationFormat;

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
        hash::BuildHasher,
        io::Write,
        path::{Path, PathBuf},
    };

    use acir_field::{AcirField, FieldElement};
    use brillig::{
        BinaryFieldOp, BinaryIntOp, BitSize, BlackBoxOp, HeapValueType, IntegerBitSize,
        MemoryAddress, Opcode as BrilligOpcode, ValueOrArray,
    };
    use regex::Regex;
    use serde::{Deserialize, Serialize};
    use serde_generate::CustomCode;
    use serde_reflection::{
        ContainerFormat, Format, Named, Registry, Tracer, TracerConfig, VariantFormat,
    };

    use crate::{
        circuit::{
            AssertionPayload, Circuit, ExpressionOrMemory, ExpressionWidth, Opcode, OpcodeLocation,
            Program,
            brillig::{BrilligInputs, BrilligOutputs},
            opcodes::{BlackBoxFuncCall, BlockType, FunctionInput},
        },
        native_types::{Witness, WitnessMap, WitnessStack},
    };

    /// Technical DTO for deserializing in Barretenberg while ignoring
    /// the Brillig opcodes, so that we can add more without affecting it.
    ///
    /// This could be achieved in other ways, for example by having a
    /// version of `Program` that deserializes into opaque bytes,
    /// which would require a 2 step (de)serialization process.
    ///
    /// This one is simpler. The cost is that msgpack will deserialize
    /// into a JSON-like structure, but since we won't be interpreting it,
    /// it's okay if new tags appear.
    #[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
    struct ProgramWithoutBrillig<F: AcirField> {
        pub functions: Vec<Circuit<F>>,
        /// We want to ignore this field. By setting its type as `unit`
        /// it will not be deserialized, but it will correctly maintain
        /// the position of the others (although in this case it doesn't)
        /// matter since it's the last field.
        pub unconstrained_functions: (),
    }

    #[test]
    fn serde_acir_cpp_codegen() {
        let mut tracer = Tracer::new(TracerConfig::default());
        tracer.trace_simple_type::<BlockType>().unwrap();
        tracer.trace_simple_type::<Program<FieldElement>>().unwrap();
        tracer.trace_simple_type::<ProgramWithoutBrillig<FieldElement>>().unwrap();
        tracer.trace_simple_type::<Circuit<FieldElement>>().unwrap();
        tracer.trace_simple_type::<ExpressionWidth>().unwrap();
        tracer.trace_simple_type::<Opcode<FieldElement>>().unwrap();
        tracer.trace_simple_type::<OpcodeLocation>().unwrap();
        tracer.trace_simple_type::<BinaryFieldOp>().unwrap();
        tracer.trace_simple_type::<FunctionInput<FieldElement>>().unwrap();
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

        serde_cpp_codegen(
            "Acir",
            PathBuf::from("./codegen/acir.cpp").as_path(),
            &tracer.registry().unwrap(),
            CustomCode::default(),
        );
    }

    #[test]
    fn serde_witness_map_cpp_codegen() {
        let mut tracer = Tracer::new(TracerConfig::default());
        tracer.trace_simple_type::<Witness>().unwrap();
        tracer.trace_simple_type::<WitnessMap<FieldElement>>().unwrap();
        tracer.trace_simple_type::<WitnessStack<FieldElement>>().unwrap();

        let namespace = "Witnesses";
        let mut code = CustomCode::default();
        // The `WitnessMap` type will have a field of type `std::map<Witnesses::Witness, std::string>`,
        // which requires us to implement the comparison operator.
        code.insert(
            vec![namespace.to_string(), "Witness".to_string()],
            "bool operator<(Witness const& rhs) const { return value < rhs.value; }".to_string(),
        );

        serde_cpp_codegen(
            namespace,
            PathBuf::from("./codegen/witness.cpp").as_path(),
            &tracer.registry().unwrap(),
            code,
        );
    }

    /// Regenerate C++ code for serializing our domain model based on serde,
    /// intended to be used by Barretenberg.
    ///
    /// If `should_overwrite()` returns `false` then just check if the old file hash is the
    /// same as the new one, to guard against unintended changes in the serialization format.
    fn serde_cpp_codegen(namespace: &str, path: &Path, registry: &Registry, code: CustomCode) {
        let old_hash = if path.is_file() {
            let old_source = std::fs::read(path).expect("failed to read existing code");
            let old_source = String::from_utf8(old_source).expect("old source not UTF-8");
            Some(rustc_hash::FxBuildHasher.hash_one(&old_source))
        } else {
            None
        };
        let msgpack_code = MsgPackCodeGenerator::generate(
            namespace,
            registry,
            code,
            MsgPackCodeConfig { pack_compact: true },
        );

        // Create C++ class definitions.
        let mut source = Vec::new();
        // Barretenberg doesn't want to support the serde_generate::Encoding::Bincode encoding any more, only the custom msgpack.
        let config = serde_generate::CodeGeneratorConfig::new(namespace.to_string())
            .with_encodings(vec![])
            .with_custom_code(msgpack_code);
        let generator = serde_generate::cpp::CodeGenerator::new(&config);
        generator.output(&mut source, registry).expect("failed to generate C++ code");

        // Further massaging of the generated code
        let mut source = String::from_utf8(source).expect("not a UTF-8 string");
        replace_throw(&mut source);
        MsgPackCodeGenerator::add_preamble(&mut source);
        MsgPackCodeGenerator::add_helpers(&mut source, namespace);
        MsgPackCodeGenerator::replace_array_with_shared_ptr(&mut source);

        if !should_overwrite() {
            if let Some(old_hash) = old_hash {
                let new_hash = rustc_hash::FxBuildHasher.hash_one(&source);
                assert_eq!(new_hash, old_hash, "Serialization format has changed",);
            }
        }

        write_to_file(source.as_bytes(), path);
    }

    /// Check if it's okay for the generated source to be overwritten with a new version.
    /// Otherwise any changes causes a test failure.
    fn should_overwrite() -> bool {
        std::env::var("NOIR_CODEGEN_OVERWRITE")
            .ok()
            .map(|v| v == "1" || v == "true")
            .unwrap_or_default()
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

    /// Replace all `throw serde::deserialization_error` with `throw_or_abort`.
    ///
    /// Since we're generating msgpack code that works specifically with the Barretenberg
    /// codebase only (these are custom functions), we might as well do the other alterations
    /// described in [the DSL](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg/cpp/src/barretenberg/dsl).
    fn replace_throw(source: &mut String) {
        *source = source.replace("throw serde::deserialization_error", "throw_or_abort");
    }

    struct MsgPackCodeConfig {
        /// If `true`, use `ARRAY` format for structs, otherwise use `MAP` when packing.
        pack_compact: bool,
    }

    /// Generate custom code for the msgpack machinery in Barretenberg.
    /// See https://github.com/AztecProtocol/aztec-packages/blob/master/barretenberg/cpp/src/barretenberg/serialize/msgpack.hpp
    struct MsgPackCodeGenerator {
        config: MsgPackCodeConfig,
        namespace: Vec<String>,
        code: CustomCode,
    }

    impl MsgPackCodeGenerator {
        /// Add the import of the Barretenberg C++ header for msgpack.
        pub(crate) fn add_preamble(source: &mut String) {
            let inc = r#"#include "serde.hpp""#;
            let pos = source.find(inc).expect("serde.hpp missing");
            source.insert_str(
                pos + inc.len(),
                "\n#include \"barretenberg/serialize/msgpack_impl.hpp\"",
            );
        }

        /// Add helper functions to cut down repetition in the generated code.
        pub(crate) fn add_helpers(source: &mut String, namespace: &str) {
            // Based on https://github.com/AztecProtocol/msgpack-c/blob/54e9865b84bbdc73cfbf8d1d437dbf769b64e386/include/msgpack/v1/adaptor/detail/cpp11_define_map.hpp#L75
            // Using a `struct Helpers` with `static` methods, because top level functions turn up as duplicates in `wasm-ld`.
            // cSpell:disable
            let helpers = r#"
    struct Helpers {
        static std::map<std::string, msgpack::object const*> make_kvmap(
            msgpack::object const& o,
            std::string const& name
        ) {
            if (o.type != msgpack::type::MAP) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP for " + name);
            }
            std::map<std::string, msgpack::object const*> kvmap;
            for (uint32_t i = 0; i < o.via.map.size; ++i) {
                if (o.via.map.ptr[i].key.type != msgpack::type::STR) {
                    std::cerr << o << std::endl;
                    throw_or_abort("expected STR for keys of " + name);
                }
                kvmap.emplace(
                    std::string(
                        o.via.map.ptr[i].key.via.str.ptr,
                        o.via.map.ptr[i].key.via.str.size),
                    &o.via.map.ptr[i].val);
            }
            return kvmap;
        }

        template<typename T>
        static void conv_fld_from_kvmap(
            std::map<std::string, msgpack::object const*> const& kvmap,
            std::string const& struct_name,
            std::string const& field_name,
            T& field,
            bool is_optional
        ) {
            auto it = kvmap.find(field_name);
            if (it != kvmap.end()) {
                try {
                    it->second->convert(field);
                } catch (const msgpack::type_error&) {
                    std::cerr << *it->second << std::endl;
                    throw_or_abort("error converting into field " + struct_name + "::" + field_name);
                }
            } else if (!is_optional) {
                throw_or_abort("missing field: " + struct_name + "::" + field_name);
            }
        }

        template<typename T>
        static void conv_fld_from_array(
            msgpack::object_array const& array,
            std::string const& struct_name,
            std::string const& field_name,
            T& field,
            uint32_t index
        ) {
            if (index >= array.size) {
                throw_or_abort("index out of bounds: " + struct_name + "::" + field_name + " at " + std::to_string(index));
            }
            auto element = array.ptr[index];
            try {
                element.convert(field);
            } catch (const msgpack::type_error&) {
                std::cerr << element << std::endl;
                throw_or_abort("error converting into field " + struct_name + "::" + field_name);
            }
        }
    };
    "#;
            // cSpell:enable
            let pos = source.find(&format!("namespace {namespace}")).expect("namespace");
            source.insert_str(pos, &format!("namespace {namespace} {{{helpers}}}\n\n"));
        }

        /// Reduce the opcode size in C++ by doing what Adam came up with in https://github.com/zefchain/serde-reflection/issues/75
        fn replace_array_with_shared_ptr(source: &mut String) {
            // Capture `std::array<$TYPE, $LEN>`
            let re = Regex::new(r#"std::array<\s*([^,<>]+?)\s*,\s*([0-9]+)\s*>"#)
                .expect("failed to create regex");

            let fixed =
                re.replace_all(source, "std::shared_ptr<std::array<${1}, ${2}>>").into_owned();

            *source = fixed;
        }

        /// Add custom code for msgpack serialization and deserialization.
        pub(crate) fn generate(
            namespace: &str,
            registry: &Registry,
            code: CustomCode,
            config: MsgPackCodeConfig,
        ) -> CustomCode {
            let mut g = Self { namespace: vec![namespace.to_string()], code, config };
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
            // Ostensibly we could use `MSGPACK_FIELDS();`, but because of how enum unpacking
            // expects each variant to have `msgpack_unpack`, we generate two empty methods.
            // self.msgpack_fields(name, std::iter::empty());
            self.msgpack_pack(name, "");
            self.msgpack_unpack(name, "");
        }

        /// Regular structs pack into a map.
        fn generate_struct(&mut self, name: &str, fields: &[Named<Format>]) {
            // We could use the `MSGPACK_FIELDS` macro with the following:
            // self.msgpack_fields(name, fields.iter().map(|f| f.name.clone()));
            // Unfortunately it doesn't seem to deal with missing optional fields,
            // which would mean we can't delete fields even if they were optional:
            // https://github.com/AztecProtocol/msgpack-c/blob/54e9865b84bbdc73cfbf8d1d437dbf769b64e386/include/msgpack/v1/adaptor/detail/cpp11_define_map.hpp#L33-L45

            // Or we can generate code for individual fields, which relies on
            // the `add_helpers` to add some utility functions. This way the
            // code is more verbose, but also easier to control, e.g. we can
            // raise errors telling specifically which field was wrong,
            // or we could reject the data if there was a new field we could
            // not recognize, or we could even handle aliases.

            // We treat unit fields as special, using them to ignore fields during deserialization:
            // * in 'map' format we skip over them, never try to deserialize them from the map
            // * in 'tuple' format we jump over their index, ignoring whatever is in that position
            fn is_unit(field: &Named<Format>) -> bool {
                matches!(field.value, Format::Unit)
            }

            let non_unit_field_count = fields.iter().filter(|f| !is_unit(f)).count();

            self.msgpack_pack(name, &{
                if self.config.pack_compact {
                    // Pack as ARRAY
                    let mut body = format!(
                        "
    packer.pack_array({});",
                        fields.len()
                    );
                    for field in fields {
                        let field_name = &field.name;
                        body.push_str(&format!(
                            r#"
    packer.pack({field_name});"#
                        ));
                    }
                    body
                } else {
                    // Pack as MAP
                    let mut body = format!(
                        "
    packer.pack_map({non_unit_field_count});",
                    );
                    for field in fields {
                        if is_unit(field) {
                            continue;
                        }
                        let field_name = &field.name;
                        body.push_str(&format!(
                            r#"
    packer.pack(std::make_pair("{field_name}", {field_name}));"#
                        ));
                    }
                    body
                }
            });

            self.msgpack_unpack(name, &{
                // Turn the MAP into a `std::map<string, msgpack::object>`,
                // then look up each field, returning error if one isn't found.
                // cSpell:disable
                let mut body = format!(
                    r#"
    std::string name = "{name}";
    if (o.type == msgpack::type::MAP) {{
        auto kvmap = Helpers::make_kvmap(o, name);"#
                );
                // cSpell:enable
                for field in fields {
                    if is_unit(field) {
                        continue;
                    }
                    let field_name = &field.name;
                    let is_optional = matches!(field.value, Format::Option(_));
                    // cSpell:disable
                    body.push_str(&format!(
                        r#"
        Helpers::conv_fld_from_kvmap(kvmap, name, "{field_name}", {field_name}, {is_optional});"#
                    ));
                    // cSpell:enable
                }
                body.push_str(
                    "
    } else if (o.type == msgpack::type::ARRAY) {
        auto array = o.via.array; ",
                );
                for (index, field) in fields.iter().enumerate() {
                    if is_unit(field) {
                        continue;
                    }
                    let field_name = &field.name;
                    // cSpell:disable
                    body.push_str(&format!(
                        r#"
        Helpers::conv_fld_from_array(array, name, "{field_name}", {field_name}, {index});"#
                    ));
                    // cSpell:enable
                }

                body.push_str(
                    r#"
    } else {
        throw_or_abort("expected MAP or ARRAY for " + name);
    }"#,
                );
                body
            });
        }

        /// Newtypes serialize as their underlying `value` that the C++ generator creates.
        fn generate_newtype(&mut self, name: &str) {
            self.msgpack_pack(name, "packer.pack(value);");
            self.msgpack_unpack(
                name,
                // cSpell:disable
                &format!(
                    r#"
    try {{
        o.convert(value);
    }} catch (const msgpack::type_error&) {{
        std::cerr << o << std::endl;
        throw_or_abort("error converting into newtype '{name}'");
    }}
            "#
                ),
                // cSpell:enable
            );
        }

        /// Tuples serialize as a vector of underlying data.
        fn generate_tuple(&mut self, _name: &str, _formats: &[Format]) {
            unimplemented!("Until we have a tuple enum in our schema we don't need this.");
        }

        /// Enums serialize as a single element map keyed by the variant type name.
        fn generate_enum(&mut self, name: &str, variants: &BTreeMap<u32, Named<VariantFormat>>) {
            // Recurse into the variants
            self.namespace.push(name.to_string());
            for variant in variants.values() {
                self.generate_variant(&variant.name, &variant.value);
            }
            self.namespace.pop();

            // Pack the enum itself
            self.msgpack_pack(name, &{
                let cases = variants
                    .iter()
                    .map(|(i, v)| {
                        format!(
                            r#"
        case {i}:
            tag = "{}";
            is_unit = {};
            break;"#,
                            v.name,
                            matches!(v.value, VariantFormat::Unit)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("");

                format!(
                    r#"
    std::string tag;
    bool is_unit;
    switch (value.index()) {{
        {cases}
        default:
            throw_or_abort("unknown enum '{name}' variant index: " + std::to_string(value.index()));
    }}
    if (is_unit) {{
        packer.pack(tag);
    }} else {{
        std::visit([&packer, tag](const auto& arg) {{
            packer.pack_map(1);
            packer.pack(tag);
            packer.pack(arg);
        }}, value);
    }}"#
                )
            });

            // Unpack the enum into a map, inspect the key, then unpack the entry value.
            // See https://c.msgpack.org/cpp/structmsgpack_1_1object.html#a8c7c484d2a6979a833bdb69412ad382c
            // for how to access the object's content without parsing it.
            self.msgpack_unpack(name, &{
                // cSpell:disable
                let mut body = format!(
                    r#"

    if (o.type != msgpack::type::object_type::MAP && o.type != msgpack::type::object_type::STR) {{
        std::cerr << o << std::endl;
        throw_or_abort("expected MAP or STR for enum '{name}'; got type " + std::to_string(o.type));
    }}
    if (o.type == msgpack::type::object_type::MAP && o.via.map.size != 1) {{
        throw_or_abort("expected 1 entry for enum '{name}'; got " + std::to_string(o.via.map.size));
    }}
    std::string tag;
    try {{
        if (o.type == msgpack::type::object_type::MAP) {{
            o.via.map.ptr[0].key.convert(tag);
        }} else {{
            o.convert(tag);
        }}
    }} catch(const msgpack::type_error&) {{
        std::cerr << o << std::endl;
        throw_or_abort("error converting tag to string for enum '{name}'");
    }}"#
                );
                // cSpell:enable

                for (i, v) in variants.iter() {
                    let variant = &v.name;
                    body.push_str(&format!(
                        r#"
    {}if (tag == "{variant}") {{
        {variant} v;"#,
                        if *i == 0 { "" } else { "else " }
                    ));

                    if !matches!(v.value, VariantFormat::Unit) {
                        // cSpell:disable
                        body.push_str(&format!(
                            r#"
        try {{
            o.via.map.ptr[0].val.convert(v);
        }} catch (const msgpack::type_error&) {{
            std::cerr << o << std::endl;
            throw_or_abort("error converting into enum variant '{name}::{variant}'");
        }}
        "#
                        ));
                        // cSpell:enable
                    }
                    // Closing brace of if statement
                    body.push_str(
                        r#"
        value = v;
    }"#,
                    );
                }
                // cSpell:disable
                body.push_str(&format!(
                    r#"
    else {{
        std::cerr << o << std::endl;
        throw_or_abort("unknown '{name}' enum variant: " + tag);
    }}"#
                ));
                // cSpell:enable

                body
            });
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
        ///
        /// Uses [define_map](https://github.com/AztecProtocol/msgpack-c/blob/54e9865b84bbdc73cfbf8d1d437dbf769b64e386/include/msgpack/v1/adaptor/detail/cpp11_define_map.hpp#L75-L88) under the hood.
        #[allow(dead_code)]
        fn msgpack_fields(&mut self, name: &str, fields: impl Iterator<Item = String>) {
            let fields = fields.collect::<Vec<_>>().join(", ");
            let code = format!("MSGPACK_FIELDS({fields});");
            self.add_code(name, &code);
        }

        /// Add a `msgpack_pack` implementation.
        fn msgpack_pack(&mut self, name: &str, body: &str) {
            let code = Self::make_fn("void msgpack_pack(auto& packer) const", body);
            self.add_code(name, &code);
        }

        /// Add a `msgpack_unpack` implementation.
        fn msgpack_unpack(&mut self, name: &str, body: &str) {
            // Using `msgpack::object const& o` instead of `auto o`, because the latter is passed as `msgpack::object::implicit_type`,
            // which would have to be cast like `msgpack::object obj = o;`. This `const&` pattern exists in `msgpack-c` codebase.

            // Instead of implementing the `msgpack_unpack` method as suggested by `msgpack.hpp` in Barretenberg,
            // we could implement an extension method on `msgpack::object` as below. However, it has to be in
            // the `msgpack::adaptor` namespace, which would mean it has to be appended at the end of the code,
            // rather than into the structs, where `CustomCode` goes.
            //
            // namespace msgpack {
            // namespace adaptor {
            // // For Opcode
            // template <> struct msgpack::adaptor::convert<Acir::Opcode> {
            //     msgpack::object const& operator()(msgpack::object const& o, Acir::Opcode& v) const
            //     {
            //         return o;
            //         if (o.type != msgpack::type::MAP || o.via.map.size != 1) {
            //             throw_or_abort("expected single element map for 'Opcode'");
            //         }

            //         auto& kv = o.via.map.ptr[0];
            //         std::string key = kv.key.as<std::string>();

            //         if (key == "BrilligCall") {
            //             Acir::Opcode::BrilligCall bc = kv.val.as<Acir::Opcode::BrilligCall>();
            //             v.value = bc;
            //         } else if (key == "AssertZero") {
            //             Acir::Opcode::AssertZero az = kv.val.as<Acir::Opcode::AssertZero>();
            //             v.value = az;
            //         } else {
            //             throw_or_abort("unknown tag for 'Opcode': " + key);
            //         }
            //         return o;
            //     }
            // };
            // } // namespace adaptor
            // } // namespace msgpack

            let code = Self::make_fn("void msgpack_unpack(msgpack::object const& o)", body);
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
