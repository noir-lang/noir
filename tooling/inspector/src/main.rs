use std::{path::PathBuf, process::exit};

use acir::circuit::ExpressionWidth;
use clap::Parser;
use noirc_artifacts::program::ProgramArtifact;
use noirc_artifacts_info::{count_opcodes_and_gates_in_program, show_info_report, InfoReport};

#[derive(Debug, Clone, Parser)]
pub(crate) struct Args {
    /// The artifact to inspect
    artifact: PathBuf,

    /// Output a JSON formatted report. Changes to this format are not currently considered breaking.
    #[clap(long, hide = true)]
    json: bool,

    /// Specify the backend expression width that should be assumed
    #[arg(long, value_parser = parse_expression_width)]
    expression_width: Option<ExpressionWidth>,

    /// Display the ACIR for compiled circuit
    #[arg(long)]
    print_acir: bool,
}

pub const DEFAULT_EXPRESSION_WIDTH: ExpressionWidth = ExpressionWidth::Bounded { width: 4 };

fn main() {
    let args = Args::parse();

    let file = match std::fs::File::open(args.artifact.clone()) {
        Ok(file) => file,
        Err(err) => {
            println!("Cannot open file `{}`: {}", args.artifact.to_string_lossy(), err,);
            exit(1);
        }
    };
    let artifact: ProgramArtifact = match serde_json::from_reader(file) {
        Ok(artifact) => artifact,
        Err(error) => {
            println!("Cannot deserialize artifact: {}", error);
            exit(1);
        }
    };

    if args.print_acir {
        println!("Compiled ACIR for main:");
        println!("{}", artifact.bytecode);
    }

    let package_name = args
        .artifact
        .with_extension("")
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "artifact".to_string());

    let expression_width = args.expression_width.unwrap_or(DEFAULT_EXPRESSION_WIDTH);
    let program_info =
        count_opcodes_and_gates_in_program(artifact, package_name.to_string(), expression_width);

    let info_report = InfoReport { programs: vec![program_info] };
    show_info_report(info_report, args.json);
}

fn parse_expression_width(input: &str) -> Result<ExpressionWidth, std::io::Error> {
    use std::io::{Error, ErrorKind};

    let width = input
        .parse::<usize>()
        .map_err(|err| Error::new(ErrorKind::InvalidInput, err.to_string()))?;

    match width {
        0 => Ok(ExpressionWidth::Unbounded),
        _ => Ok(ExpressionWidth::Bounded { width }),
    }
}
