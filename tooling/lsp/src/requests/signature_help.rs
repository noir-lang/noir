use std::future::{self, Future};

use async_lsp::ResponseError;
use fm::{FileId, PathString};
use lsp_types::{
    ParameterInformation, ParameterLabel, SignatureHelp, SignatureHelpParams, SignatureInformation,
};
use noirc_errors::{Location, Span};
use noirc_frontend::{
    ast::{CallExpression, Expression, FunctionReturnType, MethodCallExpression},
    hir_def::{function::FuncMeta, stmt::HirPattern},
    macros_api::NodeInterner,
    node_interner::ReferenceId,
    ParsedModule, Type,
};

use crate::{utils, LspState};

use super::process_request;

mod tests;
mod traversal;

pub(crate) fn on_signature_help_request(
    state: &mut LspState,
    params: SignatureHelpParams,
) -> impl Future<Output = Result<Option<SignatureHelp>, ResponseError>> {
    let uri = params.text_document_position_params.clone().text_document.uri;

    let result = process_request(state, params.text_document_position_params.clone(), |args| {
        let path = PathString::from_path(uri.to_file_path().unwrap());
        args.files.get_file_id(&path).and_then(|file_id| {
            utils::position_to_byte_index(
                args.files,
                file_id,
                &params.text_document_position_params.position,
            )
            .and_then(|byte_index| {
                let file = args.files.get_file(file_id).unwrap();
                let source = file.source();
                let (parsed_module, _errors) = noirc_frontend::parse_program(source);

                let mut finder = SignatureFinder::new(file_id, byte_index, args.interner);
                finder.find(&parsed_module)
            })
        })
    });
    future::ready(result)
}

struct SignatureFinder<'a> {
    file: FileId,
    byte_index: usize,
    interner: &'a NodeInterner,
    signature_help: Option<SignatureHelp>,
}

impl<'a> SignatureFinder<'a> {
    fn new(file: FileId, byte_index: usize, interner: &'a NodeInterner) -> Self {
        Self { file, byte_index, interner, signature_help: None }
    }

    fn find(&mut self, parsed_module: &ParsedModule) -> Option<SignatureHelp> {
        self.find_in_parsed_module(parsed_module);

        self.signature_help.clone()
    }

    fn find_in_call_expression(&mut self, call_expression: &CallExpression, span: Span) {
        self.find_in_expression(&call_expression.func);
        self.find_in_expressions(&call_expression.arguments);

        let arguments_span = Span::from(call_expression.func.span.end() + 1..span.end() - 1);
        let span = call_expression.func.span;
        let name_span = Span::from(span.end() - 1..span.end());
        let has_self = false;

        self.try_compute_signature_help(
            &call_expression.arguments,
            arguments_span,
            name_span,
            has_self,
        );
    }

    fn find_in_method_call_expression(
        &mut self,
        method_call_expression: &MethodCallExpression,
        span: Span,
    ) {
        self.find_in_expression(&method_call_expression.object);
        self.find_in_expressions(&method_call_expression.arguments);

        let arguments_span =
            Span::from(method_call_expression.method_name.span().end() + 1..span.end() - 1);
        let name_span = method_call_expression.method_name.span();
        let has_self = true;

        self.try_compute_signature_help(
            &method_call_expression.arguments,
            arguments_span,
            name_span,
            has_self,
        );
    }

    fn try_compute_signature_help(
        &mut self,
        arguments: &[Expression],
        arguments_span: Span,
        name_span: Span,
        has_self: bool,
    ) {
        if self.signature_help.is_some() {
            return;
        }

        if !self.includes_span(arguments_span) {
            return;
        }

        let mut active_parameter = None;
        for (index, arg) in arguments.iter().enumerate() {
            if self.includes_span(arg.span) || arg.span.start() as usize >= self.byte_index {
                active_parameter = Some(index as u32);
                break;
            }
        }

        if active_parameter.is_none() {
            active_parameter = Some(arguments.len() as u32);
        }

        let location = Location::new(name_span, self.file);

        // Check if the call references a named function
        if let Some(ReferenceId::Function(func_id)) = self.interner.find_referenced(location) {
            let name = self.interner.function_name(&func_id);
            let func_meta = self.interner.function_meta(&func_id);

            let signature_information =
                self.func_meta_signature_information(func_meta, name, active_parameter, has_self);
            self.set_signature_help(signature_information);
            return;
        }

        // Otherwise, the call must be a reference to an fn type
        if let Some(mut typ) = self.interner.type_at_location(location) {
            typ = typ.follow_bindings();
            if let Type::Forall(_, forall_typ) = typ {
                typ = *forall_typ;
            }
            if let Type::Function(args, return_type, _, unconstrained) = typ {
                let signature_information = self.function_type_signature_information(
                    &args,
                    &return_type,
                    unconstrained,
                    active_parameter,
                );
                self.set_signature_help(signature_information);
            }
        }
    }

