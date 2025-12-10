use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use noirc_errors::println_to_stdout;
use noirc_frontend::monomorphization::ast::Program;

use crate::errors::RuntimeError;

use super::ssa_gen::generate_ssa;
use super::{Ssa, SsaLogging};

type SsaPassResult = Result<Ssa, RuntimeError>;

/// An SSA pass reified as a construct we can put into a list,
/// which facilitates equivalence testing between different
/// stages of the processing pipeline.
pub struct SsaPass<'a> {
    msg: &'static str,
    run: Box<dyn Fn(Ssa) -> SsaPassResult + 'a>,
}

impl<'a> SsaPass<'a> {
    pub fn new<F>(f: F, msg: &'static str) -> Self
    where
        F: Fn(Ssa) -> Ssa + 'a,
    {
        Self::new_try(move |ssa| Ok(f(ssa)), msg)
    }

    pub fn new_try<F>(f: F, msg: &'static str) -> Self
    where
        F: Fn(Ssa) -> SsaPassResult + 'a,
    {
        Self { msg, run: Box::new(f) }
    }

    pub fn msg(&self) -> &str {
        self.msg
    }

    pub fn run(&self, ssa: Ssa) -> SsaPassResult {
        (self.run)(ssa)
    }

    /// Follow up the pass with another one, without adding a separate message for it.
    ///
    /// This is useful for attaching cleanup passes that we don't want to appear on their
    /// own in the pipeline, because it would just increase the noise.
    pub fn and_then<F>(self, f: F) -> Self
    where
        F: Fn(Ssa) -> Ssa + 'a,
    {
        self.and_then_try(move |ssa| Ok(f(ssa)))
    }

    /// Same as `and_then` but for passes that can fail.
    pub fn and_then_try<F>(self, f: F) -> Self
    where
        F: Fn(Ssa) -> SsaPassResult + 'a,
    {
        Self {
            msg: self.msg,
            run: Box::new(move |ssa| {
                let ssa = self.run(ssa)?;
                let ssa = f(ssa)?;
                Ok(ssa)
            }),
        }
    }
}

// This is just a convenience object to bundle the ssa with `print_ssa_passes` for debug printing.
pub struct SsaBuilder<'local> {
    /// The SSA being built; it is the input and the output of every pass ran by the builder.
    ssa: Ssa,
    /// Options to control which SSA passes to print.
    ssa_logging: SsaLogging,
    /// Whether to print the amount of time it took to run individual SSA passes.
    print_codegen_timings: bool,
    /// Counters indexed by the message in the SSA pass, so we can distinguish between multiple
    /// runs of the same pass in the printed messages.
    passed: HashMap<String, usize>,
    /// List of SSA pass message fragments that we want to skip, for testing purposes.
    skip_passes: Vec<String>,

    /// Providing a file manager is optional - if provided it can be used to print source
    /// locations along with each ssa instructions when debugging.
    files: Option<&'local fm::FileManager>,
}

impl<'local> SsaBuilder<'local> {
    pub fn from_program(
        program: Program,
        ssa_logging: SsaLogging,
        print_codegen_timings: bool,
        emit_ssa: &Option<PathBuf>,
        files: Option<&'local fm::FileManager>,
    ) -> Result<Self, RuntimeError> {
        let ssa = generate_ssa(program)?;
        if let Some(emit_ssa) = emit_ssa {
            let mut emit_ssa_dir = emit_ssa.clone();
            // We expect the full package artifact path to be passed in here,
            // and attempt to create the target directory if it does not exist.
            emit_ssa_dir.pop();
            create_named_dir(emit_ssa_dir.as_ref(), "target");
            let ssa_path = emit_ssa.with_extension("ssa.json");
            write_to_file(&serde_json::to_vec(&ssa).unwrap(), &ssa_path);
        }
        Ok(Self::from_ssa(ssa, ssa_logging, print_codegen_timings, files).print("Initial SSA"))
    }

    pub fn from_ssa(
        ssa: Ssa,
        ssa_logging: SsaLogging,
        print_codegen_timings: bool,
        files: Option<&'local fm::FileManager>,
    ) -> Self {
        Self {
            ssa_logging,
            print_codegen_timings,
            ssa,
            files,
            passed: Default::default(),
            skip_passes: Default::default(),
        }
    }

    pub fn ssa(&self) -> &Ssa {
        &self.ssa
    }

    pub fn with_passed(mut self, passed: HashMap<String, usize>) -> Self {
        self.passed = passed;
        self
    }

    pub fn with_skip_passes(mut self, skip_passes: Vec<String>) -> Self {
        self.skip_passes = skip_passes;
        self
    }

    pub fn finish(self) -> Ssa {
        self.ssa.generate_entry_point_index()
    }

    /// Run a list of SSA passes.
    pub fn run_passes(mut self, passes: &[SsaPass]) -> Result<Self, RuntimeError> {
        for pass in passes {
            self = self.try_run_pass(|ssa| pass.run(ssa), pass.msg)?;
        }
        Ok(self)
    }

    /// Runs the given SSA pass and prints the SSA afterward if `print_ssa_passes` is true.
    #[allow(dead_code)]
    fn run_pass<F>(mut self, pass: F, msg: &str) -> Self
    where
        F: FnOnce(Ssa) -> Ssa,
    {
        self.ssa = time(msg, self.print_codegen_timings, || pass(self.ssa));
        self.print(msg)
    }

    /// The same as `run_pass` but for passes that may fail
    fn try_run_pass<F>(mut self, pass: F, msg: &str) -> Result<Self, RuntimeError>
    where
        F: FnOnce(Ssa) -> Result<Ssa, RuntimeError>,
    {
        // Count the number of times we have seen this message.
        let cnt = *self.passed.entry(msg.to_string()).and_modify(|cnt| *cnt += 1).or_insert(1);
        let step = self.passed.values().sum::<usize>();
        let msg = format!("{msg} ({cnt}) (step {step})");

        // See if we should skip this pass, including the count, so we can skip the n-th occurrence of a step.
        let skip = self.skip_passes.iter().any(|s| msg.contains(s));

        if !skip {
            self.ssa = time(&msg, self.print_codegen_timings, || pass(self.ssa))?;
            Ok(self.print(&msg))
        } else {
            Ok(self)
        }
    }

    fn print(mut self, msg: &str) -> Self {
        let print_ssa_pass = self.ssa_logging.matches(msg);

        // Always normalize if we are going to print at least one of the passes
        if !matches!(self.ssa_logging, SsaLogging::None) {
            self.ssa.normalize_ids();
        }

        if print_ssa_pass {
            println_to_stdout!("After {msg}:\n{}", self.ssa.print_with(self.files));
        }
        self
    }
}

// Helper to time SSA passes
pub(super) fn time<T>(name: &str, print_timings: bool, f: impl FnOnce() -> T) -> T {
    let start_time = chrono::Utc::now().time();
    let result = f();

    if print_timings {
        let end_time = chrono::Utc::now().time();
        println_to_stdout!("{name}: {} ms", (end_time - start_time).num_milliseconds());
    }

    result
}

fn create_named_dir(named_dir: &Path, name: &str) -> PathBuf {
    std::fs::create_dir_all(named_dir)
        .unwrap_or_else(|_| panic!("could not create the `{name}` directory"));

    PathBuf::from(named_dir)
}

fn write_to_file(bytes: &[u8], path: &Path) {
    let display = path.display();

    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create {display}: {why}"),
        Ok(file) => file,
    };

    if let Err(why) = file.write_all(bytes) {
        panic!("couldn't write to {display}: {why}");
    }
}
