use crate::{
    ast::{BlockExpression, Statement},
    hir::{
        comptime::{IResult, Interpreter, InterpreterState},
        resolution::resolver::Resolver,
        type_check::{TypeCheckState, TypeChecker},
    },
    macros_api::NodeInterner,
    node_interner::StmtId,
    Type,
};

/// The job of the Composer is to interleave name resolution, type checking, and
/// comptime interpretation together into "one" pass. It does this by running each
/// pass for each statement before moving on to the next statement instead of finishing
/// the entire program one pass at a time.
///
/// This is used so that the comptime interpreter can expand macros into fresh code which
/// can then immediately be name resolved and type checked afterward.
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
    type FunctionResult;
    type BlockResult;
    type StatementResult;

    type FunctionState;

    type FunctionInput;
    type BlockInput<'c>;
    type StatementInput;

    fn enter_function(&mut self, function: Self::FunctionInput) -> Self::FunctionState;
    fn exit_function(
        &mut self,
        body: Self::BlockResult,
        state: Self::FunctionState,
    ) -> Self::FunctionResult;

    /// The name resolver and interpreter both need this event to push a new scope.
    fn enter_block(&mut self);
    fn exit_block<'local>(&mut self, input: Self::BlockInput<'local>) -> Self::BlockResult;

    /// No pass currently needs to pause in the middle of a statement and save state
    /// so we don't need an enter/exit pair for this case
    fn do_statement(&mut self, statement: Self::StatementInput) -> Self::StatementResult;
}

impl Composer {
    fn interpreter(&mut self) -> Interpreter {
        Interpreter::with_state(&mut self.interner, self.interpreter_state)
    }

    fn type_checker(&mut self) -> TypeChecker {
        TypeChecker::with_state(&mut self.interner, self.type_checker_state)
    }

    pub fn compose_block(&mut self, resolver: &mut Resolver, block: BlockExpression) -> Vec<StmtId> {
        resolver.enter_block();
        self.type_checker().enter_block();
        self.interpreter().enter_block();

        let mut statements = Vec::with_capacity(block.statements.len());
        let mut types = Vec::with_capacity(block.statements.len());
        let mut results = Vec::with_capacity(block.statements.len());

        for statement in block.statements {
            let (id, typ, result) = self.compose_statement(statement);
            statements.push(id);
            types.push(typ);
            results.push(result);
        }

        resolver.exit_block(&statements);
        self.type_checker().exit_block((&statements, types));
        self.interpreter().exit_block(results);
        statements
    }

    pub fn compose_statement(&mut self, stmt_id: StmtId) -> (Type, IResult<()>) {
        let typ = self.type_checker().do_statement(stmt_id);
        let result = self.interpreter().do_statement(stmt_id);
        (typ, result)
    }
}
