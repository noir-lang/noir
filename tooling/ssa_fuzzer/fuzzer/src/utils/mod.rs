pub mod redis;

use crate::fuzz_lib::fuzzer::FuzzerOutput;
use base64::{Engine as _, engine::general_purpose};
use serde_json;

fn fuzzer_output_to_json(fuzzer_output: FuzzerOutput) -> String {
    let program = fuzzer_output.program;
    let program_json = serde_json::to_string(&program).unwrap();
    let mut program_json: serde_json::Value = serde_json::from_str(&program_json).unwrap();
    // bb takes the program from "bytecode" field, but default `CompiledProgram` serializer
    // serializes it to "program" field.
    if let Some(program_value) = program_json.get("program").cloned() {
        program_json.as_object_mut().unwrap().remove("program");
        program_json.as_object_mut().unwrap().insert("bytecode".to_string(), program_value);
    }
    let witness_map_bytes = fuzzer_output.witness_map.serialize().unwrap();
    let witness_map_b64 = general_purpose::STANDARD.encode(&witness_map_bytes);
    let mut output_json = serde_json::Map::new();
    output_json.insert("program".to_string(), program_json.clone());
    output_json.insert("witness_map_b64".to_string(), serde_json::Value::String(witness_map_b64));
    let json = serde_json::to_string(&output_json).unwrap();
    json
}

pub fn push_fuzzer_output_to_queue(
    test_id: String,
    fuzzer_output: FuzzerOutput,
) -> redis::RedisResult<String> {
    let json = fuzzer_output_to_json(fuzzer_output);
    let mut json_value: serde_json::Value = serde_json::from_str(&json).unwrap();
    if let serde_json::Value::Object(ref mut map) = json_value {
        map.insert("test_id".to_string(), serde_json::Value::String(test_id));
    }
    let json_with_test_id = serde_json::to_string(&json_value).unwrap();

    // Use the Redis module to push to queue
    redis::push_to_redis_queue("fuzzer_output", &json_with_test_id)?;

    Ok(json_with_test_id)
}
