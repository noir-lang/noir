use iter_extended::vecmap;
use noirc_artifacts::program::ProgramArtifact;
use prettytable::{Row, row, table};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct InfoReport {
    pub programs: Vec<ProgramInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgramInfo {
    pub package_name: String,
    pub functions: Vec<FunctionInfo>,
    #[serde(skip)]
    pub unconstrained_functions_opcodes: usize,
    pub unconstrained_functions: Vec<FunctionInfo>,
}

impl From<ProgramInfo> for Vec<Row> {
    fn from(program_info: ProgramInfo) -> Self {
        let mut main = vecmap(program_info.functions, |function| {
            row![
                Fm->format!("{}", program_info.package_name),
                Fc->format!("{}", function.name),
                Fc->format!("{}", function.opcodes),
                Fc->format!("{}", program_info.unconstrained_functions_opcodes),
            ]
        });
        main.extend(vecmap(program_info.unconstrained_functions, |function| {
            row![
                Fm->format!("{}", program_info.package_name),
                Fc->format!("{}", function.name),
                Fc->format!("N/A"),
                Fc->format!("{}", function.opcodes),
            ]
        }));
        main
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionInfo {
    pub name: String,
    pub opcodes: usize,
}

pub fn count_opcodes_and_gates_in_program(
    compiled_program: ProgramArtifact,
    package_name: String,
) -> ProgramInfo {
    let functions = compiled_program
        .bytecode
        .functions
        .into_par_iter()
        .map(|function| FunctionInfo {
            name: function.function_name.clone(),
            opcodes: function.opcodes.len(),
        })
        .collect();

    let unconstrained_info: Vec<FunctionInfo> = compiled_program
        .bytecode
        .unconstrained_functions
        .iter()
        .map(|function| FunctionInfo {
            name: function.function_name.clone(),
            opcodes: function.bytecode.len(),
        })
        .collect();
    let unconstrained_functions_opcodes = compiled_program
        .bytecode
        .unconstrained_functions
        .into_par_iter()
        .map(|function| function.bytecode.len())
        .sum();

    ProgramInfo {
        package_name,
        functions,
        unconstrained_functions_opcodes,
        unconstrained_functions: unconstrained_info,
    }
}

fn format_info_report(info_report: &InfoReport, json: bool) -> String {
    if json {
        serde_json::to_string(info_report).unwrap()
    } else {
        // Otherwise format table.
        if info_report.programs.is_empty() {
            String::new()
        } else {
            let mut program_table =
                table!([Fm->"Package", Fm->"Function", Fm->"ACIR Opcodes", Fm->"Brillig Opcodes"]);

            for program_info in &info_report.programs {
                let program_rows: Vec<Row> = program_info.clone().into();
                for row in program_rows {
                    program_table.add_row(row);
                }
            }
            program_table.to_string()
        }
    }
}

pub fn show_info_report(info_report: InfoReport, json: bool) {
    let output = format_info_report(&info_report, json);
    println!("{output}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create test ProgramInfo with specified parameters
    fn create_test_program_info(
        package_name: &str,
        function_count: usize,
        unconstrained_count: usize,
    ) -> ProgramInfo {
        ProgramInfo {
            package_name: package_name.to_string(),
            functions: (0..function_count)
                .map(|i| FunctionInfo { name: format!("function_{}", i), opcodes: 100 + i * 10 })
                .collect(),
            unconstrained_functions_opcodes: 500,
            unconstrained_functions: (0..unconstrained_count)
                .map(|i| FunctionInfo { name: format!("unconstrained_{}", i), opcodes: 50 + i * 5 })
                .collect(),
        }
    }

    #[test]
    fn test_format_info_report_json_output() {
        let info_report =
            InfoReport { programs: vec![create_test_program_info("test_package", 2, 1)] };

        let output = format_info_report(&info_report, true);

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // verify structure
        assert!(parsed["programs"].is_array());
        assert_eq!(parsed["programs"].as_array().unwrap().len(), 1);

        let program = &parsed["programs"][0];
        assert_eq!(program["package_name"], "test_package");
        assert_eq!(program["functions"].as_array().unwrap().len(), 2);
        assert_eq!(program["unconstrained_functions"].as_array().unwrap().len(), 1);

        // verify function details
        assert_eq!(program["functions"][0]["name"], "function_0");
        assert_eq!(program["functions"][0]["opcodes"], 100);
        assert_eq!(program["functions"][1]["name"], "function_1");
        assert_eq!(program["functions"][1]["opcodes"], 110);

        // Verify unconstrained function details
        assert_eq!(program["unconstrained_functions"][0]["name"], "unconstrained_0");
        assert_eq!(program["unconstrained_functions"][0]["opcodes"], 50);
    }

    #[test]
    fn test_format_info_report_table_output() {
        let info_report =
            InfoReport { programs: vec![create_test_program_info("my_package", 1, 1)] };

        let output = format_info_report(&info_report, false);

        // Verify table contains expected strings
        assert!(output.contains("Package"));
        assert!(output.contains("Function"));
        assert!(output.contains("ACIR Opcodes"));
        assert!(output.contains("Brillig Opcodes"));
        assert!(output.contains("my_package"));
        assert!(output.contains("function_0"));
        assert!(output.contains("unconstrained_0"));
        assert!(output.contains("100")); // function_0 opcodes
        assert!(output.contains("50")); // unconstrained_0 opcodes
        assert!(output.contains("500")); // total unconstrained opcodes
        assert!(output.contains("N/A")); // ACIR opcodes for unconstrained functions
    }

    #[test]
    fn test_format_info_report_empty_programs() {
        let info_report = InfoReport { programs: vec![] };

        let json_output = format_info_report(&info_report, true);
        let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
        assert_eq!(parsed["programs"].as_array().unwrap().len(), 0);

        // output should be empty
        let table_output = format_info_report(&info_report, false);
        assert_eq!(table_output, "");
    }

    #[test]
    fn test_format_info_report_multiple_programs() {
        let info_report = InfoReport {
            programs: vec![
                create_test_program_info("package_a", 2, 1),
                create_test_program_info("package_b", 1, 2),
            ],
        };

        let output = format_info_report(&info_report, false);

        // Verify both programs appear in output
        assert!(output.contains("package_a"));
        assert!(output.contains("package_b"));
        assert!(output.contains("function_0"));
        assert!(output.contains("function_1"));
        assert!(output.contains("unconstrained_0"));
        assert!(output.contains("unconstrained_1"));
    }
}
