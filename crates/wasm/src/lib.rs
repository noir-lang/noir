#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
use acvm::{
    acir::{circuit::Circuit, native_types::Witness},
    FieldElement,
};
use gloo_utils::format::JsValueSerdeExt;
use js_sys::BigInt;
use log::{debug, Level};
use noirc_abi::{input_parser, Abi, MAIN_RETURN_NAME};
use noirc_driver::{CompileOptions, Driver};
use noirc_frontend::graph::{CrateName, CrateType};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf, str::FromStr};
use wasm_bindgen::prelude::*;

mod js_sys_util;

#[derive(Serialize, Deserialize)]
pub struct BuildInfo {
    git_hash: &'static str,
    version: &'static str,
    dirty: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WASMCompileOptions {
    #[serde(default = "default_entry_point")]
    entry_point: String,

    #[serde(default = "default_circuit_name")]
    circuit_name: String,

    // Compile each contract function used within the program
    #[serde(default = "bool::default")]
    contracts: bool,

    #[serde(default)]
    compile_options: CompileOptions,

    #[serde(default)]
    optional_dependencies_set: Vec<String>,

    #[serde(default = "default_log_level")]
    log_level: String,
}

fn default_log_level() -> String {
    String::from("info")
}

fn default_circuit_name() -> String {
    String::from("contract")
}

fn default_entry_point() -> String {
    String::from("main.nr")
}

impl Default for WASMCompileOptions {
    fn default() -> Self {
        Self {
            entry_point: default_entry_point(),
            circuit_name: default_circuit_name(),
            log_level: default_log_level(),
            contracts: false,
            compile_options: CompileOptions::default(),
            optional_dependencies_set: vec![],
        }
    }
}

#[wasm_bindgen]
pub fn init_log_level(level: String) {
    // Set the static variable from Rust
    use std::sync::Once;

    let log_level = Level::from_str(&level).unwrap_or(Level::Error);
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        wasm_logger::init(wasm_logger::Config::new(log_level));
    });
}
const BUILD_INFO: BuildInfo = BuildInfo {
    git_hash: env!("GIT_COMMIT"),
    version: env!("CARGO_PKG_VERSION"),
    dirty: env!("GIT_DIRTY"),
};
pub fn add_noir_lib(driver: &mut Driver, crate_name: &str) {
    let path_to_lib = PathBuf::from(&crate_name).join("lib.nr");
    let library_crate = driver.create_non_local_crate(path_to_lib, CrateType::Library);

    driver.propagate_dep(library_crate, &CrateName::new(crate_name).unwrap());
}

#[wasm_bindgen]
pub fn compile(args: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let options: WASMCompileOptions = if args.is_undefined() || args.is_null() {
        debug!("Initializing compiler with default values.");
        WASMCompileOptions::default()
    } else {
        JsValueSerdeExt::into_serde(&args)
            .unwrap_or_else(|_| panic!("Could not deserialize compile arguments"))
    };

    debug!("Compiler configuration {:?}", &options);

    // For now we default to plonk width = 3, though we can add it as a parameter
    let language = acvm::Language::PLONKCSat { width: 3 };
    let mut driver = noirc_driver::Driver::new(&language);

    let path = PathBuf::from(&options.entry_point);
    driver.create_local_crate(path, CrateType::Binary);

    // We are always adding std lib implicitly. It comes bundled with binary.
    add_noir_lib(&mut driver, "std");

    for dependency in options.optional_dependencies_set {
        add_noir_lib(&mut driver, dependency.as_str());
    }

    driver.check_crate(&options.compile_options).unwrap_or_else(|_| panic!("Crate check failed"));

    if options.contracts {
        let compiled_contracts = driver
            .compile_contracts(&options.compile_options)
            .unwrap_or_else(|_| panic!("Contract compilation failed"));

        // Flatten each contract into a list of its functions, each being assigned a unique name.
        let collected_compiled_programs: Vec<_> = compiled_contracts
            .into_iter()
            .flat_map(|contract| {
                let contract_id = format!("{}-{}", options.circuit_name, &contract.name);
                contract.functions.into_iter().map(move |(function, program)| {
                    let program_name = format!("{}-{}", contract_id, function);
                    (program_name, program)
                })
            })
            .collect();

        <JsValue as JsValueSerdeExt>::from_serde(&collected_compiled_programs).unwrap()
    } else {
        let main =
            driver.main_function().unwrap_or_else(|_| panic!("Could not find main function!"));
        let compiled_program = driver
            .compile_no_check(&options.compile_options, main)
            .unwrap_or_else(|_| panic!("Compilation failed"));

        <JsValue as JsValueSerdeExt>::from_serde(&compiled_program).unwrap()
    }
}

