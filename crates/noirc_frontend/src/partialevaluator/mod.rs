use std::collections::HashMap;

use acvm::FieldElement;
use noirc_errors::Location;

use crate::{
    monomorphisation::ast::{
        ArrayLiteral, Binary, CallBuiltin, CallLowLevel, Cast, DefinitionId, Expression, FuncId,
        Function, Index, Literal, Program, Type, Unary,
    },
    util::vecmap,
    BinaryOpKind, Signedness, UnaryOp,
};

pub fn evaluate(mut program: Program) -> Program {
    let main = program.main();

    let temporary_body = Expression::Literal(Literal::Bool(false));
    let mut new_main = Function {
        id: main.id,
        name: main.name.clone(),
        parameters: main.parameters.clone(),
        body: temporary_body,
        return_type: main.return_type.clone(),
    };

    let args = vec![unit(); main.parameters.len()];
    let main_id = program.main_id();

    let mut evaluator = Evaluator::new(&program);
    evaluator.function(main_id, args);

    // Return one big main function containing every statement
    new_main.body = Expression::Block(evaluator.evaluated);
    let abi = program.abi;
    Program::new(new_main, abi)
}

fn unit() -> Expression {
    Expression::Literal(Literal::Unit)
}

fn bool(value: bool) -> Expression {
    Expression::Literal(Literal::Bool(value))
}

fn int(value: FieldElement, typ: Type) -> Expression {
    let value = truncate(value, &typ);
    Expression::Literal(Literal::Integer(value, typ))
}

fn int_u128(value: u128, typ: Type) -> Expression {
    let value = truncate_u128(value, &typ);
    Expression::Literal(Literal::Integer(value, typ))
}

fn truncate(value: FieldElement, typ: &Type) -> FieldElement {
    match typ {
        Type::Integer(..) => truncate_u128(value.to_u128(), typ),
        _other => value,
    }
}

fn truncate_u128(value: u128, typ: &Type) -> FieldElement {
    match typ {
        Type::Integer(_, bits) => {
            let type_modulo = 1_u128 << bits;
            let value = value % type_modulo;
            FieldElement::from(value)
        }
        _other => FieldElement::from(value),
    }
}

fn is_zero(expr: &Expression) -> bool {
    match expr {
        Expression::Literal(Literal::Integer(value, _)) => value.is_zero(),
        _ => false,
    }
}

fn is_one(expr: &Expression) -> bool {
    match expr {
        Expression::Literal(Literal::Integer(value, _)) => value.is_one(),
        _ => false,
    }
}

type Scope = HashMap<DefinitionId, Expression>;

struct Evaluator<'a> {
    call_stack: Vec<Scope>,
    program: &'a Program,

    /// Already-evaluated expressions representing the full program once finished
    evaluated: Vec<Expression>,
}

impl<'a> Evaluator<'a> {
    fn new(program: &'a Program) -> Self {
        Self { program, call_stack: vec![], evaluated: vec![] }
    }

    fn current_scope(&mut self) -> &mut Scope {
        self.call_stack.last_mut().unwrap()
    }

    fn function(&mut self, f: FuncId, args: Vec<Expression>) -> Expression {
        let function = &self.program[f];
        assert_eq!(function.parameters.len(), args.len());

        let params_and_args = function.parameters.iter().zip(args);
        let scope = params_and_args.map(|((id, ..), arg)| (*id, arg)).collect();
        self.call_stack.push(scope);

        let ret = self.expression(&function.body);

        self.call_stack.pop();
        ret
    }

    fn expression(&mut self, expr: &Expression) -> Expression {
        match expr {
            Expression::Literal(literal) => self.literal(literal),
            Expression::Ident(ident) => self.ident(ident),
            Expression::Block(block) => self.block(block),
            Expression::Unary(unary) => self.unary(unary),
            Expression::Binary(binary) => self.binary(binary),
            Expression::Index(index) => self.index(index),
            Expression::Cast(cast) => self.cast(cast),
            Expression::For(for_loop) => self.for_loop(for_loop),
            Expression::If(if_expr) => self.if_expr(if_expr),
            Expression::Tuple(tuple) => self.tuple(tuple),
            Expression::ExtractTupleField(tuple, field) => self.extract(tuple, *field),
            Expression::Call(call) => self.call(call),
            Expression::CallBuiltin(builtin) => self.call_builtin(builtin),
            Expression::CallLowLevel(low_level) => self.call_low_level(low_level),
            Expression::Let(let_stmt) => self.let_statement(let_stmt),
            Expression::Constrain(expr, loc) => self.constrain(expr, *loc),
            Expression::Assign(assign) => self.assign(assign),
            Expression::Semi(expr) => self.expression(expr),
        }
    }

