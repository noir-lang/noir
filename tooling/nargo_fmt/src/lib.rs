#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(unused_qualifications, clippy::use_self)]

mod config;
mod visitor;

use noirc_frontend::ParsedModule;
use visitor::FmtVisitor;

pub fn format(source: &str, parsed_module: ParsedModule) -> String {
    let mut fmt = FmtVisitor::new(source);
    fmt.visit_module(parsed_module);
    fmt.finish()
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

            let (parsed_module, errors) = noirc_frontend::parse_program(&source);
            let fmt_text = crate::format(&source, parsed_module);

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
