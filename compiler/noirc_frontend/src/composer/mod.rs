//! The Composer's job is to recur on the Ast, calling name resolution,
//! type checking, and the comptime interpreter in lock-step one node at a time.
//! It accomplishes this with the `functor::Ast` which does not have sub-nodes
//! that these passes can recur upon. Instead, it has generic slots for result values.
//!
//! The Composer's job then is to just recur on the raw Ast, create a `functor::Ast`
//! for that node holding results from any recursive calls, and hand that off to each pass.
pub mod functor;

use iter_extended::vecmap;

use crate::{
    ast::{BlockExpression, ArrayLiteral},
    ast::{FunctionKind, NoirFunction},
    hir::{
        comptime::{IResult, Interpreter, InterpreterState},
        resolution::resolver::Resolver,
        type_check::{TypeCheckState, TypeChecker},
    },
    hir_def::{expr::HirBlockExpression, function::HirFunction},
    node_interner::{FuncId, NodeInterner, StmtId, ExprId},
    Type, macros_api::{Expression, ExpressionKind, Literal},
};

macro_rules! apply3 {
    ($f: expr, $a:expr, $b:expr, $c:expr) => {{
        ($f($a), $f($b), $f($c))
    }}
}

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

    pub fn compose_expression(&mut self, expression: Expression) -> Results {
        match expression.kind {
            ExpressionKind::Literal(literal) => self.compose_literal(literal),
            ExpressionKind::Block(_) => todo!(),
            ExpressionKind::Prefix(_) => todo!(),
            ExpressionKind::Index(_) => todo!(),
            ExpressionKind::Call(_) => todo!(),
            ExpressionKind::MethodCall(_) => todo!(),
            ExpressionKind::Constructor(_) => todo!(),
            ExpressionKind::MemberAccess(_) => todo!(),
            ExpressionKind::Cast(_) => todo!(),
            ExpressionKind::Infix(_) => todo!(),
            ExpressionKind::If(_) => todo!(),
            ExpressionKind::Variable(_) => todo!(),
            ExpressionKind::Tuple(_) => todo!(),
            ExpressionKind::Lambda(_) => todo!(),
            ExpressionKind::Parenthesized(_) => todo!(),
            ExpressionKind::Quote(_) => todo!(),
            ExpressionKind::Comptime(_) => todo!(),
            ExpressionKind::Error => todo!(),
        }
    }

    pub fn compose_literal(&mut self, literal: Literal) -> Results {
        use functor::Literal::*;
        let (a, b, c) = match literal {
            Literal::Bool(value) => (Bool(value), Bool(value), Bool(value)),
            Literal::Integer(value, sign) => (Integer(value, sign), Integer(value, sign), Integer(value, sign)),
            Literal::Str(string) => (Str(string), Str(string), Str(string)),
            Literal::RawStr(string, x) => (RawStr(string, x), RawStr(string, x), RawStr(string, x)),
            Literal::FmtStr(string) => (FmtStr(string), FmtStr(string), FmtStr(string)),
            Literal::Unit => (Unit, Unit, Unit),
            Literal::Array(literal) => {
                let (a1, a2, a3) = self.compose_array(literal);
                apply3!(functor::Literal::Array, a1, a2, a3)
            }
            Literal::Slice(literal) => {
                let (a1, a2, a3) = self.compose_array(literal);
                apply3!(functor::Literal::Slice, a1, a2, a3)
            }
        };
        (a.resolve(self), b.type_check(self), c.scan(self))
    }

    pub fn compose_array(&mut self, array: ArrayLiteral) -> (functor::ArrayLiteral<ExprId>, functor::ArrayLiteral<Type>, functor::ArrayLiteral<IResult<()>>) {
        match array {
            ArrayLiteral::Standard(expressions) => {
                let (a, b, c) = trimap(expressions, |expr| self.compose_expression(expr));
                apply3!(functor::ArrayLiteral::Standard, a, b, c)
            },
            ArrayLiteral::Repeated { repeated_element, length } => {
                use functor::ArrayLiteral::Repeated;
                let (a1, b1, c1) = self.compose_expression(*repeated_element);
                let (a2, b2, c2) = self.compose_expression(*length);
                let resolved = Repeated { repeated_element: a1, length: a2 };
                let checked = Repeated { repeated_element: b1, length: b2 };
                let scanned = Repeated { repeated_element: c1, length: c2 };
                (resolved, checked, scanned)
            },
        }
    }
}

fn trimap<Iter, F, T, A, B, C>(iter: Iter, mut f: F) -> (Vec<A>, Vec<B>, Vec<C>) where
    F: FnMut(T) -> (A, B, C),
    Iter: IntoIterator<Item = T>
{
    let iter = iter.into_iter();
    let capacity = iter.size_hint().1.unwrap_or(1);
    let mut vec_a = Vec::with_capacity(capacity);
    let mut vec_b = Vec::with_capacity(capacity);
    let mut vec_c = Vec::with_capacity(capacity);

    for element in iter {
        let (a, b, c) = f(element);
        vec_a.push(a);
        vec_b.push(b);
        vec_c.push(c);
    }

    (vec_a, vec_b, vec_c)
}

type Results = (ExprId, Type, IResult<()>);

type ResolvedExpr = functor::Expression<ExprId>;
type TypeCheckedExpr = functor::Expression<Type>;
type ScannedExpr = functor::Expression<IResult<()>>;

type ResolvedLiteral = functor::Literal<ExprId>;
type TypeCheckedLiteral = functor::Literal<Type>;
type ScannedLiteral = functor::Literal<IResult<()>>;

impl ResolvedExpr {
    fn resolve(self, composer: &mut Composer) -> ExprId {
        todo!()
    }
}

impl TypeCheckedExpr {
    fn type_check(self, composer: &mut Composer) -> Type {
        todo!()
    }
}

impl ScannedExpr {
    fn scan(self, composer: &mut Composer) -> IResult<()> {
        todo!()
    }
}

impl ResolvedLiteral {
    fn resolve(self, composer: &mut Composer) -> ExprId {
        todo!()
    }
}

impl TypeCheckedLiteral {
    fn type_check(self, composer: &mut Composer) -> Type {
        todo!()
    }
}

impl ScannedLiteral {
    fn scan(self, composer: &mut Composer) -> IResult<()> {
        todo!()
    }
}
