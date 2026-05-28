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

#[cfg(feature = "test-fixtures")]
pub mod test_fixtures;

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
    use msgpack_tagged::{MsgpackTagged, Product, Sum, TagRegistry};
    use regex::Regex;
    use serde::{Deserialize, Serialize};
    use serde_generate::CustomCode;
    use serde_reflection::{
        ContainerFormat, Format, Named, Registry, Samples, Tracer, TracerConfig, VariantFormat,
    };

    use crate::{
        circuit::{
            AssertionPayload, Circuit, ExpressionOrMemory, Opcode, OpcodeLocation, Program,
            brillig::{BrilligInputs, BrilligOutputs},
            opcodes::{BlackBoxFuncCall, BlockType, FunctionInput, MemOp},
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
    #[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default, Hash, MsgpackTagged)]
    struct ProgramWithoutBrillig<F: AcirField> {
        #[tag(0)]
        pub functions: Vec<Circuit<F>>,
        /// We want to ignore this field. By setting its type as `unit`
        /// it will not be deserialized, but it will correctly maintain
        /// the position of the others (although in this case it doesn't)
        /// matter since it's the last field.
        #[tag(1)]
        pub unconstrained_functions: (),
    }

    #[test]
    fn serde_acir_cpp_codegen() {
        // MemOp has a custom Deserialize impl with validation that rejects the zero-expression
        // samples that trace_simple_type generates. Enable record_samples_for_structs so that
        // pre-registered samples are used when the tracer encounters a known struct type.
        let config = TracerConfig::default().record_samples_for_structs(true);
        let mut tracer = Tracer::new(config);

        let mut samples = Samples::new();
        tracer
            .trace_value(&mut samples, &MemOp::read_at_mem_index(Witness(0), Witness(0)))
            .unwrap();
        tracer
            .trace_value(&mut samples, &MemOp::write_to_mem_index(Witness(0), Witness(0)))
            .unwrap();

        tracer.trace_simple_type::<BlockType>().unwrap();
        tracer.trace_simple_type::<Program<FieldElement>>().unwrap();
        tracer.trace_simple_type::<ProgramWithoutBrillig<FieldElement>>().unwrap();
        tracer.trace_simple_type::<Circuit<FieldElement>>().unwrap();
        tracer.trace_type::<Opcode<FieldElement>>(&samples).unwrap();
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

        // The `msgpack_tagged` tag registry mirrors the type graph that
        // serde-reflection traced above, but additionally carries the
        // integer tag → field-name (and variant) mapping the C++ codegen
        // needs for the new int-keyed dispatch branch. We register the
        // same top-level types as the tracer: `Program<F>` covers the
        // bulk of the graph via its transitive walk, and
        // `ProgramWithoutBrillig<F>` is a sibling type that needs its own
        // entry (its serde name `"ProgramWithoutBrillig"` is distinct
        // from `"Program"`, and its `unconstrained_functions: ()` field
        // doesn't introduce any types that Program's walk didn't already
        // reach).
        let mut tag_registry = TagRegistry::new();
        <Program<FieldElement> as MsgpackTagged>::register_into(&mut tag_registry);
        <ProgramWithoutBrillig<FieldElement> as MsgpackTagged>::register_into(&mut tag_registry);

        serde_cpp_codegen(
            "Acir",
            PathBuf::from("./codegen/acir.cpp").as_path(),
            &tracer.registry().unwrap(),
            &tag_registry,
            CustomCode::default(),
        );
    }

    #[test]
    fn serde_witness_map_cpp_codegen() {
        let mut tracer = Tracer::new(TracerConfig::default());
        tracer.trace_simple_type::<Witness>().unwrap();
        tracer.trace_simple_type::<WitnessMap<FieldElement>>().unwrap();
        tracer.trace_simple_type::<WitnessStack<FieldElement>>().unwrap();

        let mut tag_registry = TagRegistry::new();
        <Witness as MsgpackTagged>::register_into(&mut tag_registry);
        <WitnessMap<FieldElement> as MsgpackTagged>::register_into(&mut tag_registry);
        <WitnessStack<FieldElement> as MsgpackTagged>::register_into(&mut tag_registry);

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
            &tag_registry,
            code,
        );
    }

    /// Regenerate C++ code for serializing our domain model based on serde,
    /// intended to be used by Barretenberg.
    ///
    /// If `should_overwrite()` returns `false` then just check if the old file hash is the
    /// same as the new one, to guard against unintended changes in the serialization format.
    fn serde_cpp_codegen(
        namespace: &str,
        path: &Path,
        registry: &Registry,
        tag_registry: &TagRegistry,
        code: CustomCode,
    ) {
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
            tag_registry,
            code,
            MsgPackCodeConfig::from_env(),
        );

        // Create C++ class definitions.
        let mut source = Vec::new();
        // We use `serde_generate` to take advantage of its integration with `serde_reflection` but only use our
        // custom msgpack code generation.
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
        MsgPackCodeGenerator::add_autogen_header(&mut source);

        if !should_overwrite()
            && let Some(old_hash) = old_hash
        {
            let new_hash = rustc_hash::FxBuildHasher.hash_one(&source);
            assert_eq!(new_hash, old_hash, "Serialization format has changed",);
        }

        write_to_file(source.as_bytes(), path);
    }

    /// Get a boolean flag env var.
    fn env_flag(name: &str, default: bool) -> bool {
        let Ok(s) = std::env::var(name) else {
            return default;
        };
        match s.as_str() {
            "1" | "true" | "yes" => true,
            "0" | "false" | "no" => false,
            _ => default,
        }
    }

    /// Check if it's okay for the generated source to be overwritten with a new version.
    /// Otherwise any changes causes a test failure.
    fn should_overwrite() -> bool {
        env_flag("NOIR_CODEGEN_OVERWRITE", false)
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
        /// If `true`, use `ARRAY` format, otherwise use `MAP` when packing structs.
        pack_compact: bool,
        /// If `true`, skip generating `msgpack_pack` methods.
        no_pack: bool,
    }

    impl MsgPackCodeConfig {
        fn from_env() -> Self {
            Self {
                // We agreed on the default format to be compact, so it makes sense for Barretenberg to use it for serialization.
                pack_compact: env_flag("NOIR_CODEGEN_PACK_COMPACT", true),
                // Barretenberg didn't use serialization outside tests, so they decided they don't want to have this code at all.
                // But in the latest code they seem to have kept it again.
                no_pack: env_flag("NOIR_CODEGEN_NO_PACK", false),
            }
        }
    }

    /// Assert that every product / sum container in the serde-reflection
    /// `registry` is also present in the `MsgpackTagged` `tag_registry`,
    /// and that every product is in *canonical* (tag-ascending) source
    /// order. Panics with a focused message on the first miss.
    ///
    /// **Why the coverage check.** The int-keyed dispatch branch in the
    /// generated C++ needs each field's u8 tag (for structs) and each
    /// variant's u8 tag (for enums). That metadata only exists on the
    /// `MsgpackTagged` side. A type in `registry` but not in
    /// `tag_registry` means somebody added a wire type without deriving
    /// `MsgpackTagged` — fail loudly at codegen time, not silently at
    /// runtime.
    ///
    /// **Why the order check.** Under `Format::MsgpackCompact` a struct
    /// is emitted as a fixarray in *source* order; under
    /// `Format::MsgpackTagged` with the `Array` strategy it's a fixarray
    /// in *tag-ascending* order. Both shapes look identical to the C++
    /// decoder (just `msgpack::type::ARRAY`), so byte-different wires
    /// for the same logical value would silently land on the wrong
    /// fields if the two orders ever diverge. Forcing
    /// `tag_order_matches_source` keeps them lockstep — both formats
    /// produce byte-identical arrays, and the codegen reads positionally
    /// in tag-ascending order.
    fn assert_tag_registry_covers(registry: &Registry, tag_registry: &TagRegistry) {
        use serde_reflection::ContainerFormat;
        for (name, container) in registry {
            // Only named-struct and enum containers need an entry in the
            // `MsgpackTagged` registry: their int-keyed dispatch branch
            // depends on per-field / per-variant tag metadata. Unit
            // structs, newtypes, and (currently `unimplemented!`) tuple
            // structs pass through to their inner type (or are no-ops)
            // and don't register themselves in `TagRegistry`.
            let needs_tags =
                matches!(container, ContainerFormat::Struct(_) | ContainerFormat::Enum(_),);
            if !needs_tags {
                continue;
            }
            let Some(entry) = tag_registry.get(name) else {
                panic!(
                    "MsgpackTagged tag registry is missing {name:?} — the type is on \
                     the wire (serde-reflection traced it as a struct/enum) but \
                     doesn't derive `MsgpackTagged`. Add `#[derive(MsgpackTagged)]` \
                     to the type, and a `register_into` call at the codegen test \
                     site if the type isn't reachable from a type that's already \
                     registered.",
                );
            };
            if let Some(p) = entry.tagged().as_product()
                && !p.tag_order_matches_source
            {
                panic!(
                    "MsgpackTagged product {name:?} declares its fields in an order \
                     that doesn't match tag-ascending. The C++ codegen can't tell \
                     `MsgpackCompact` (source-order array) from \
                     `MsgpackTagged::Array` (tag-ascending array) on the wire — \
                     reorder the Rust fields so `#[tag(N)]` values are increasing \
                     in source order, or drop the type from the C++ wire types.",
                );
            }
        }
    }

    /// Generate custom code for the msgpack machinery in Barretenberg.
    /// See https://github.com/AztecProtocol/aztec-packages/blob/master/barretenberg/cpp/src/barretenberg/serialize/msgpack.hpp
    struct MsgPackCodeGenerator<'a> {
        config: MsgPackCodeConfig,
        namespace: Vec<String>,
        code: CustomCode,
        /// Carries the integer-tag metadata produced by `MsgpackTagged`
        /// derives. The serde-reflection `Registry` only knows field/variant
        /// *names*; consulting this registry lets us pair each name with its
        /// stable u8 tag so the generated C++ can dispatch on int-keyed
        /// `MsgpackTagged` wires the same way the Rust decoder does.
        tag_registry: &'a TagRegistry,
    }

    impl<'a> MsgPackCodeGenerator<'a> {
        /// Prepend a banner marking the file as auto-generated and pointing
        /// readers at the generator. Without it, the committed `.cpp` files
        /// look hand-written and an unsuspecting editor would lose their
        /// changes the next time the codegen tests run.
        pub(crate) fn add_autogen_header(source: &mut String) {
            let header = "\
// AUTO-GENERATED — DO NOT EDIT.
//
// Generated by the `cpp_codegen` tests in `acvm-repo/acir/src/lib.rs`.
// To regenerate, run:
//
//     NOIR_CODEGEN_OVERWRITE=1 cargo test -p acir cpp_codegen
//
";
            source.insert_str(0, header);
        }

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
                if (!is_optional && it->second->type == msgpack::type::NIL) {
                    throw_or_abort("nil value for required field: " + struct_name + "::" + field_name);
                }
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
            if (element.type == msgpack::type::NIL) {
                throw_or_abort("nil value for required field: " + struct_name + "::" + field_name);
            }
            try {
                element.convert(field);
            } catch (const msgpack::type_error&) {
                std::cerr << element << std::endl;
                throw_or_abort("error converting into field " + struct_name + "::" + field_name);
            }
        }

        /// Convert `val` into `field`, or throw a focused error mentioning
        /// the struct + field name. Used by the int-keyed dispatch path
        /// where each `switch` case populates one field directly.
        template<typename T>
        static void convert_or_throw(
            msgpack::object const& val,
            std::string const& struct_name,
            std::string const& field_name,
            T& field
        ) {
            try {
                val.convert(field);
            } catch (const msgpack::type_error&) {
                std::cerr << val << std::endl;
                throw_or_abort("error converting into field " + struct_name + "::" + field_name);
            }
        }

        /// Whether `o` is a non-empty MAP whose first key is an integer.
        /// This is the signature of `Format::MsgpackTagged`: int keys for
        /// struct field tags and enum variant tags. Legacy `Format::Msgpack`
        /// keys are always strings, so a positive-integer first key is a
        /// reliable shape discriminator between the two.
        static bool is_int_keyed_map(msgpack::object const& o) {
            return o.type == msgpack::type::MAP
                && o.via.map.size > 0
                && o.via.map.ptr[0].key.type == msgpack::type::POSITIVE_INTEGER;
        }

        /// Iterate an int-keyed MAP and invoke `dispatch(tag, val)` for each
        /// `(u8, msgpack::object)` entry. The per-tag `switch` inside the
        /// caller's lambda decides which field (or variant) to populate;
        /// unknown tags fall through to `default` and are silently skipped,
        /// matching the `MsgpackTagged` decoder's forward-compat policy
        /// (`allow_unknown_tags` / retired tags drained).
        template<typename Dispatch>
        static void int_map_dispatch(
            msgpack::object const& o,
            std::string const& name,
            Dispatch&& dispatch
        ) {
            for (uint32_t i = 0; i < o.via.map.size; ++i) {
                uint8_t tag;
                try {
                    o.via.map.ptr[i].key.convert(tag);
                } catch (const msgpack::type_error&) {
                    std::cerr << o.via.map.ptr[i].key << std::endl;
                    throw_or_abort("expected u8 tag in int-keyed map for " + name);
                }
                dispatch(tag, o.via.map.ptr[i].val);
            }
        }

        /// Cap a `MAP` or `ARRAY` entry count against `active + reserved`.
        /// Under-length wires are caught downstream (`conv_fld_from_array`
        /// errors out of bounds; `conv_fld_from_kvmap` errors on missing
        /// required keys), so we only need the upper bound here.
        ///
        /// * Up to `reserved` extra trailing entries are tolerated as
        ///   retired fields (`#[tagged(reserved(...))]` on the Rust side).
        /// * Anything beyond that is forward-compat drift that the
        ///   producer only emits when newer fields were added. The
        ///   Rust-side cue is `#[tagged(allow_unknown_tags)]`; the
        ///   message points at it so a reviewer can see the opt-in.
        static void check_size(
            uint32_t actual,
            std::string const& name,
            uint32_t active,
            uint32_t reserved
        ) {
            uint32_t max_size = active + reserved;
            if (actual > max_size) {
                throw_or_abort(
                    name + " has " + std::to_string(actual) +
                    " entries but at most " + std::to_string(max_size) +
                    " are expected (" + std::to_string(active) +
                    " active + " + std::to_string(reserved) +
                    " reserved); opt into `#[tagged(allow_unknown_tags)]` on the Rust type to accept extras");
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
        ///
        /// Walks the serde-reflection `registry` and asserts that every
        /// product/sum container is also present in the `MsgpackTagged`
        /// `tag_registry` — every wire type must derive `MsgpackTagged`
        /// for the int-keyed dispatch to have tag metadata available.
        /// Products additionally have to be in *canonical* (tag-ascending)
        /// source order — otherwise `MsgpackCompact` (source-order array)
        /// and `MsgpackTagged::Array` (tag-ascending array) would produce
        /// indistinguishable but byte-different wires on the same input,
        /// and the C++ side can't tell which to expect.
        pub(crate) fn generate(
            namespace: &str,
            registry: &Registry,
            tag_registry: &'a TagRegistry,
            code: CustomCode,
            config: MsgPackCodeConfig,
        ) -> CustomCode {
            assert_tag_registry_covers(registry, tag_registry);
            let mut g = Self { namespace: vec![namespace.to_string()], code, config, tag_registry };
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
                    // Top-level struct — look up its Product in the
                    // tag registry. `assert_tag_registry_covers` has
                    // already validated coverage.
                    let product: Product = self
                        .tag_registry
                        .get(name)
                        .expect("assert_tag_registry_covers should have caught this")
                        .tagged()
                        .as_product()
                        .expect("struct name should map to a Product");
                    self.generate_struct(name, fields, product);
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

        /// Regular structs pack into a map. `product` is the
        /// `MsgpackTagged` metadata for this shape: either the top-level
        /// type's entry (for ordinary structs) or the variant's payload
        /// (when recursing into a struct-variant). It carries the
        /// per-field u8 tag the int-keyed dispatch branch needs.
        fn generate_struct(&mut self, name: &str, fields: &[Named<Format>], product: Product) {
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
                // Dispatch on wire shape:
                //   * MAP with INT first key → `Format::MsgpackTagged`
                //     (int-keyed: dispatch by u8 tag).
                //   * MAP with STR keys → legacy `Format::Msgpack`
                //     (string-keyed: look up by field name).
                //   * ARRAY → legacy `Format::MsgpackCompact` and also
                //     `Format::MsgpackTagged::Array` (both emit fields in
                //     tag-ascending order under the codegen invariant
                //     `tag_order_matches_source`, so positional indexing
                //     decodes both correctly).
                // cSpell:disable
                let mut body = format!(
                    r#"
    std::string name = "{name}";
    if (o.type == msgpack::type::MAP) {{
        if (Helpers::is_int_keyed_map(o)) {{
            Helpers::int_map_dispatch(o, name, [&](uint8_t tag, msgpack::object const& val) {{
                switch (tag) {{"#
                );
                // cSpell:enable
                for field in fields {
                    let field_name = &field.name;
                    let tag = product.tag_for(field_name).unwrap_or_else(|| {
                        panic!(
                            "field {field_name:?} of {name:?} is not in the MsgpackTagged \
                             Product — serde-reflection and #[derive(MsgpackTagged)] disagree \
                             on which fields are on the wire",
                        )
                    });
                    if is_unit(field) {
                        // Field is `()` in Rust / `std::monostate` in C++.
                        // The wire still carries a value at this tag; we
                        // consume it without binding to any C++ member,
                        // mirroring `deserialize_unit`'s skip-any semantics.
                        // cSpell:disable
                        body.push_str(&format!(
                            r#"
                    case {tag}:
                        // Field is `std::monostate` — wire entry intentionally discarded.
                        break;"#
                        ));
                        // cSpell:enable
                        continue;
                    }
                    // cSpell:disable
                    body.push_str(&format!(
                        r#"
                    case {tag}:
                        Helpers::convert_or_throw(val, name, "{field_name}", {field_name});
                        break;"#
                    ));
                    // cSpell:enable
                }
                // Reserved tags: retired in the Rust type via
                // `#[tagged(reserved(...))]`. The Rust decoder skips
                // them silently regardless of `allow_unknown_tags`; do
                // the same here, with an explicit case so the strict
                // default below can distinguish "retired" from "never
                // declared".
                if !product.reserved.is_empty() {
                    for &reserved_tag in product.reserved {
                        body.push_str(&format!(
                            "
                    case {reserved_tag}:"
                        ));
                    }
                    body.push_str(
                        r#"
                        // Reserved tag (retired field) — skip silently.
                        break;"#,
                    );
                }
                // Default branch: strict by default (reject unknown tags
                // so a C++ reviewer can see the type-level intent). Opt
                // into silent forward-compat with `#[tagged(allow_unknown_tags)]`
                // on the Rust struct.
                if product.allow_unknown_tags {
                    body.push_str(
                        r#"
                    default:
                        // `#[tagged(allow_unknown_tags)]` on the Rust side:
                        // silently skip any tag we don't recognize.
                        break;"#,
                    );
                } else {
                    // cSpell:disable
                    body.push_str(&format!(
                        r#"
                    default:
                        std::cerr << val << std::endl;
                        throw_or_abort("unknown tag for {name}: " + std::to_string(tag));"#
                    ));
                    // cSpell:enable
                }
                // cSpell:disable
                body.push_str(
                    r#"
                }
            });
        } else {"#,
                );
                // cSpell:enable
                // String-keyed map branch (`Format::Msgpack`): the
                // lookup is by field name, so extra wire keys would
                // otherwise pass unnoticed. Same `active + reserved`
                // ceiling and same `allow_unknown_tags` opt-out as the
                // ARRAY branch.
                if !product.allow_unknown_tags {
                    body.push_str(&format!(
                        r#"
            Helpers::check_size(o.via.map.size, name, {active}, {reserved});"#,
                        active = fields.len(),
                        reserved = product.reserved.len(),
                    ));
                }
                // cSpell:disable
                body.push_str(
                    r#"
            auto kvmap = Helpers::make_kvmap(o, name);"#,
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
        }
    } else if (o.type == msgpack::type::ARRAY) {
        auto array = o.via.array;",
                );
                // Cap the array length: an older reader of a newer wire
                // (forward-compat) should reject extra trailing items
                // unless the type opts into `#[tagged(allow_unknown_tags)]`;
                // a newer reader of an older wire that retired trailing
                // fields (backward-compat) gets `reserved.len()` extra
                // trailing positions tolerated either way.
                //
                // Under-length wires are caught downstream by
                // `Helpers::conv_fld_from_array` (it errors when its
                // index is past `array.size`), so we only need the
                // upper bound here.
                if !product.allow_unknown_tags {
                    body.push_str(&format!(
                        r#"
        Helpers::check_size(array.size, name, {active}, {reserved});"#,
                        active = fields.len(),
                        reserved = product.reserved.len(),
                    ));
                }
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
        fn generate_tuple(&self, _name: &str, _formats: &[Format]) {
            unimplemented!("Until we have a tuple enum in our schema we don't need this.");
        }

        /// Enums serialize as a single element map keyed by the variant type name.
        fn generate_enum(&mut self, name: &str, variants: &BTreeMap<u32, Named<VariantFormat>>) {
            // Look up the `Sum` upfront — both the per-variant recursion
            // (each variant's payload Product comes from here) and the
            // int-keyed `msgpack_unpack` body below need it. Coverage is
            // already enforced by `assert_tag_registry_covers`.
            let sum: Sum = self
                .tag_registry
                .get(name)
                .expect("assert_tag_registry_covers should have caught this")
                .tagged()
                .as_sum()
                .expect("enum name should map to a Sum");

            // Resolve a variant's payload Product by its serde name. The
            // `MsgpackTagged` macro and serde-derive agree on names, so
            // a miss here is an internal bug.
            let payload_for = |variant_name: &str| -> Product {
                sum.variants.iter().find(|v| v.name == variant_name).map_or_else(
                    || {
                        panic!(
                            "variant {variant_name:?} of enum {name:?} is not in the \
                             MsgpackTagged Sum — serde-reflection and \
                             #[derive(MsgpackTagged)] disagree on variants",
                        )
                    },
                    |v| v.payload,
                )
            };

            // Recurse into the variants
            self.namespace.push(name.to_string());
            for variant in variants.values() {
                let payload = payload_for(&variant.name);
                self.generate_variant(&variant.name, &variant.value, payload);
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

            // Unpack the enum:
            //   * MAP with INT key → `Format::MsgpackTagged` variant
            //     (int-keyed dispatch on the u8 tag).
            //   * MAP with STR key → legacy `Format::Msgpack` variant
            //     (string-keyed dispatch on variant name).
            //   * STR top-level → legacy `Format::MsgpackCompact` unit
            //     variant (bare variant-name string).
            // The 1-entry-size invariant of the MAP cases is enforced
            // upfront.
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
    if (Helpers::is_int_keyed_map(o)) {{
        // `Format::MsgpackTagged` — int-keyed variant.
        uint8_t tag;
        try {{
            o.via.map.ptr[0].key.convert(tag);
        }} catch (const msgpack::type_error&) {{
            std::cerr << o << std::endl;
            throw_or_abort("expected u8 variant tag for enum '{name}'");
        }}
        switch (tag) {{"#
                );
                // cSpell:enable

                for v in variants.values() {
                    let variant = &v.name;
                    let tag = sum.variants.iter().find(|reg_v| reg_v.name == variant).map_or_else(
                        || {
                            panic!(
                                "variant {variant:?} of enum {name:?} is not in the MsgpackTagged \
                                 Sum — serde-reflection and #[derive(MsgpackTagged)] disagree",
                            )
                        },
                        |reg_v| reg_v.tag,
                    );
                    if matches!(v.value, VariantFormat::Unit) {
                        // Unit variants carry a `nil` payload that the
                        // variant's empty `msgpack_unpack` would accept,
                        // but constructing the value and assigning is
                        // simpler and avoids reading `o.via.map.ptr[0].val`
                        // (which Barretenberg's `-Werror=unused-variable`
                        // flags when all variants of an enum are unit).
                        body.push_str(&format!(
                            r#"
            case {tag}: {{
                {variant} v;
                value = v;
                break;
            }}"#
                        ));
                    } else {
                        // cSpell:disable
                        body.push_str(&format!(
                            r#"
            case {tag}: {{
                {variant} v;
                try {{
                    o.via.map.ptr[0].val.convert(v);
                }} catch (const msgpack::type_error&) {{
                    std::cerr << o << std::endl;
                    throw_or_abort("error converting into enum variant '{name}::{variant}'");
                }}
                value = v;
                break;
            }}"#
                        ));
                        // cSpell:enable
                    }
                }

                // Reserved variant tags: retired variants on the Rust side
                // (`#[tagged(reserved(...))]` on the enum). If the enum
                // marks a unit variant with `#[tagged(on_reserved)]`,
                // legacy wires carrying a retired tag route to that
                // fallback; otherwise we throw with a "retired" message
                // that's distinguishable from the "unknown" case below.
                let lookup_unit_variant = |fallback_tag: u8| -> &str {
                    sum.variants.iter().find(|v| v.tag == fallback_tag).map_or_else(
                        || {
                            panic!(
                                "MsgpackTagged Sum for enum {name:?} declares a fallback \
                                 tag {fallback_tag} that doesn't match any registered variant",
                            )
                        },
                        |v| v.name,
                    )
                };
                if !sum.reserved.is_empty() {
                    for &reserved_tag in sum.reserved {
                        body.push_str(&format!(
                            "
            case {reserved_tag}:"
                        ));
                    }
                    if let Some(fallback_tag) = sum.on_reserved_tag {
                        let fallback_name = lookup_unit_variant(fallback_tag);
                        // cSpell:disable
                        body.push_str(&format!(
                            r#" {{
                // `#[tagged(on_reserved)]` fallback: retired tag routes to
                // the designated unit variant (payload discarded).
                {fallback_name} v;
                value = v;
                break;
            }}"#
                        ));
                        // cSpell:enable
                    } else {
                        // cSpell:disable
                        body.push_str(&format!(
                            r#"
                std::cerr << o << std::endl;
                throw_or_abort("retired variant tag for enum '{name}' (declare `#[tagged(on_reserved)]` on a unit variant to route legacy data here): " + std::to_string(tag));"#
                        ));
                        // cSpell:enable
                    }
                }

                // Default branch for unknown variant tags (not active,
                // not reserved). Routes to `#[tagged(on_unknown)]` if
                // set — the forward-compat opt-in for newer producers
                // introducing variants this code doesn't know about.
                if let Some(fallback_tag) = sum.on_unknown_tag {
                    let fallback_name = lookup_unit_variant(fallback_tag);
                    // cSpell:disable
                    body.push_str(&format!(
                        r#"
            default: {{
                // `#[tagged(on_unknown)]` fallback: any tag we don't recognize
                // (and isn't reserved) routes here.
                {fallback_name} v;
                value = v;
                break;
            }}"#
                    ));
                    // cSpell:enable
                } else {
                    // cSpell:disable
                    body.push_str(&format!(
                        r#"
            default:
                std::cerr << o << std::endl;
                throw_or_abort("unknown '{name}' enum variant tag: " + std::to_string(tag));"#
                    ));
                    // cSpell:enable
                }
                // cSpell:disable
                body.push_str(
                    r#"
        }
    } else {"#,
                );
                // cSpell:enable
                // Reuse the existing format-string body for the legacy
                // string-keyed dispatch path. The `format!(name = ...)`
                // substitution from the previous body is preserved in the
                // already-appended text; the section below is plain C++
                // with no further interpolation.
                body.push_str(&format!(
                    r#"
        // `Format::Msgpack` (MAP, string-keyed) or `Format::MsgpackCompact`
        // unit variant (bare STR) — both dispatch on the variant name.
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
                ));
                // cSpell:enable

                for (i, v) in variants {
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
        }}
    }}"#
                ));
                // cSpell:enable

                body
            });
        }

        /// Generate msgpack code for nested enum variants. `payload` is
        /// the variant's `MsgpackTagged` `Product`, looked up from the
        /// parent enum's `Sum`. Only the `Struct` variant case consults
        /// it (it carries the int-keyed dispatch metadata for that
        /// variant's named fields); the other cases are tag-irrelevant
        /// (unit / passthrough newtype / unimplemented tuple).
        fn generate_variant(&mut self, name: &str, variant: &VariantFormat, payload: Product) {
            match variant {
                VariantFormat::Variable(_) => {
                    unreachable!("internal construct")
                }
                VariantFormat::Unit => self.generate_unit_struct(name),
                VariantFormat::NewType(_format) => self.generate_newtype(name),
                VariantFormat::Tuple(formats) => self.generate_tuple(name, formats),
                VariantFormat::Struct(fields) => self.generate_struct(name, fields, payload),
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
            if self.config.no_pack {
                return;
            }
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
