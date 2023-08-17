// based on this code: https://github.com/zkat/miette/blob/main/src/panic.rs

use backtrace::Backtrace;
use miette::{Diagnostic, WrapErr};
use thiserror::Error;

const PANIC_MESSAGE: &str = "This is a bug. We may have already fixed this in newer versions of Nargo so try searching for similar issues at https://github.com/noir-lang/noir/issues/.\nIf there isn't an open issue for this bug, consider opening one at https://github.com/noir-lang/noir/issues/new?labels=bug&template=bug_report.yml";

pub fn set_hook() {
    std::panic::set_hook(Box::new(move |info| {
        let mut report: miette::Result<()> = Err(Panic.into());
        if let Some(loc) = info.location() {
            report = report
                .with_context(|| format!("at {}:{}:{}", loc.file(), loc.line(), loc.column()));
        }
        if let Err(err) = report.with_context(|| PANIC_MESSAGE.to_string()) {
            eprintln!("{err:?}");
        }
    }));
}

#[derive(Debug, Error, Diagnostic)]
#[error("{0}{}", Panic::backtrace())]
#[diagnostic()]
struct Panic;

impl Panic {
    fn backtrace() -> String {
        use std::fmt::Write;
        if let Ok(var) = std::env::var("RUST_BACKTRACE") {
            if !var.is_empty() && var != "0" {
                const HEX_WIDTH: usize = std::mem::size_of::<usize>() + 2;
                // Padding for next lines after frame's address
                const NEXT_SYMBOL_PADDING: usize = HEX_WIDTH + 6;
                let mut backtrace = String::new();
                let trace = Backtrace::new();
                let frames = backtrace_ext::short_frames_strict(&trace).enumerate();
                for (idx, (frame, sub_frames)) in frames {
                    let ip = frame.ip();
                    let _ = write!(backtrace, "\n{:4}: {:2$?}", idx, ip, HEX_WIDTH);

                    let symbols = frame.symbols();
                    if symbols.is_empty() {
                        let _ = write!(backtrace, " - <unresolved>");
                        continue;
                    }

                    for (idx, symbol) in symbols[sub_frames].iter().enumerate() {
                        // Print symbols from this address,
                        // if there are several addresses
                        // we need to put it on next line
                        if idx != 0 {
                            let _ = write!(backtrace, "\n{:1$}", "", NEXT_SYMBOL_PADDING);
                        }

                        if let Some(name) = symbol.name() {
                            let _ = write!(backtrace, " - {}", name);
                        } else {
                            let _ = write!(backtrace, " - <unknown>");
                        }

                        // See if there is debug information with file name and line
                        if let (Some(file), Some(line)) = (symbol.filename(), symbol.lineno()) {
                            let _ = write!(
                                backtrace,
                                "\n{:3$}at {}:{}",
                                "",
                                file.display(),
                                line,
                                NEXT_SYMBOL_PADDING
                            );
                        }
                    }
                }
                return backtrace;
            }
        }
        "".into()
    }
}
