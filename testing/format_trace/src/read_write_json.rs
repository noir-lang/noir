use serde_json::Value;
use std::fs;

pub fn serialize_file(src_filename: String) -> Value {
    let file_content = fs::read_to_string(src_filename).expect("Failed to read the file");

    serde_json::from_str(&file_content)
        .expect("Failed to parse the json file that was given as a source")
}

pub fn save_to_file(dest_filename: String, json_string: String) {
    let mut json_string_copy = json_string.clone();
    if !json_string_copy.ends_with('\n') {
        json_string_copy.push('\n');
    }
    fs::write(&dest_filename, json_string_copy).unwrap_or_else(|_| {
        panic!("Unable to write to destination file: {}", dest_filename.as_str())
    });
}
