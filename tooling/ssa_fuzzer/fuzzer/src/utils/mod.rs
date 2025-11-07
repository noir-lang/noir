// TODO(sn): separate to features
#![allow(dead_code)]
pub(crate) mod redis;
use crate::fuzz_lib::fuzzer::FuzzerOutput;
use base64::{Engine as _, engine::general_purpose};

// TODO(sn): change to impl Serialize for FuzzerOutput with brillig inputs and outputs
pub(crate) fn fuzzer_output_to_json(fuzzer_output: FuzzerOutput) -> String {
    let program = &fuzzer_output.program;
    let program_json = serde_json::to_string(program).unwrap();
    let mut program_json: serde_json::Value = serde_json::from_str(&program_json).unwrap();

    // Noir program compiles into json, storing bytecode of the program
    // to "bytecode" field, but default `CompiledProgram` serializer
    // serializes it to "program" field.
    if let Some(acir_program_value) = program_json.get("program").cloned() {
        program_json.as_object_mut().unwrap().remove("program");
        program_json.as_object_mut().unwrap().insert("bytecode".to_string(), acir_program_value);
    }

    let witness_map_bytes = fuzzer_output.witness_stack.serialize().unwrap();
    let witness_map_b64 = general_purpose::STANDARD.encode(&witness_map_bytes);

    let inputs = fuzzer_output.get_input_witnesses();
    let outputs = fuzzer_output.get_return_witnesses();

    let mut output_json = serde_json::Map::new();
    output_json.insert("program".to_string(), program_json.clone());
    output_json.insert(
        "inputs".to_string(),
        serde_json::Value::Array(
            inputs.iter().map(|x| serde_json::Value::String(x.to_string())).collect(),
        ),
    );
    output_json.insert(
        "outputs".to_string(),
        serde_json::Value::Array(
            outputs.iter().map(|x| serde_json::Value::String(x.to_string())).collect(),
        ),
    );
    output_json.insert("witness_map_b64".to_string(), serde_json::Value::String(witness_map_b64));
    serde_json::to_string(&output_json).unwrap()
}

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
