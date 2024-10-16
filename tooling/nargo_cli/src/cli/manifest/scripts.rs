use anyhow::Result;
use deno_task_shell::parser::SequentialList;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fmt;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ScriptDefinition(String);

impl ScriptDefinition {
    pub fn new(script: String) -> Self {
        Self(script)
    }

    pub fn parse(&self, args: &[OsString]) -> Result<SequentialList> {
        // The following implementation has been copied from the `deno_task_shell` crate
        // with slight modifications only.
        // The original implementation can be found here:
        // https://github.com/denoland/deno/blob/c34e26a9d56596645ee63b19f99c09cf4aea4b37/cli/tools/task.rs#L111-L123

        let manifest_script = &self.0;
        let additional_args = args
            .iter()
            // surround all the additional arguments in double quotes
            // and sanitize any command substitution
            .map(|a| {
                format!(
                    "\"{}\"",
                    a.to_string_lossy()
                        .to_string()
                        .replace('"', "\\\"")
                        .replace('$', "\\$")
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        let full_script = format!("{manifest_script} {additional_args}");
        let script = full_script.trim().to_owned();
        deno_task_shell::parser::parse(&script)
    }
}

impl fmt::Display for ScriptDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
