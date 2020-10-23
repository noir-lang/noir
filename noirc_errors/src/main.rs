use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term;

// `files::SimpleFile` and `files::SimpleFiles` help you get up and running with
// `codespan-reporting` quickly! More complicated use cases can be supported
// by creating custom implementations of the `files::Files` trait.

fn main() {
    let mut files = SimpleFiles::new();

let file_id = files.add(
    "main.noir",
    
        r#"var x = 5"#,
);

let diagnostic = Diagnostic::error()
    .with_message("unknown word use to start declaration")
    .with_code("E0308")
    .with_labels(vec![
        Label::primary(file_id, 0..3).with_message("Cannot use `var` in a declaration statement"),
    ])
    .with_notes(vec![
            unindent::unindent("
            expected priv, const or let
                found var
        ")
        ]);

// We now set up the writer and configuration, and then finally render the
// diagnostic to standard error.

let writer = StandardStream::stderr(ColorChoice::Always);
let config = codespan_reporting::term::Config::default();

term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
}