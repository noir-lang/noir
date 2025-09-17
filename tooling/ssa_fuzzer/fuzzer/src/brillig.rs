#![no_main]

mod abstract_vm_integration;
pub(crate) mod fuzz_lib;
mod mutations;
mod utils;

use abstract_vm_integration::{
    AbstractVMComparisonResult, ExecuteAbstractVMBytecode,
    TranspileBrilligBytecodeToAbstractVMBytecode, compare_with_abstract_vm,
};
use bincode::serde::{borrow_decode_from_slice, encode_to_vec};
use fuzz_lib::{
    fuzz_target_lib::fuzz_target,
    fuzzer::FuzzerData,
    options::{FuzzerCommandOptions, FuzzerMode, FuzzerOptions, InstructionOptions},
};
use libfuzzer_sys::Corpus;
use mutations::mutate;
use noirc_driver::CompileOptions;
use noirc_evaluator::ssa::ir::function::RuntimeType;
use noirc_frontend::monomorphization::ast::InlineType as FrontendInlineType;
use rand::{SeedableRng, rngs::StdRng};
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};

lazy_static::lazy_static! {
    static ref SIMULATOR_BIN_PATH: String = std::env::var("SIMULATOR_BIN_PATH").expect("SIMULATOR_BIN_PATH must be set");
    static ref TRANSPILER_BIN_PATH: String = std::env::var("TRANSPILER_BIN_PATH").expect("TRANSPILER_BIN_PATH must be set");
}

/// Placeholder for creating a base contract artifact to feed to the transpiler
fn create_base_contract_artifact() -> Value {
    json!({
        "noir_version": "1.0.0-beta.11+a92d049c8771332a383aec07474691764c4d90f0-aztec",
        "name": "AvmTest",
        "functions": [{
            "name": "main2",
            "hash": "9106907505563584043",
            "is_unconstrained": true,
            "custom_attributes": ["public"],
            "abi": {
                "parameters": [{
                    "name": "a",
                    "type": {
                        "kind": "integer",
                        "sign": "unsigned",
                        "width": 64
                    },
                    "visibility": "private"
                }],
                "return_type": {
                    "abi_type": {
                        "kind": "integer",
                        "sign": "unsigned",
                        "width": 64
                    },
                    "visibility": "public"
                },
                "error_types": {
                    "17843811134343075018": {
                        "error_kind": "string",
                        "string": "Stack too deep"
                    }
                }
            },
            "bytecode": "",
            "debug_symbols": "dVDNCoQgEH6XOXtIoVp6lYgwm0IQFdOFJXz3HaN228Ne5pvx+5GZHWac0jpqu7gNun6HKWhj9Doap2TUztLrDlUpvIGOM+AtQc4MLsUYA2IR3CwU5GVAG6GzyRgGT2nSIdq8tAdGGYitGKCdCSlw0QZLl9nXXf23cl43j9NOfSs+EaLOeaBJKh1+FsklLWg5GTzHJVl1Y+PLX8x1CB+cwjkFLEm3a1DtRcVEPeTy2xs=",
            "expression_width": {"Bounded": {"width": 4}}
        }],
        "outputs": {},
        "file_map": {}
    })
}

fn transpile(bytecode_base64: String) -> Result<String, String> {
    let start_time = std::time::Instant::now();
    let mut contract = create_base_contract_artifact();

    // Set the bytecode in the contract
    if let Some(functions) = contract.get_mut("functions").and_then(|f| f.as_array_mut()) {
        if let Some(function) = functions.get_mut(0) {
            if let Some(obj) = function.as_object_mut() {
                obj.insert("bytecode".to_string(), Value::String(bytecode_base64));
            }
        }
    }

    let contract_json = serde_json::to_string(&contract)
        .map_err(|e| format!("Failed to serialize contract: {e}"))?;

    fs::write("contract_artifact.json", contract_json)
        .map_err(|e| format!("Failed to write contract artifact: {e}"))?;
    let output = Command::new(TRANSPILER_BIN_PATH.as_str())
        .arg("contract_artifact.json")
        .arg("output.json")
        .output()
        .map_err(|e| format!("Failed to execute transpiler: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Transpiler failed: {stderr}"));
    }

    log::debug!("Transpiler output: {output:?}");

    let output_content = fs::read_to_string("output.json")
        .map_err(|e| format!("Failed to read output.json: {e}"))?;

    let output_json: Value = serde_json::from_str(&output_content)
        .map_err(|e| format!("Failed to parse output.json: {e}"))?;

    let bytecode = output_json
        .get("functions")
        .and_then(|f| f.as_array())
        .and_then(|arr| arr.first())
        .and_then(|func| func.get("bytecode"))
        .and_then(|bc| bc.as_str())
        .ok_or("Failed to extract bytecode from output")?;

    log::debug!("Transpilation took: {:?}", start_time.elapsed());
    Ok(bytecode.to_string())
}

