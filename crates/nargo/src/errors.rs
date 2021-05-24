use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug)]
pub enum CliError {
    Generic(String),
    DestinationAlreadyExists(String),
}

impl CliError {
    pub(crate) fn write(&self) -> ! {
        match self {
            CliError::Generic(msg) => CliError::write_msg_exit(&msg),
            CliError::DestinationAlreadyExists(msg) => CliError::write_msg_exit(&msg),
        }
    }

    fn write_msg_exit(msg: &str) -> ! {
        let mut stderr = StandardStream::stderr(ColorChoice::Always);
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
            .expect("cannot set color for stderr in StandardStream");
        writeln!(&mut stderr, "{}", msg).expect("cannot write to stderr");

        std::process::exit(0)
    }
}
