use crate::{
    ast::BlockExpression,
    ast::{FunctionKind, NoirFunction},
    hir::{
        comptime::{IResult, Interpreter, InterpreterState},
        resolution::resolver::Resolver,
        type_check::{TypeCheckState, TypeChecker},
    },
    hir_def::{expr::HirBlockExpression, function::HirFunction},
    node_interner::{FuncId, NodeInterner, StmtId},
    Type,
};

/// The job of the Composer is to interleave name resolution, type checking, and
/// comptime interpretation together into "one" pass. It does this by running each
/// pass for each statement before moving on to the next statement instead of finishing
/// the entire program one pass at a time.
///
/// This is used so that the comptime interpreter can expand macros into fresh code which
/// can then immediately be name resolved and type checked afterward.
#[derive(Default)]
pub struct Composer {
    interner: NodeInterner,
    type_checker_state: TypeCheckState,
    interpreter_state: InterpreterState,
}

/// This trait isn't technically required but it is a good way to understand the API
/// that the Composer requires from each pass.
///
/// To avoid making the entire passes async-safe, this trait sets several suspend points
/// at which the current state is saved and we switch to another pass. These points are:
/// - Blocks
/// - Statements
/// - Functions
pub trait Composable {
    type Result;
    type FunctionState;
    type BlockInput<'c>;

    fn enter_function(&mut self, function: FuncId) -> Self::FunctionState;
    fn exit_function(&mut self, body: Self::Result, state: Self::FunctionState) -> Self::Result;

    /// The name resolver and interpreter both need this event to push a new scope.
    fn enter_block(&mut self);
    fn exit_block<'local>(&mut self, input: Self::BlockInput<'local>) -> Self::Result;

    /// No pass currently needs to pause in the middle of a statement and save state
    /// so we don't need an enter/exit pair for this case
    fn do_statement(&mut self, statement: StmtId) -> Self::Result;
}

impl Composer {
    fn interpreter(&mut self) -> Interpreter {
        Interpreter::with_state(&mut self.interner, self.interpreter_state)
    }

    fn type_checker(&mut self) -> TypeChecker {
        TypeChecker::with_state(&mut self.interner, self.type_checker_state)
    }

    pub fn compose_function(
        &mut self,
        resolver: &mut Resolver,
        function: NoirFunction,
        id: FuncId,
    ) {
        let resolver_state = resolver.enter_function(function, id);
        let checker_state = self.type_checker().enter_function(id);
        let interpreter_state = self.interpreter().enter_function(id);

        let (hir_func, body_type, body_result) = match function.kind {
            FunctionKind::Builtin | FunctionKind::LowLevel | FunctionKind::Oracle => {
                (HirFunction::empty(), Type::Unit, Ok(()))
            }
            FunctionKind::Normal | FunctionKind::Recursive => {
                resolver.intern_function_body(function, id)
            }
        };

        resolver.exit_function(resolver_state, id, hir_func);
        self.type_checker().exit_function(body_type, checker_state);
        self.interpreter().exit_function(body_result, interpreter_state);
    }

    pub fn compose_block(
        &mut self,
        resolver: &mut Resolver,
        block: BlockExpression,
    ) -> (HirBlockExpression, Type, IResult<()>) {
        resolver.enter_block();
        self.type_checker().enter_block();
        self.interpreter().enter_block();

        let mut statements = Vec::with_capacity(block.statements.len());
        let mut types = Vec::with_capacity(block.statements.len());
        let mut results = Vec::with_capacity(block.statements.len());

        for statement in block.statements {
            let (id, typ, result) = resolver.compose_stmt(statement);
            statements.push(id);
            types.push(typ);
            results.push(result);
        }

        let block_id = resolver.exit_block(&statements);
        let block_type = self.type_checker().exit_block((&statements, types));
        let block_result = self.interpreter().exit_block(results);
        (block_id, block_type, block_result)
    }

    pub fn compose_statement(&mut self, stmt_id: StmtId) -> (Type, IResult<()>) {
        let typ = self.type_checker().do_statement(stmt_id);
        let result = self.interpreter().do_statement(stmt_id);
        (typ, result)
    }
}
