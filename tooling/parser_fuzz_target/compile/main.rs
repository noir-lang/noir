#[macro_use]
extern crate afl;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::prelude::*;
use serde_json;
use tempfile::tempdir;

test_binary::build_test_binary_once!(mock_backend, "../backend_interface/test-binaries");

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let tmp_dir = tempdir().unwrap();

            let v_result: serde_json::Result<serde_json::Value> = serde_json::from_str(s);
            if v_result.is_err() {
                return;
            }
            match v_result.unwrap() {
                serde_json::Value::Object(m) => {
                    for (key, value) in m.iter() {
                        match value {
                            serde_json::Value::String(value_str) => {

                                let path = Path::new(key);
                                match path.extension().and_then(std::ffi::OsStr::to_str) {
                                    Some("json") => (),
                                    Some("nr") => (),
                                    Some("toml") => (),
                                    _ => continue,
                                }
                                if !path.is_relative() {
                                    continue
                                }
                                if let Some(path_parent) = path.parent() {
                                    std::fs::create_dir_all(tmp_dir.path().join(path_parent)).unwrap();
                                }
                                let file_path = tmp_dir.path().join(path);
                                let mut tmp_file = File::create(file_path).unwrap();
                                write!(tmp_file, "{}", value_str).unwrap();

                            },
                            _ => continue,
                        }
                    }
                },
                _ => return (),
            }

            let test_program_dir = PathBuf::from("{tmp_dir}");

            let mut cmd = Command::cargo_bin("nargo").unwrap();
            cmd.env("NARGO_BACKEND_PATH", path_to_mock_backend());
            cmd.arg("--program-dir").arg(test_program_dir);
            cmd.arg("execute").arg("--force");

        }
    });
}