// Deserializes bytes into ACIR structure
#[deprecated(
    note = "we have moved away from this serialization strategy. Call `acir_read_bytes` instead"
)]
#[allow(deprecated)]
#[wasm_bindgen]
pub fn acir_from_bytes(bytes: Vec<u8>) -> JsValue {
    console_error_panic_hook::set_once();
    let circuit = Circuit::from_bytes(&bytes);
    <JsValue as JsValueSerdeExt>::from_serde(&circuit).unwrap()
}

#[deprecated(
    note = "we have moved away from this serialization strategy. Call `acir_write_bytes` instead"
)]
#[allow(deprecated)]
#[wasm_bindgen]
pub fn acir_to_bytes(acir: JsValue) -> Vec<u8> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = JsValueSerdeExt::into_serde(&acir).unwrap();
    circuit.to_bytes()
}

// Deserializes bytes into ACIR structure
#[wasm_bindgen]
pub fn acir_read_bytes(bytes: Vec<u8>) -> JsValue {
    console_error_panic_hook::set_once();
    let circuit = Circuit::read(&*bytes).unwrap();
    <JsValue as JsValueSerdeExt>::from_serde(&circuit).unwrap()
}

#[wasm_bindgen]
pub fn acir_write_bytes(acir: JsValue) -> Vec<u8> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = JsValueSerdeExt::into_serde(&acir).unwrap();
    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();
    bytes
}

#[wasm_bindgen]
pub fn build_info() -> JsValue {
    console_error_panic_hook::set_once();
    <JsValue as JsValueSerdeExt>::from_serde(&BUILD_INFO).unwrap()
}

// Below is all copied from noir_wasm_util branch

fn js_map_to_witness_map(js_map: js_sys::Map) -> BTreeMap<Witness, FieldElement> {
    let mut witness_skeleton: BTreeMap<Witness, FieldElement> = BTreeMap::new();
    for key_result in js_map.keys() {
        let key = key_result.expect("bad key");
        let idx = js_value_to_u32(&key);
        let hex_str = js_map.get(&key).as_string().expect("not a string");
        let field_element = FieldElement::from_hex(&hex_str).expect("bad hex str");
        witness_skeleton.insert(Witness(idx), field_element);
    }
    witness_skeleton
}

fn witness_map_to_js_map(witness_map: BTreeMap<Witness, FieldElement>) -> js_sys::Map {
    let js_map = js_sys::Map::new();
    for (witness, field_value) in witness_map.iter() {
        let js_idx = js_sys::Number::from(witness.0);
        let mut hex_str = "0x".to_owned();
        hex_str.push_str(&field_value.to_hex());
        let js_hex_str = js_sys::JsString::from(hex_str);
        js_map.set(&js_idx, &js_hex_str);
    }
    js_map
}

fn read_circuit(circuit: js_sys::Uint8Array) -> Circuit {
    let circuit: Vec<u8> = circuit.to_vec();
    match Circuit::read(&*circuit) {
        Ok(circuit) => circuit,
        Err(err) => panic!("Circuit read err: {}", err),
    }
}