    fn literal(&mut self, literal: &Literal) -> Expression {
        Expression::Literal(match literal {
            Literal::Array(array) => {
                let contents = vecmap(&array.contents, |elem| self.expression(elem));
                Literal::Array(ArrayLiteral { contents, element_type: array.element_type.clone() })
            }
            Literal::Integer(value, typ) => Literal::Integer(value.clone(), typ.clone()),
            Literal::Bool(value) => Literal::Bool(*value),
            Literal::Str(value) => Literal::Str(value.clone()),
            Literal::Unit => Literal::Unit,
        })
    }

    fn ident(&self, ident: &crate::monomorphisation::ast::Ident) -> Expression {
        match self.call_stack.last().unwrap().get(&ident.id) {
            Some(value) => value.clone(),
            None => unreachable!(
                "Cannot find id {} for variable {}, not yet compiled",
                ident.id.0, ident.name
            ),
        }
    }

    fn block(&mut self, block: &[Expression]) -> Expression {
        let exprs = block.iter().take(block.len().saturating_sub(1));

        for expr in exprs {
            let new_expr = self.expression(expr);
            self.evaluated.push(new_expr);
        }

        if let Some(last_expr) = block.last() {
            self.expression(last_expr)
        } else {
            unit()
        }
    }

    fn unary(&mut self, unary: &Unary) -> Expression {
        let rhs = self.expression(&unary.rhs);

        match (unary.operator, rhs) {
            (UnaryOp::Minus, Expression::Literal(Literal::Integer(value, typ))) => match typ {
                Type::Field => int(-value, typ),
                Type::Integer(Signedness::Signed, bits) => {
                    if bits <= 128 && value.fits_in_u128() {
                        // -value = !value + 1 in two's complement
                        let value = FieldElement::from(!value.to_u128() + 1);
                        int(value, typ)
                    } else {
                        let rhs = Expression::Literal(Literal::Integer(value, typ));
                        Expression::Unary(Unary { operator: UnaryOp::Minus, rhs: Box::new(rhs) })
                    }
                }
                other => unreachable!("ICE: Expected integer type, got {}", other),
            },
            (UnaryOp::Minus, rhs) => {
                Expression::Unary(Unary { operator: UnaryOp::Minus, rhs: Box::new(rhs) })
            }
            (UnaryOp::Not, Expression::Literal(Literal::Integer(value, typ))) => match typ {
                Type::Field => unreachable!("Binary not operation invalid for field elements"),
                Type::Integer(Signedness::Signed, bits) => {
                    if bits <= 128 && value.fits_in_u128() {
                        let value = FieldElement::from(!value.to_u128());
                        int(value, typ)
                    } else {
                        let rhs = Expression::Literal(Literal::Integer(value, typ));
                        Expression::Unary(Unary { operator: UnaryOp::Minus, rhs: Box::new(rhs) })
                    }
                }
                other => unreachable!("ICE: Expected integer type, got {}", other),
            },
            (UnaryOp::Not, Expression::Literal(Literal::Bool(value))) => bool(!value),
            (UnaryOp::Not, rhs) => {
                Expression::Unary(Unary { operator: UnaryOp::Not, rhs: Box::new(rhs) })
            }
        }
    }

    fn binary(&mut self, binary: &Binary) -> Expression {
        let lhs = self.expression(&binary.lhs);
        let rhs = self.expression(&binary.rhs);

        if let Some(optimized) = binary_constant_int(&lhs, &rhs, binary.operator) {
            return optimized;
        }

        if let Some(optimized) = binary_constant_bool(&lhs, &rhs, binary.operator) {
            return optimized;
        }

        match binary_one_zero(&lhs, &rhs, binary.operator) {
            ReturnLhsOrRhs::Lhs => return lhs,
            ReturnLhsOrRhs::Rhs => return rhs,
            ReturnLhsOrRhs::Neither => (),
        }

        Expression::Binary(Binary {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            operator: binary.operator,
        })
    }

