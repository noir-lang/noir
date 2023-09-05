use std::collections::HashSet;

use noirc_errors::{CustomDiagnostic, FileDiagnostic, Location};

use crate::hir::Context;
use crate::{
    hir_def::{expr::HirExpression, stmt::HirStatement},
    node_interner::{DefinitionKind, ExprId, FuncId, NodeInterner},
};

use super::def_map::{ModuleData, ModuleId};

#[derive(Default)]
pub struct FunctionVisibility {
    called_functions: HashSet<(FuncId, Location)>,
    processed_functions: HashSet<FuncId>,
    pub errors: Vec<FileDiagnostic>,
}

impl FunctionVisibility {
    fn lookup_function_calls(&mut self, interner: &NodeInterner, expr_id: ExprId) {
        match interner.expression(&expr_id) {
            HirExpression::Ident(ident) => {
                if let Some(definition_info) = interner.try_definition(ident.id) {
                    if let DefinitionKind::Function(id) = definition_info.kind {
                        self.called_functions.insert((id, ident.location));
                    }
                }
            }
            HirExpression::Literal(_) => (),
            HirExpression::Block(block) => {
                for stmt in block.0 {
                    if let HirStatement::Expression(expr_id) = interner.statement(&stmt) {
                        self.lookup_function_calls(interner, expr_id);
                    }
                }
            }
            HirExpression::Prefix(prefix) => {
                self.lookup_function_calls(interner, prefix.rhs);
            }
            HirExpression::Infix(infix) => {
                self.lookup_function_calls(interner, infix.lhs);
                self.lookup_function_calls(interner, infix.rhs);
            }
            HirExpression::Index(index) => {
                self.lookup_function_calls(interner, index.index);
                self.lookup_function_calls(interner, index.collection);
            }

            HirExpression::MemberAccess(access) => {
                self.lookup_function_calls(interner, access.lhs);
            }

            HirExpression::Call(call_expr) => {
                self.lookup_function_calls(interner, call_expr.func);
                for argument in call_expr.arguments {
                    self.lookup_function_calls(interner, argument);
                }
            }
            HirExpression::Cast(cast) => self.lookup_function_calls(interner, cast.lhs),
            HirExpression::For(for_expr) => {
                self.lookup_function_calls(interner, for_expr.start_range);
                self.lookup_function_calls(interner, for_expr.end_range);
                self.lookup_function_calls(interner, for_expr.block);
            }
            HirExpression::If(if_expr) => {
                self.lookup_function_calls(interner, if_expr.condition);
                self.lookup_function_calls(interner, if_expr.consequence);
                if let Some(alternative) = if_expr.alternative {
                    self.lookup_function_calls(interner, alternative);
                }
            }
            HirExpression::Tuple(fields) => {
                for field in fields {
                    self.lookup_function_calls(interner, field);
                }
            }
            HirExpression::Constructor(constructor) => {
                for field in constructor.fields {
                    self.lookup_function_calls(interner, field.1);
                }
            }
            HirExpression::Lambda(lambda) => self.lookup_function_calls(interner, lambda.body),
            HirExpression::MethodCall(_) => (),
            HirExpression::Error => (),
        }
    }

    /// Checks that the input function is only calling functions with 'pub' modifier outside its module
    /// recursively checks the calling functions, if they are not already checked.
    pub fn check_visibility(&mut self, context: &Context, func_id: &FuncId) {
        if !self.processed_functions.contains(func_id) {
            let interner = &context.def_interner;
            let meta = interner.function_meta(func_id);

            //Retrieve the called functions
            self.called_functions.clear();
            let body = interner.function(func_id).block(interner);
            for stmt in body.statements() {
                match interner.statement(stmt) {
                    HirStatement::Let(let_stmt) => {
                        self.lookup_function_calls(interner, let_stmt.expression);
                    }
                    HirStatement::Constrain(constrain_stmt) => {
                        self.lookup_function_calls(interner, constrain_stmt.0);
                    }
                    HirStatement::Assign(assign_stmt) => {
                        self.lookup_function_calls(interner, assign_stmt.expression);
                    }
                    HirStatement::Expression(expr_id) | HirStatement::Semi(expr_id) => {
                        self.lookup_function_calls(interner, expr_id);
                    }
                    HirStatement::Error => (),
                }
            }
            //check callee visibility
            for id in &self.called_functions {
                let callee_meta = interner.function_meta(&id.0);
                if callee_meta.module_id != meta.module_id && !callee_meta.is_public {
                    //we allow calling private function if if is inside a sub-module
                    if meta.module_id.krate == callee_meta.module_id.krate {
                        let crate_id = meta.module_id.krate;
                        if let Some(crate_map) = context.def_map(&crate_id) {
                            let module_data = &crate_map.modules()[meta.module_id.local_id.0];
                            if lookup_module_descendant(
                                module_data,
                                callee_meta.module_id,
                                crate_map.modules(),
                            ) {
                                continue;
                            }
                        }
                    }
                    let primary_message =
                    format!("function `{}` is private. Calling private functions will be deprecated in future versions", interner.function_name(&id.0));
                    let secondary_message = "private function".to_string();
                    let diagnostic = CustomDiagnostic::simple_warning(
                        primary_message,
                        secondary_message,
                        id.1.span,
                    );
                    let error = FileDiagnostic::new(meta.location.file, diagnostic);
                    self.errors.push(error);
                }
            }
            self.processed_functions.insert(*func_id);
            for id in &self.called_functions.clone() {
                self.check_visibility(context, &id.0);
            }
        }
    }
}

// Returns true if module_id is a descendant of module
fn lookup_module_descendant(
    module: &ModuleData,
    module_id: ModuleId,
    modules: &arena::Arena<ModuleData>,
) -> bool {
    for child in module.children.values() {
        if *child == module_id.local_id {
            return true;
        }
        lookup_module_descendant(&modules[child.0], module_id, modules);
    }
    false
}
