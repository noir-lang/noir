use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("execute.rs");
    let mut test_file = File::create(destination).unwrap();

    // Try to find the directory that Cargo sets when it is running; otherwise fallback to assuming the CWD
    // is the root of the repository and append the crate path
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => std::env::current_dir().unwrap().join("tooling").join("nargo_fmt"),
    };
    let test_dir = manifest_dir.join("tests");

    generate_formatter_tests(&mut test_file, &test_dir);
}

fn generate_formatter_tests(test_file: &mut File, test_data_dir: &Path) {
    let inputs_dir = test_data_dir.join("input");
    let outputs_dir = test_data_dir.join("expected");

    let test_case_files =
        fs::read_dir(inputs_dir).unwrap().flatten().filter(|c| c.path().is_file());

    for file in test_case_files {
        let file_path = file.path();
        let file_name = file_path.file_name().unwrap();
        let test_name = file_path.file_stem().unwrap().to_str().unwrap();

        if test_name.contains('-') {
            panic!(
                "Invalid test directory: {test_name}. Cannot include `-`, please convert to `_`"
            );
        };

        let input_source_path = file.path();
        let input_source = std::fs::read_to_string(input_source_path).unwrap();

        let config = input_source
            .lines()
            .flat_map(|line| line.strip_prefix("//@"))
            .collect::<Vec<_>>()
            .join("\n");

        let output_source_path = outputs_dir.join(file_name).display().to_string();
        let output_source =
            std::fs::read_to_string(output_source_path.clone()).unwrap_or_else(|_| {
                panic!("expected output source at {:?} was not found", &output_source_path)
            });

        write!(
            test_file,
            r##"
    #[test]
    fn format_{test_name}() {{
        let input = r#"{input_source}"#;
        let expected_output = r#"{output_source}"#;


        let (parsed_module, _errors) = noirc_frontend::parse_program(input);

        let config = nargo_fmt::Config::of("{config}").unwrap();
        let fmt_text = nargo_fmt::format(input, parsed_module, &config);

        if std::env::var("UPDATE_EXPECT").is_ok() {{
            std::fs::write("{output_source_path}", fmt_text.clone()).unwrap();
        }}

        similar_asserts::assert_eq!(fmt_text, expected_output);
    }}
            "##
        )
        .expect("Could not write templated test file.");

        write!(
            test_file,
            r##"
        #[test]
        fn format_idempotent_{test_name}() {{
            let expected_output = r#"{output_source}"#;

            let (parsed_module, _errors) = noirc_frontend::parse_program(expected_output);

            let config = nargo_fmt::Config::of("{config}").unwrap();
            let fmt_text = nargo_fmt::format(expected_output, parsed_module, &config);

            similar_asserts::assert_eq!(fmt_text, expected_output);
        }}
                "##
        )
        .expect("Could not write templated test file.");
    }
}