    fn index(&mut self, index: &Index) -> Expression {
        let collection = Box::new(self.expression(&index.collection));
        let index = Box::new(self.expression(&index.index));
        Expression::Index(Index { collection, index })
    }

    fn cast(&mut self, cast: &Cast) -> Expression {
        let lhs = Box::new(self.expression(&cast.lhs));
        Expression::Cast(Cast { lhs, r#type: cast.r#type.clone() })
    }

    fn for_loop(&mut self, for_loop: &crate::monomorphisation::ast::For) -> Expression {
        let start = match self.expression(&for_loop.start_range) {
            Expression::Literal(Literal::Integer(value, _)) if value.fits_in_u128() => {
                value.to_u128()
            }
            other => unreachable!(
                "Unable to evaluate comptime 'start range' value of for loop. Got {}",
                other
            ),
        };

        let end = match self.expression(&for_loop.end_range) {
            Expression::Literal(Literal::Integer(value, _)) if value.fits_in_u128() => {
                value.to_u128()
            }
            other => unreachable!(
                "Unable to evaluate comptime 'end range' value of for loop. Got {}",
                other
            ),
        };

        let contents = vecmap(start..end, |i| {
            // Don't need to push a new scope, name resolution ensures we cannot refer to the
            // loop variable outside of the loop
            let index = int_u128(i, for_loop.index_type.clone());
            self.current_scope().insert(for_loop.index_variable, index);
            self.expression(&for_loop.block)
        });

        Expression::Literal(Literal::Array(ArrayLiteral {
            contents,
            element_type: for_loop.element_type.clone(),
        }))
    }

    fn if_expr(&mut self, if_expr: &crate::monomorphisation::ast::If) -> Expression {
        todo!()
    }

    fn tuple(&mut self, tuple: &[Expression]) -> Expression {
        let fields = vecmap(tuple, |field| self.expression(field));
        Expression::Tuple(fields)
    }

    fn extract(&mut self, tuple: &Expression, field: usize) -> Expression {
        match self.expression(tuple) {
            Expression::Tuple(mut fields) => fields.swap_remove(field),
            tuple => {
                // Is this case reachable?
                Expression::ExtractTupleField(Box::new(tuple), field)
            }
        }
    }

    fn call(&mut self, call: &crate::monomorphisation::ast::Call) -> Expression {
        let args = vecmap(&call.arguments, |arg| self.expression(arg));
        self.function(call.func_id, args)
    }

    fn call_builtin(&mut self, call: &CallBuiltin) -> Expression {
        let arguments = vecmap(&call.arguments, |arg| self.expression(arg));
        Expression::CallBuiltin(CallBuiltin { opcode: call.opcode.clone(), arguments })
    }

    fn call_low_level(&mut self, call: &CallLowLevel) -> Expression {
        let arguments = vecmap(&call.arguments, |arg| self.expression(arg));
        Expression::CallLowLevel(CallLowLevel { opcode: call.opcode.clone(), arguments })
    }

    fn let_statement(&mut self, let_stmt: &crate::monomorphisation::ast::Let) -> Expression {
        let expression = self.expression(&let_stmt.expression);
        self.current_scope().insert(let_stmt.id, expression);
        unit()
    }

    fn constrain(&mut self, expr: &Expression, loc: Location) -> Expression {
        let expr = self.expression(expr);
        Expression::Constrain(Box::new(expr), loc)
    }

    fn assign(&mut self, assign: &crate::monomorphisation::ast::Assign) -> Expression {
        todo!()
    }
}

/// Basic optimizations: both are constant ints
fn binary_constant_int(
    lhs: &Expression,
    rhs: &Expression,
    operator: BinaryOpKind,
) -> Option<Expression> {
    if let (
        Expression::Literal(Literal::Integer(lvalue, ltyp)),
        Expression::Literal(Literal::Integer(rvalue, rtyp)),
    ) = (&lhs, &rhs)
    {
        assert_eq!(ltyp, rtyp);
        match operator {
            BinaryOpKind::Add => return Some(int(*lvalue + *rvalue, ltyp.clone())),
            BinaryOpKind::Subtract => return Some(int(*lvalue - *rvalue, ltyp.clone())),
            BinaryOpKind::Multiply => return Some(int(*lvalue * *rvalue, ltyp.clone())),
            BinaryOpKind::Divide => return Some(int(*lvalue / *rvalue, ltyp.clone())),
            BinaryOpKind::Equal => return Some(bool(lvalue == rvalue)),
            BinaryOpKind::NotEqual => return Some(bool(*lvalue != *rvalue)),
            BinaryOpKind::Less => return Some(bool(*lvalue < *rvalue)),
            BinaryOpKind::LessEqual => return Some(bool(*lvalue <= *rvalue)),
            BinaryOpKind::Greater => return Some(bool(*lvalue > *rvalue)),
            BinaryOpKind::GreaterEqual => return Some(bool(*lvalue >= *rvalue)),
            _ => (),
        };

        if let (Some(lvalue), Some(rvalue)) = (lvalue.try_into_u128(), rvalue.try_into_u128()) {
            match operator {
                BinaryOpKind::And => return Some(int_u128(lvalue & rvalue, ltyp.clone())),
                BinaryOpKind::Or => return Some(int_u128(lvalue | rvalue, ltyp.clone())),
                BinaryOpKind::Xor => return Some(int_u128(lvalue ^ rvalue, ltyp.clone())),
                BinaryOpKind::ShiftRight => return Some(int_u128(lvalue >> rvalue, ltyp.clone())),
                BinaryOpKind::ShiftLeft => return Some(int_u128(lvalue << rvalue, ltyp.clone())),
                BinaryOpKind::Modulo => return Some(int_u128(lvalue % rvalue, ltyp.clone())),
                _ => (),
            }
        }
    }
    None
}

/// Basic optimizations: both are constant bools
fn binary_constant_bool(
    lhs: &Expression,
    rhs: &Expression,
    operator: BinaryOpKind,
) -> Option<Expression> {
    if let (
        Expression::Literal(Literal::Bool(lvalue)),
        Expression::Literal(Literal::Bool(rvalue)),
    ) = (&lhs, &rhs)
    {
        Some(match operator {
            BinaryOpKind::Equal => bool(lvalue == rvalue),
            BinaryOpKind::NotEqual => bool(*lvalue != *rvalue),
            BinaryOpKind::And => bool(*lvalue && *rvalue),
            BinaryOpKind::Or => bool(*lvalue || *rvalue),
            BinaryOpKind::Xor => bool(*lvalue ^ *rvalue),
            _ => return None,
        })
    } else {
        None
    }
}

enum ReturnLhsOrRhs {
    Lhs,
    Rhs,
    Neither,
}

/// Other optimizations for 1 and 0 constants
/// This returns a 'ReturnLhsOrRhs' - if we wanted to return lhs or rhs
/// directly we'd need to take ownership of them or clone them.
fn binary_one_zero(lhs: &Expression, rhs: &Expression, operator: BinaryOpKind) -> ReturnLhsOrRhs {
    if is_zero(&lhs) {
        match operator {
            BinaryOpKind::Add => return ReturnLhsOrRhs::Rhs,
            _ => (),
        }
    }

    if is_zero(&rhs) {
        match operator {
            BinaryOpKind::Add => return ReturnLhsOrRhs::Lhs,
            BinaryOpKind::Subtract => return ReturnLhsOrRhs::Lhs,
            _ => (),
        }
    }

    if is_one(&lhs) {
        match operator {
            BinaryOpKind::Multiply => return ReturnLhsOrRhs::Rhs,
            _ => (),
        }
    }

    if is_one(&rhs) {
        match operator {
            BinaryOpKind::Multiply => return ReturnLhsOrRhs::Lhs,
            BinaryOpKind::Divide => return ReturnLhsOrRhs::Lhs,
            BinaryOpKind::Modulo => return ReturnLhsOrRhs::Lhs,
            _ => (),
        }
    }
    ReturnLhsOrRhs::Neither
}
