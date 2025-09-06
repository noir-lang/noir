#[cfg(feature = "redis-support")]
pub(crate) mod redis;

use crate::fuzz_lib::fuzzer::FuzzerOutput;
use base64::{Engine as _, engine::general_purpose};

// TODO(sn): change to impl Serialize for FuzzerOutput with brillig inputs and outputs
// TODO(sn): legacy for https://github.com/AztecProtocol/aztec-packages/tree/next/barretenberg/security/ssa_fuzzer_programs_proving
//  "program" field  -> "acir_program"; witness_stack -> acir_witness_stack
pub(crate) fn fuzzer_output_to_json(fuzzer_output: FuzzerOutput) -> String {
    let acir_program = &fuzzer_output.acir_program;
    let acir_program_json = serde_json::to_string(acir_program).unwrap();
    let mut acir_program_json: serde_json::Value =
        serde_json::from_str(&acir_program_json).unwrap();

    let brillig_program = &fuzzer_output.brillig_program;
    let brillig_program_json = serde_json::to_string(brillig_program).unwrap();
    let mut brillig_program_json: serde_json::Value =
        serde_json::from_str(&brillig_program_json).unwrap();

    // Noir program compiles into json, storing bytecode of the program
    // to "bytecode" field, but default `CompiledProgram` serializer
    // serializes it to "program" field.
    if let Some(acir_program_value) = acir_program_json.get("program").cloned() {
        acir_program_json.as_object_mut().unwrap().remove("program");
        acir_program_json
            .as_object_mut()
            .unwrap()
            .insert("bytecode".to_string(), acir_program_value);
    }
    if let Some(brillig_program_value) = brillig_program_json.get("program").cloned() {
        brillig_program_json.as_object_mut().unwrap().remove("program");
        brillig_program_json
            .as_object_mut()
            .unwrap()
            .insert("bytecode".to_string(), brillig_program_value);
    }
    let witness_map_bytes = fuzzer_output.witness_stack_acir.serialize().unwrap();
    let witness_map_b64 = general_purpose::STANDARD.encode(&witness_map_bytes);

    let brillig_inputs = fuzzer_output.get_input_values_brillig();
    let brillig_outputs = fuzzer_output.get_return_values_brillig();

    let mut output_json = serde_json::Map::new();
    output_json.insert("program".to_string(), acir_program_json.clone());
    output_json.insert("brillig_program".to_string(), brillig_program_json.clone());
    output_json.insert(
        "brillig_inputs".to_string(),
        serde_json::Value::Array(
            brillig_inputs.iter().map(|x| serde_json::Value::String(x.to_string())).collect(),
        ),
    );
    output_json.insert(
        "brillig_outputs".to_string(),
        serde_json::Value::Array(
            brillig_outputs.iter().map(|x| serde_json::Value::String(x.to_string())).collect(),
        ),
    );
    output_json.insert("witness_map_b64".to_string(), serde_json::Value::String(witness_map_b64));
    serde_json::to_string(&output_json).unwrap()
}

#[cfg(feature = "redis-support")]
pub(crate) fn push_fuzzer_output_to_redis_queue(
    queue_name: &str,
    test_id: String,
    fuzzer_output: FuzzerOutput,
) -> redis::RedisResult<String> {
    let json = fuzzer_output_to_json(fuzzer_output);
    let mut json_value: serde_json::Value = serde_json::from_str(&json).unwrap();
    if let serde_json::Value::Object(ref mut map) = json_value {
        map.insert("test_id".to_string(), serde_json::Value::String(test_id));
    }
    let json_with_test_id = serde_json::to_string(&json_value).unwrap();

    redis::push_to_redis_queue(queue_name, &json_with_test_id)?;

    Ok(json_with_test_id)
}