/// Global simulator process that stays alive across calls
static SIMULATOR_PROCESS: OnceLock<Mutex<Option<SimulatorProcess>>> = OnceLock::new();

struct SimulatorProcess {
    child: Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl Drop for SimulatorProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl SimulatorProcess {
    fn new() -> Result<Self, String> {
        let mut child = Command::new("node")
            .arg(SIMULATOR_BIN_PATH.as_str())
            .env("LOG_LEVEL", "silent")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start simulator process: {e}"))?;

        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
        let stdout = BufReader::new(stdout);

        Ok(SimulatorProcess { child, stdin, stdout })
    }

    fn execute(&mut self, bytecode: &str, inputs: &[String]) -> Result<Vec<String>, String> {
        let request = json!({
            "bytecode": bytecode,
            "inputs": inputs
        });

        // Send request
        let request_line = format!(
            "{}\n",
            serde_json::to_string(&request)
                .map_err(|e| format!("Failed to serialize request: {e}"))?
        );
        log::debug!("Simulator request: {request_line}");

        self.stdin
            .write_all(request_line.as_bytes())
            .map_err(|e| format!("Failed to write to simulator: {e}"))?;
        self.stdin.flush().map_err(|e| format!("Failed to flush simulator input: {e}"))?;

        // Read response
        log::debug!("Reading response from simulator");
        let mut response_line = String::new();
        self.stdout
            .read_line(&mut response_line)
            .map_err(|e| format!("Failed to read from simulator: {e}"))?;

        log::debug!("Simulator response: {response_line}");

        let response: Value = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Failed to parse simulator response: {e}"))?;

        if let Some(reverted) = response.get("reverted").and_then(|v| v.as_bool()) {
            if reverted {
                return Err("Execution reverted".to_string());
            }
        }

        let outputs = response
            .get("output")
            .and_then(|v| v.as_array())
            .ok_or("Missing output in simulator response")?;

        let result: Result<Vec<String>, String> = outputs
            .iter()
            .map(|v| v.as_str().ok_or("Invalid output format".to_string()).map(|s| s.to_string()))
            .collect();

        result.map_err(|e| format!("Failed to parse output array: {e}"))
    }
}

fn recreate_simulator() -> Result<(), String> {
    let mutex = SIMULATOR_PROCESS.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().map_err(|e| format!("Failed to lock simulator mutex: {e}"))?;
    *guard = Some(SimulatorProcess::new()?);
    Ok(())
}

fn get_or_create_simulator()
-> Result<std::sync::MutexGuard<'static, Option<SimulatorProcess>>, String> {
    let mutex = SIMULATOR_PROCESS.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().map_err(|e| format!("Failed to lock simulator mutex: {e}"))?;

    if guard.is_none() {
        *guard = Some(SimulatorProcess::new()?);
    }

    Ok(guard)
}

/// Initialize the simulator process
fn initialize() {
    let _mutex = get_or_create_simulator().expect("Failed to create simulator");
}

/// Simulate Abstract VM bytecode execution
fn simulate_abstract_vm(bytecode: String, inputs: Vec<String>) -> Result<Vec<String>, String> {
    log::debug!(
        "Simulating Abstract VM with bytecode length: {}, inputs: {:?}",
        bytecode.len(),
        inputs
    );

    let mut simulator_guard = get_or_create_simulator()?;
    let simulator = simulator_guard.as_mut().ok_or("Simulator not initialized")?;

    simulator.execute(&bytecode, &inputs)
}

const MAX_EXECUTION_TIME_TO_KEEP_IN_CORPUS: u64 = 10;
const INLINE_TYPE: FrontendInlineType = FrontendInlineType::Inline;
const BRILLIG_RUNTIME: RuntimeType = RuntimeType::Brillig(INLINE_TYPE);
const TARGET_RUNTIMES: [RuntimeType; 1] = [BRILLIG_RUNTIME];

