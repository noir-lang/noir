use std::collections::HashSet;

use fm::FileManager;
use noirc_errors::{
    CustomDiagnostic, Span, function_names::FunctionNames, reporter::ReportedErrors,
};

use crate::{
    ast::{Lambda, NoirFunction, NoirTrait, NoirTraitImpl, TypeImpl, Visitor},
    hir::{Context, ParsedFiles},
    parser::ParsedSubModule,
};

pub fn report_all(
    context: &Context,
    diagnostics: &[CustomDiagnostic],
    deny_warnings: bool,
    silence_warnings: bool,
) -> ReportedErrors {
    let function_names = function_names_for_diagnostics(diagnostics, &context.parsed_files);
    noirc_errors::reporter::report_all(
        context.file_manager.as_file_map(),
        &function_names,
        diagnostics,
        deny_warnings,
        silence_warnings,
    )
}

pub fn report_one(
    diagnostic: &CustomDiagnostic,
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    deny_warnings: bool,
    silence_warnings: bool,
) {
    let diagnostics = std::slice::from_ref(diagnostic);
    let function_names = function_names_for_diagnostics(diagnostics, parsed_files);
    noirc_errors::reporter::report_all(
        file_manager.as_file_map(),
        &function_names,
        diagnostics,
        deny_warnings,
        silence_warnings,
    );
}

/// Compute the set of function names needed to report the given diagnostics.
/// Only the names of functions that exists in files that appear in the call stacks
/// of the diagnostics are computed.
pub fn function_names_for_diagnostics(
    diagnostics: &[CustomDiagnostic],
    parsed_files: &ParsedFiles,
) -> FunctionNames {
    let mut file_ids = HashSet::new();
    for diagnostic in diagnostics {
        for location in &diagnostic.call_stack {
            file_ids.insert(location.file);
        }
    }

    let mut function_name = FunctionNames::new();
    let mut visitor = FunctionNamesCollector::new(&mut function_name);
    for file_id in file_ids {
        if let Some((parsed_file, _)) = parsed_files.get(&file_id) {
            parsed_file.accept(&mut visitor);
        }
    }

    function_name
}

/// Collects function names in a given ParsedModule.
/// Note: names are fully qualified respective to the file they are in. For example,
/// a function `foo` inside a module `bar` that is declared in a separate file will have
/// "foo" as its fully qualified name, and not "bar::foo".
struct FunctionNamesCollector<'a> {
    function_names: &'a mut FunctionNames,
    /// Path of the current item. Used to build fully qualified names.
    path: Vec<String>,
}

impl<'a> FunctionNamesCollector<'a> {
    fn new(function_names: &'a mut FunctionNames) -> Self {
        Self { function_names, path: Vec::new() }
    }

    fn fully_qualified_name(&self, name: &str) -> String {
        if self.path.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.path.join("::"), name)
        }
    }
}

impl Visitor for FunctionNamesCollector<'_> {
    fn visit_noir_function(&mut self, function: &NoirFunction, _span: Span) -> bool {
        let name = function.name().to_string();
        let full_name = self.fully_qualified_name(&name);
        self.function_names.insert(function.location(), full_name);

        self.path.push(name);
        function.accept_children(self);
        self.path.pop();

        false
    }

    fn visit_parsed_submodule(&mut self, parsed_submodule: &ParsedSubModule, _span: Span) -> bool {
        let name = parsed_submodule.name.to_string();

        self.path.push(name);
        parsed_submodule.accept_children(self);
        self.path.pop();

        false
    }

    fn visit_noir_trait(&mut self, trait_: &NoirTrait, _: Span) -> bool {
        let name = trait_.name.to_string();

        self.path.push(name);
        trait_.accept_children(self);
        self.path.pop();

        false
    }

    fn visit_type_impl(&mut self, type_impl: &TypeImpl, _: Span) -> bool {
        let name = type_impl.object_type.to_string();

        self.path.push(name);
        type_impl.accept_children(self);
        self.path.pop();

        false
    }

    fn visit_noir_trait_impl(&mut self, noir_trait_impl: &NoirTraitImpl, _: Span) -> bool {
        let name = format!("<{} as {}>", noir_trait_impl.object_type, noir_trait_impl.r#trait);

        self.path.push(name);
        noir_trait_impl.accept_children(self);
        self.path.pop();

        false
    }

    fn visit_lambda(&mut self, lambda: &Lambda, _span: Span) -> bool {
        let name = "{{closure}}".to_string();
        let full_name = self.fully_qualified_name(&name);
        self.function_names.insert(lambda.body.location, full_name);

        self.path.push(name);
        lambda.accept_children(self);
        self.path.pop();

        false
    }
}