    fn func_meta_signature_information(
        &self,
        func_meta: &FuncMeta,
        name: &str,
        active_parameter: Option<u32>,
        has_self: bool,
    ) -> SignatureInformation {
        let mut label = String::new();
        let mut parameters = Vec::new();

        label.push_str("fn ");
        label.push_str(name);
        label.push('(');
        for (index, (pattern, typ, _)) in func_meta.parameters.0.iter().enumerate() {
            if index > 0 {
                label.push_str(", ");
            }

            if has_self && index == 0 {
                if let Type::MutableReference(..) = typ {
                    label.push_str("&mut self");
                } else {
                    label.push_str("self");
                }
            } else {
                let parameter_start = label.chars().count();

                self.hir_pattern_to_argument(pattern, &mut label);
                label.push_str(": ");
                label.push_str(&typ.to_string());

                let parameter_end = label.chars().count();

                parameters.push(ParameterInformation {
                    label: ParameterLabel::LabelOffsets([
                        parameter_start as u32,
                        parameter_end as u32,
                    ]),
                    documentation: None,
                });
            }
        }
        label.push(')');

        match &func_meta.return_type {
            FunctionReturnType::Default(_) => (),
            FunctionReturnType::Ty(typ) => {
                label.push_str(" -> ");
                label.push_str(&typ.to_string());
            }
        }

        SignatureInformation {
            label,
            documentation: None,
            parameters: Some(parameters),
            active_parameter,
        }
    }

    fn function_type_signature_information(
        &self,
        args: &[Type],
        return_type: &Type,
        unconstrained: bool,
        active_parameter: Option<u32>,
    ) -> SignatureInformation {
        let mut label = String::new();
        let mut parameters = Vec::new();

        if unconstrained {
            label.push_str("unconstrained ");
        }
        label.push_str("fn(");
        for (index, typ) in args.iter().enumerate() {
            if index > 0 {
                label.push_str(", ");
            }

            let parameter_start = label.chars().count();
            label.push_str(&typ.to_string());
            let parameter_end = label.chars().count();

            parameters.push(ParameterInformation {
                label: ParameterLabel::LabelOffsets([parameter_start as u32, parameter_end as u32]),
                documentation: None,
            });
        }
        label.push(')');

        if let Type::Unit = return_type {
            // Nothing
        } else {
            label.push_str(" -> ");
            label.push_str(&return_type.to_string());
        }

        SignatureInformation {
            label,
            documentation: None,
            parameters: Some(parameters),
            active_parameter,
        }
    }

    fn hir_pattern_to_argument(&self, pattern: &HirPattern, text: &mut String) {
        match pattern {
            HirPattern::Identifier(hir_ident) => {
                text.push_str(self.interner.definition_name(hir_ident.id));
            }
            HirPattern::Mutable(pattern, _) => self.hir_pattern_to_argument(pattern, text),
            HirPattern::Tuple(_, _) | HirPattern::Struct(_, _, _) => text.push('_'),
        }
    }

    fn set_signature_help(&mut self, signature_information: SignatureInformation) {
        let signature_help = SignatureHelp {
            active_parameter: signature_information.active_parameter,
            signatures: vec![signature_information],
            active_signature: Some(0),
        };
        self.signature_help = Some(signature_help);
    }

    fn includes_span(&self, span: Span) -> bool {
        span.start() as usize <= self.byte_index && self.byte_index <= span.end() as usize
    }
}