libfuzzer_sys::fuzz_target!(
    init: {
        println!("Initializing simulator process");
        initialize();
    }, |data: &[u8]| -> Corpus {
    let _ = env_logger::try_init();

    let mut compile_options = CompileOptions::default();
    if let Ok(triage_value) = std::env::var("TRIAGE") {
        match triage_value.as_str() {
            "FULL" => compile_options.show_ssa = true,
            "FINAL" => {
                compile_options.show_ssa_pass =
                    vec!["Dead Instruction Elimination (3)".to_string()];
            }
            "FIRST_AND_FINAL" => {
                compile_options.show_ssa_pass = vec![
                    "After Removing Unreachable Functions (1)".to_string(),
                    "Dead Instruction Elimination (3)".to_string(),
                ];
            }
            _ => (),
        }
    }

    // You can disable some instructions with bugs that are not fixed yet
    let modes = vec![FuzzerMode::NonConstant];
    let instruction_options = InstructionOptions {
        array_get_enabled: false,
        array_set_enabled: false,
        ecdsa_secp256k1_enabled: false,
        ecdsa_secp256r1_enabled: false,
        blake2s_hash_enabled: false,
        blake3_hash_enabled: false,
        aes128_encrypt_enabled: false,
        field_to_bytes_to_field_enabled: false,
        point_add_enabled: false,
        multi_scalar_mul_enabled: false,
        shl_enabled: false,
        shr_enabled: false,
        ..InstructionOptions::default()
    };
    let fuzzer_command_options =
        FuzzerCommandOptions { loops_enabled: false, ..FuzzerCommandOptions::default() };
    let options = FuzzerOptions {
        compile_options,
        instruction_options,
        modes,
        fuzzer_command_options,
        ..FuzzerOptions::default()
    };
    let fuzzer_data = borrow_decode_from_slice(data, bincode::config::legacy())
        .unwrap_or((FuzzerData::default(), 1337))
        .0;
    let start = std::time::Instant::now();
    let fuzzer_output = fuzz_target(fuzzer_data, TARGET_RUNTIMES.to_vec(), options);

    let transpiler: TranspileBrilligBytecodeToAbstractVMBytecode = Box::new(transpile);
    let simulator: ExecuteAbstractVMBytecode = Box::new(simulate_abstract_vm);

    match compare_with_abstract_vm(&fuzzer_output, &transpiler, &simulator) {
        AbstractVMComparisonResult::Match => {
            log::debug!("Abstract VM and Brillig outputs match");
        }
        AbstractVMComparisonResult::Mismatch { brillig_outputs, abstract_vm_outputs } => {
            log::error!("Abstract VM and Brillig outputs mismatch!");
            log::error!("Brillig outputs: {brillig_outputs:?}");
            log::error!("Abstract VM outputs: {abstract_vm_outputs:?}");
            panic!("Abstract VM vs Brillig mismatch detected");
        }
        AbstractVMComparisonResult::TranspilerError(err) => {
            panic!("Transpiler error: {err}");
        }
        AbstractVMComparisonResult::SimulatorError(err) => {
            log::error!("Simulator error: {err}");
            if err.contains("EOF while parsing a value") {
                log::warn!("Recreating simulator");
                recreate_simulator().expect("Failed to recreate simulator");
            } else {
                panic!("Simulator error: {err}");
            }
        }
        AbstractVMComparisonResult::BrilligCompilationError(err) => {
            log::debug!("Brillig compilation error: {err}");
        }
    }

    if start.elapsed().as_secs() > MAX_EXECUTION_TIME_TO_KEEP_IN_CORPUS {
        return Corpus::Reject;
    }
    Corpus::Keep
});

libfuzzer_sys::fuzz_mutator!(|data: &mut [u8], _size: usize, max_size: usize, seed: u32| {
    let mut rng = StdRng::seed_from_u64(u64::from(seed));
    let mut new_fuzzer_data: FuzzerData = borrow_decode_from_slice(data, bincode::config::legacy())
        .unwrap_or((FuzzerData::default(), 1337))
        .0;
    mutate(&mut new_fuzzer_data, &mut rng);
    let new_bytes = encode_to_vec(&new_fuzzer_data, bincode::config::legacy()).unwrap();
    if new_bytes.len() > max_size {
        return 0;
    }
    data[..new_bytes.len()].copy_from_slice(&new_bytes);
    new_bytes.len()
});