fn js_value_to_u32(val: &JsValue) -> u32 {
    if let Ok(val_int) = BigInt::new(&val) {
        // Seems like the only way in javascript to get the number of bits
        let num_bits = val_int.to_string(2).unwrap().length();

        if num_bits > 32 {
            panic!("value can not be greater than a 32 bit number, number of bits is {}", num_bits);
        }

        // The following lines of code will convert a BigInt into a u32
        // To do this, we convert it to a Javascript string in base10, then to a utf8 encoded rust string
        // then we parse the rust string as a u32
        let val_as_string = val_int.to_string(10).unwrap().as_string().unwrap();

        let value_u32: u32 = val_as_string.parse().unwrap();
        return value_u32;
    }
    panic!("expected a big integer")
}

#[wasm_bindgen]
pub fn arrange_initial_witness(abi_json_str: String, inputs_json_str: String) -> js_sys::Map {
    console_error_panic_hook::set_once();

    let abi = match serde_json::from_str::<Abi>(&abi_json_str) {
        Ok(abi) => abi,
        Err(err) => panic!("Failed to read ABI: {}", err),
    };
    let parser = input_parser::Format::Json;
    let input_map = match parser.parse(&inputs_json_str, &abi) {
        Ok(input_map) => input_map,
        Err(err) => panic!("Failed to parse input: {}", err),
    };
    let initial_witness = match abi.encode(&input_map, None) {
        Ok(initial_witness) => initial_witness,
        Err(err) => panic!("Failed to arrange initial witness: {}", err),
    };
    js_sys_util::witness_map_to_js_map(initial_witness)
}

#[wasm_bindgen]
pub fn arrange_public_witness(abi_json_str: String, inputs_json_str: String) -> js_sys::Map {
    console_error_panic_hook::set_once();

    let abi = match serde_json::from_str::<Abi>(&abi_json_str) {
        Ok(abi) => abi,
        Err(err) => panic!("Failed to read ABI: {}", err),
    };
    let public_abi = abi.public_abi();
    let parser = input_parser::Format::Json;
    let mut input_map = match parser.parse(&inputs_json_str, &public_abi) {
        Ok(input_map) => input_map,
        Err(err) => panic!("Failed to parse input: {}", err),
    };
    let return_value = input_map.remove(MAIN_RETURN_NAME);
    let public_witness = match public_abi.encode(&input_map, return_value) {
        Ok(public_witness) => public_witness,
        Err(err) => panic!("Failed to arrange initial witness: {}", err),
    };
    js_sys_util::witness_map_to_js_map(public_witness)
}

#[wasm_bindgen]
pub fn select_return_value(abi_json_str: String, intermediate_witness: js_sys::Map) -> String {
    console_error_panic_hook::set_once();

    let intermediate_witness = js_map_to_witness_map(intermediate_witness);
    let abi = match serde_json::from_str::<Abi>(&abi_json_str) {
        Ok(abi) => abi,
        Err(err) => panic!("Failed to read ABI: {}", err),
    };
    let parser = input_parser::Format::Json;
    let return_value = match abi.decode(&intermediate_witness) {
        // `None` indicates that the circuit has no return value -> return a serialised "null"
        Ok((_inputs_map, None)) => return "null".to_owned(),
        Ok((_inputs_map, Some(return_value))) => return_value,
        Err(err) => panic!("Failed to decode intermediate witness: {}", err),
    };
    let input_map = BTreeMap::from([(MAIN_RETURN_NAME.to_owned(), return_value)]);
    match parser.serialize(&input_map) {
        Ok(json_str) => json_str,
        Err(err) => panic!("Failed to serialise return value: {}", err),
    }
}

#[wasm_bindgen]
pub fn select_public_witness(
    circuit: js_sys::Uint8Array,
    intermediate_witness: js_sys::Map,
) -> js_sys::Map {
    console_error_panic_hook::set_once();

    let circuit = read_circuit(circuit);
    let intermediate_witness = js_map_to_witness_map(intermediate_witness);
    let public_witness = circuit
        .public_inputs
        .indices()
        .iter()
        .map(|idx| {
            let witness = Witness(*idx);
            let field_element =
                *intermediate_witness.get(&witness).expect("witness element not found");
            (witness, field_element)
        })
        .collect::<BTreeMap<_, _>>();
    witness_map_to_js_map(public_witness)
}
