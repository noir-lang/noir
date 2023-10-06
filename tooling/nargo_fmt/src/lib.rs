#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(unused_qualifications, clippy::use_self)]

/// A Rust code formatting utility designed to manage and format untouched fragments of source code,
/// including comments, whitespace, and other characters. While the module doesn't directly address comments,
/// it treats them as unchanged fragments, ensuring their original placement and content remain preserved.
///
/// Key methods include:
/// - `format_missing`: Addresses characters between the last processed position and a given end position,
///   capturing comments and other untouched sequences.
/// - `format_missing_indent`: Functions similarly to `format_missing`, but introduces added indentation.
/// - `format_missing_inner`: The core method for handling missing fragments, appending them to the output buffer.
///   Pure whitespace fragments might be replaced or adjusted based on context.
/// - `push_vertical_spaces`: Standardizes vertical spacing, eliminating potential excessive empty lines
///   or ensuring adequate vertical separation.
///
/// By recognizing and properly handling these untouched fragments, the utility ensures comments remain intact
/// in both placement and content during the formatting process.
mod config;
pub mod errors;
mod visitor;

use noirc_frontend::ParsedModule;
use visitor::FmtVisitor;

pub use config::Config;

pub fn format(source: &str, parsed_module: ParsedModule, config: &Config) -> String {
    let mut fmt = FmtVisitor::new(source, config);
    fmt.visit_module(parsed_module);
    fmt.finish()
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, path::PathBuf};

    use crate::Config;

    #[test]
    fn test() {
        let files = std::fs::read_dir("tests/input").unwrap();
        for file in files {
            let file = file.unwrap();

            let config = Config::default();

            let source_path = file.path();
            let source = std::fs::read_to_string(&source_path).unwrap();

            let (parsed_module, errors) = noirc_frontend::parse_program(&source);
            let fmt_text = crate::format(&source, parsed_module, &config);

            assert!(errors.is_empty());

            let target_path: PathBuf = source_path
                .components()
                .map(|component| {
                    if component.as_os_str() == "input" {
                        OsStr::new("expected")
                    } else {
                        component.as_os_str()
                    }
                })
                .collect();

            let target = match std::fs::read_to_string(&target_path) {
                Ok(t) => t,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    std::fs::write(target_path, fmt_text.clone()).unwrap();
                    fmt_text.clone()
                }
                Err(err) => unreachable!("{err}"),
            };

            // FIXME: better diff
            assert_eq!(fmt_text, target);
        }
    }
}
