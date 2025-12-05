use iter_extended::vecmap;
use noirc_artifacts::program::ProgramArtifact;
use prettytable::{Row, row, table};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct InfoReport {
    pub programs: Vec<ProgramInfo>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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

pub fn show_info_report(info_report: InfoReport, json: bool) {
    if json {
        // Expose machine-readable JSON data.
        println!("{}", serde_json::to_string(&info_report).unwrap());
    } else {
        // Otherwise print human-readable table.
        if !info_report.programs.is_empty() {
            let mut program_table = table!([Fm->"Package", Fm->"Function", Fm->"Expression Width", Fm->"ACIR Opcodes", Fm->"Brillig Opcodes"]);

            for program_info in info_report.programs {
                let program_rows: Vec<Row> = program_info.into();
                for row in program_rows {
                    program_table.add_row(row);
                }
            }
            program_table.printstd();
        }
    }
}
