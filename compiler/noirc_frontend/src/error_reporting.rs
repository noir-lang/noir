use std::collections::HashSet;

use fm::{FileId, FileManager};
use noirc_errors::{
    CustomDiagnostic, Location, Span, function_names::FunctionNames, reporter::ReportedErrors,
};

use crate::{
    ParsedModule,
    ast::{LetStatement, NoirFunction, NoirTrait, NoirTraitImpl, TypeImpl, Visitor},
    hir::ParsedFiles,
    parser::ParsedSubModule,
};

pub fn report_all(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    diagnostics: &[CustomDiagnostic],
    deny_warnings: bool,
    silence_warnings: bool,
) -> ReportedErrors {
    let function_names = function_names_for_diagnostics(diagnostics, parsed_files);
    noirc_errors::reporter::report_all(
        file_manager.as_file_map(),
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
    let include_comptime_items = true;
    for file_id in file_ids {
        if let Some((parsed_file, _)) = parsed_files.get(&file_id) {
            let mut visitor =
                FunctionNamesCollector::new(&mut function_name, file_id, include_comptime_items);
            parsed_file.accept(&mut visitor);
        }
    }

    function_name
}

/// Computes the set of function names that appear in the given parsed module.
pub fn function_names_in_parsed_module(
    parsed_module: &ParsedModule,
    file_id: FileId,
    include_comptime_items: bool,
) -> FunctionNames {
    let mut function_names = FunctionNames::new();
    let mut visitor =
        FunctionNamesCollector::new(&mut function_names, file_id, include_comptime_items);
    parsed_module.accept(&mut visitor);
    function_names
}

/// Collects function names in a given ParsedModule.
/// Note: names are fully qualified respective to the file they are in. For example,
/// a function `foo` inside a module `bar` that is declared in a separate file will have
/// "foo" as its fully qualified name, and not "bar::foo".
struct FunctionNamesCollector<'a> {
    function_names: &'a mut FunctionNames,
    file_id: FileId,
    /// Whether to include comptime items such as a global's contents or meta attributes.
    /// Tracking these names is only relevant during compilation when an error could
    /// happen during comptime evaluation. However, these locations are not relevant
    /// after a compilation artifact has been produced since evaluation of those items
    /// has already succeeded.
    include_comptime_items: bool,
    /// Path of the current item. Used to build fully qualified names.
    path: Vec<String>,
}

impl<'a> FunctionNamesCollector<'a> {
    fn new(
        function_names: &'a mut FunctionNames,
        file_id: FileId,
        include_comptime_items: bool,
    ) -> Self {
        Self { function_names, file_id, include_comptime_items, path: Vec::new() }
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
        let name =
            format!("<impl {} for {}>", noir_trait_impl.r#trait, noir_trait_impl.object_type);

        self.path.push(name);
        noir_trait_impl.accept_children(self);
        self.path.pop();

        false
    }

    fn visit_meta_attribute(
        &mut self,
        attribute: &crate::token::MetaAttribute,
        _target: crate::ast::AttributeTarget,
        span: Span,
    ) -> bool {
        if !self.include_comptime_items {
            return false;
        }

        let name = format!("#[{}]", attribute.name);
        let full_name = self.fully_qualified_name(&name);
        let location = Location::new(span, self.file_id);
        self.function_names.insert(location, full_name);

        self.path.push(name);
        attribute.accept_children(self);
        self.path.pop();

        true
    }

    fn visit_global(&mut self, let_statement: &LetStatement, span: Span) -> bool {
        if !self.include_comptime_items {
            return false;
        }

        let name = format!("<global {}>", let_statement.pattern);
        let full_name = self.fully_qualified_name(&name);
        let location = Location::new(span, self.file_id);
        self.function_names.insert(location, full_name);

        self.path.push(name);
        let_statement.accept_children(self);
        self.path.pop();

        false
    }
}
