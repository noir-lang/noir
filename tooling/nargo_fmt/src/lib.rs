#![deny(unused_qualifications, clippy::use_self)]

mod config;
mod visitor;

use visitor::FmtVisitor;

pub fn format(source: &str) -> Result<String, Vec<noirc_errors::CustomDiagnostic>> {
    let (module, errors) = noirc_frontend::parse_program(source);

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut fmt = FmtVisitor::new(source);
    fmt.visit_module(module);

    Ok(fmt.finish())
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, path::PathBuf};

    #[test]
    fn test() {
        let files = std::fs::read_dir("tests/input").unwrap();
        for file in files {
            let file = file.unwrap();

            let source_path = file.path();
            let source = std::fs::read_to_string(&source_path).unwrap();
            let fmt_text = crate::format(&source).unwrap();

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

            // TODO: better diff
            assert_eq!(fmt_text, target);
        }
    }
}
