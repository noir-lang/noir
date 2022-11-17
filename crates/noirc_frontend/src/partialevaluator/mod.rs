use std::{collections::HashMap, rc::Rc};

use acvm::FieldElement;
use noirc_errors::Location;

use crate::{
    monomorphisation::ast::{
        ArrayLiteral, Assign, Binary, CallBuiltin, CallLowLevel, Cast, Definition, Expression,
        FuncId, Function, Ident, If, Index, LValue, Let, Literal, Program, SharedId, Type, Unary,
    },
    util::vecmap,
    BinaryOpKind, Signedness, UnaryOp,
};

pub fn evaluate(mut program: Program) -> Program {
    let main_id = program.main_id();
    let main = program.main();

    let temporary_body = Expression::Literal(Literal::Bool(false));
    let mut new_main = Function {
        id: main.id,
        name: main.name.clone(),
        parameters: main.parameters.clone(),
        body: temporary_body,
        return_type: main.return_type.clone(),
    };

    let args = vecmap(&main.parameters, |(id, mutable, name, typ)| {
        Expression::Ident(Ident {
            location: None,
            definition: Definition::Local(*id),
            mutable: *mutable,
            name: name.clone(),
            typ: typ.clone(),
        })
    });

    // Evaluate main and grab the resulting statements
    let mut evaluator = Evaluator::new(&program);
    let last_expr = evaluator.function_with_id(main_id, args);
    let mut statements = evaluator.evaluated.pop().unwrap();
    assert!(evaluator.evaluated.is_empty());

    statements.push(last_expr);

    // Return one big main function containing every statement
    new_main.body = Expression::Block(statements);
    Program::new(new_main, program.abi)
}

type Scope = HashMap<Definition, Expression>;

struct Evaluator<'a> {
    call_stack: Vec<Scope>,
    program: &'a Program,

    /// Already-evaluated expressions representing the full program once finished
    evaluated: Vec<Vec<Expression>>,
    counter: u32,
}

impl<'a> Evaluator<'a> {
    fn new(program: &'a Program) -> Self {
        Self { program, call_stack: vec![], evaluated: vec![vec![]], counter: 0 }
    }

    fn current_scope(&mut self) -> &mut Scope {
        self.call_stack.last_mut().unwrap()
    }

    fn push_expression(&mut self, expr: Expression) {
        if expr.has_side_effects() {
            self.evaluated.last_mut().unwrap().push(expr);
        }
    }

    fn get_function(&self, f: &Expression) -> Option<Definition> {
        match f {
            Expression::Ident(variable) => match &variable.definition {
                Definition::Local(_) => None,
                other => Some(other.clone()),
            },
            _ => None,
        }
    }

    fn next_unique_id(&mut self) -> SharedId {
        self.counter += 1;
        SharedId(self.counter)
    }

    fn function(&mut self, f: Expression, arguments: Vec<Expression>) -> Expression {
        // TODO: Need a better error message here. Need to add Spans to monomorphised asts
        match self.get_function(&f) {
            Some(Definition::Function(id)) => self.function_with_id(id, arguments),
            Some(Definition::Builtin(opcode)) => {
                Expression::CallBuiltin(CallBuiltin { opcode, arguments })
            }
            Some(Definition::LowLevel(opcode)) => {
                Expression::CallLowLevel(CallLowLevel { opcode, arguments })
            }
            Some(Definition::Local(_)) => unreachable!(),
            None => panic!("Function must be known at compile-time but is not: {}", f),
        }
    }

    fn function_with_id(&mut self, f: FuncId, args: Vec<Expression>) -> Expression {
        // TODO: Need a better error message here. Need to add Spans to monomorphised asts
        let function = &self.program[f];
        assert_eq!(function.parameters.len(), args.len());

        let params_and_args = function.parameters.iter().zip(args);
        let scope = params_and_args
            .map(|((id, mutable, name, typ), arg)| {
                let definition = Definition::Local(*id);
                let arg = if *mutable {
                    self.push_expression(Expression::Let(Let {
                        id: *id,
                        mutable: true,
                        name: name.clone(),
                        expression: Box::new(arg),
                    }));
                    Expression::Ident(Ident {
                        location: None,
                        definition: definition.clone(),
                        mutable: *mutable,
                        name: name.clone(),
                        typ: typ.clone(),
                    })
                } else {
                    arg
                };
                (definition, arg)
            })
            .collect();

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
            Expression::Semi(expr) => Expression::Semi(Box::new(self.expression(expr))),

            Expression::Shared(..) => unreachable!(),
        }
    }

    fn literal(&mut self, literal: &Literal) -> Expression {
        Expression::Literal(match literal {
            Literal::Array(array) => {
                let contents = vecmap(&array.contents, |elem| self.expression(elem));
                Literal::Array(ArrayLiteral { contents, element_type: array.element_type.clone() })
            }
            Literal::Integer(value, typ) => Literal::Integer(*value, typ.clone()),
            Literal::Bool(value) => Literal::Bool(*value),
            Literal::Str(value) => Literal::Str(value.clone()),
            Literal::Unit => Literal::Unit,
        })
    }

    fn ident(&self, ident: &Ident) -> Expression {
        let make_ident = || {
            Expression::Ident(Ident {
                location: ident.location,
                definition: ident.definition.clone(),
                mutable: ident.mutable,
                name: ident.name.clone(),
                typ: ident.typ.clone(),
            })
        };

        // Cloning here relies on `value` containing no side-effectful code.
        // Side-effectful code should be pushed to self.evaluated separately
        self.call_stack.last().unwrap().get(&ident.definition).cloned().unwrap_or_else(make_ident)
    }

    fn block(&mut self, block: &[Expression]) -> Expression {
        let exprs = block.iter().take(block.len().saturating_sub(1));

        for expr in exprs {
            let new_expr = self.expression(expr);
            self.push_expression(new_expr);
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

    fn index(&mut self, index_expr: &Index) -> Expression {
        let collection = self.expression(&index_expr.collection);
        let index = self.expression(&index_expr.index);

        if let Some((i, _)) = as_int(&index) {
            if i.try_to_u64().map_or(true, |i| i >= index_expr.collection_length) {
                return unreachable();
            }
        }

        if let (Some(array), Some((index, _))) = (as_array(&collection), as_int(&index)) {
            if let Some(index) = index.try_to_u64().and_then(|x| x.try_into().ok()) {
                let _: usize = index;
                return array.contents[index].clone();
            }
        }

        let collection = Box::new(collection);
        let index = Box::new(index);
        let collection_length = index_expr.collection_length;
        Expression::Index(Index { collection, index, collection_length })
    }

    fn cast(&mut self, cast: &Cast) -> Expression {
        let lhs = Box::new(self.expression(&cast.lhs));
        Expression::Cast(Cast { lhs, r#type: cast.r#type.clone() })
    }

    fn for_loop(&mut self, for_loop: &crate::monomorphisation::ast::For) -> Expression {
        let start = self.expression(&for_loop.start_range);
        let start = match as_int(&start) {
            Some((value, _)) if value.fits_in_u128() => value.to_u128(),
            _ => unreachable!(
                "Unable to evaluate comptime 'start range' value of for loop. Got {}",
                start
            ),
        };

        let end = self.expression(&for_loop.end_range);
        let end = match as_int(&end) {
            Some((value, _)) if value.fits_in_u128() => value.to_u128(),
            _ => unreachable!(
                "Unable to evaluate comptime 'end range' value of for loop. Got {}",
                end
            ),
        };

        let contents = vecmap(start..end, |i| {
            // Don't need to push a new scope, name resolution ensures we cannot refer to the
            // loop variable outside of the loop
            let index = int_u128(i, for_loop.index_type.clone());
            self.current_scope().insert(Definition::Local(for_loop.index_variable), index);
            self.expression(&for_loop.block)
        });

        Expression::Literal(Literal::Array(ArrayLiteral {
            contents,
            element_type: for_loop.element_type.clone(),
        }))
    }

    fn if_expr(&mut self, if_expr: &If) -> Expression {
        let condition = match self.expression(&if_expr.condition) {
            Expression::Literal(Literal::Bool(true)) => {
                return self.expression(&if_expr.consequence)
            }
            Expression::Literal(Literal::Bool(false)) => {
                if let Some(alt) = &if_expr.alternative {
                    return self.expression(alt);
                } else {
                    return unit();
                }
            }
            other => other,
        };

        // Otherwise continue with a non-comptime condition
        let condition = Box::new(condition);

        // Must separate out evaluated side effects (*_evaluated) from the
        // non-side effectful expression that is returned, which may be
        // stored in a variable and cloned
        self.evaluated.push(vec![]);
        let consequence = Box::new(self.expression(&if_expr.consequence));
        let mut consequence_evaluated = self.evaluated.pop().unwrap();

        let (alternative, alternative_evaluated) = if let Some(alt) = &if_expr.alternative {
            self.evaluated.push(vec![]);
            let alt = Box::new(self.expression(alt));
            let alt_eval = self.evaluated.pop().unwrap();
            let alt_eval = if alt_eval.is_empty() { None } else { Some(alt_eval) };
            (Some(alt), alt_eval)
        } else {
            (None, None)
        };

        // Check if the if-expr's type is Unit and if so, re-combine the evaluated
        // statements and resulting expression, then directly return a unit literal.
        // This isn't necessary but cleans up the output somewhat.
        if if_expr.typ == Type::Unit {
            consequence_evaluated.push(*consequence);
            let alternatives = match (alternative_evaluated, alternative) {
                (Some(mut alternatives), Some(alternative)) => {
                    alternatives.push(*alternative);
                    Some(alternatives)
                }
                (None, Some(alternative)) => Some(vec![*alternative]),
                (None, None) => None,
                (Some(_), None) => unreachable!(),
            };

            self.push_expression(Expression::If(If {
                condition,
                consequence: Box::new(Expression::Block(consequence_evaluated)),
                alternative: alternatives.map(|alts| Box::new(Expression::Block(alts))),
                typ: if_expr.typ.clone(),
            }));
            unit()
        } else {
            if !consequence_evaluated.is_empty()
                || alternative_evaluated.as_ref().map_or(false, |alt| alt.is_empty())
            {
                self.push_expression(Expression::If(If {
                    condition: condition.clone(),
                    consequence: Box::new(Expression::Block(consequence_evaluated)),
                    alternative: alternative_evaluated.map(|alt| Box::new(Expression::Block(alt))),
                    typ: if_expr.typ.clone(),
                }));
            }

            Expression::If(If { condition, consequence, alternative, typ: if_expr.typ.clone() })
        }
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
        let function = self.expression(&call.func);
        self.function(function, args)
    }

    fn call_builtin(&mut self, call: &CallBuiltin) -> Expression {
        let arguments = vecmap(&call.arguments, |arg| self.expression(arg));
        Expression::CallBuiltin(CallBuiltin { opcode: call.opcode.clone(), arguments })
    }

    fn call_low_level(&mut self, call: &CallLowLevel) -> Expression {
        let arguments = vecmap(&call.arguments, |arg| self.expression(arg));
        Expression::CallLowLevel(CallLowLevel { opcode: call.opcode.clone(), arguments })
    }

    fn let_statement(&mut self, let_stmt: &Let) -> Expression {
        let expression = self.expression(&let_stmt.expression);
        if let_stmt.mutable || expression.contains_variables() {
            self.push_expression(Expression::Let(Let {
                id: let_stmt.id,
                mutable: true,
                name: let_stmt.name.clone(),
                expression: Box::new(expression),
            }));
        } else {
            let id = self.next_unique_id();
            let e = Expression::Shared(id, Rc::new(expression));
            self.current_scope().insert(Definition::Local(let_stmt.id), e);
        }
        unit()
    }

    fn constrain(&mut self, expr: &Expression, loc: Option<Location>) -> Expression {
        let expr = self.expression(expr);
        self.push_expression(Expression::Constrain(Box::new(expr), loc));
        unit()
    }

    fn assign(&mut self, assign: &Assign) -> Expression {
        let expression = Box::new(self.expression(&assign.expression));

        let expr = match self.lvalue(&assign.lvalue) {
            Some(lvalue) => Expression::Assign(Assign { lvalue, expression }),
            None => unreachable(),
        };

        self.push_expression(expr);
        unit()
    }

    /// This returns an Option LValue because it can fail if there is an index
    /// expression that uses an index that is greater than or equal to the length
    /// of the array.
    fn lvalue(&mut self, lvalue: &LValue) -> Option<LValue> {
        match lvalue {
            LValue::Ident(ident) => Some(LValue::Ident(ident.clone())),
            LValue::Index { array, index, array_len } => {
                let array = self.lvalue(array)?;
                let index = self.expression(index);

                if let Some((index, _)) = as_int(&index) {
                    if index.try_to_u64().map_or(true, |i| i >= *array_len) {
                        return None;
                    }
                }

                let array = Box::new(array);
                let index = Box::new(index);
                let array_len = *array_len;
                Some(LValue::Index { array, index, array_len })
            }
            LValue::MemberAccess { object, field_index } => {
                let object = Box::new(self.lvalue(object)?);
                let field_index = *field_index;
                Some(LValue::MemberAccess { object, field_index })
            }
        }
    }
}

/// Basic optimizations: both are constant ints
fn binary_constant_int(
    lhs: &Expression,
    rhs: &Expression,
    operator: BinaryOpKind,
) -> Option<Expression> {
    if let (Some((lvalue, ltyp)), Some((rvalue, rtyp))) = (as_int(lhs), as_int(rhs)) {
        assert_eq!(ltyp, rtyp);
        match operator {
            BinaryOpKind::Add => return Some(int(lvalue + rvalue, ltyp.clone())),
            BinaryOpKind::Subtract => return Some(int(lvalue - rvalue, ltyp.clone())),
            BinaryOpKind::Multiply => return Some(int(lvalue * rvalue, ltyp.clone())),
            BinaryOpKind::Divide => return Some(int(lvalue / rvalue, ltyp.clone())),
            BinaryOpKind::Equal => return Some(bool(lvalue == rvalue)),
            BinaryOpKind::NotEqual => return Some(bool(lvalue != rvalue)),
            BinaryOpKind::Less => return Some(bool(lvalue < rvalue)),
            BinaryOpKind::LessEqual => return Some(bool(lvalue <= rvalue)),
            BinaryOpKind::Greater => return Some(bool(lvalue > rvalue)),
            BinaryOpKind::GreaterEqual => return Some(bool(lvalue >= rvalue)),
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
    if let (Some(lvalue), Some(rvalue)) = (as_bool(lhs), as_bool(rhs)) {
        Some(match operator {
            BinaryOpKind::Equal => bool(lvalue == rvalue),
            BinaryOpKind::NotEqual => bool(lvalue != rvalue),
            BinaryOpKind::And => bool(lvalue && rvalue),
            BinaryOpKind::Or => bool(lvalue || rvalue),
            BinaryOpKind::Xor => bool(lvalue ^ rvalue),
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
    if is_zero(lhs) && operator == BinaryOpKind::Add {
        return ReturnLhsOrRhs::Rhs;
    }

    if is_zero(rhs) {
        match operator {
            BinaryOpKind::Add => return ReturnLhsOrRhs::Lhs,
            BinaryOpKind::Subtract => return ReturnLhsOrRhs::Lhs,
            _ => (),
        }
    }

    if is_one(lhs) && operator == BinaryOpKind::Multiply {
        return ReturnLhsOrRhs::Rhs;
    }

    if is_one(rhs) {
        match operator {
            BinaryOpKind::Multiply => return ReturnLhsOrRhs::Lhs,
            BinaryOpKind::Divide => return ReturnLhsOrRhs::Lhs,
            BinaryOpKind::Modulo => return ReturnLhsOrRhs::Lhs,
            _ => (),
        }
    }
    ReturnLhsOrRhs::Neither
}

fn unit() -> Expression {
    Expression::Literal(Literal::Unit)
}

fn unreachable() -> Expression {
    let false_expr = Expression::Literal(Literal::Bool(false));
    Expression::Constrain(Box::new(false_expr), None)
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

fn as_bool(expr: &Expression) -> Option<bool> {
    match expr {
        Expression::Literal(Literal::Bool(value)) => Some(*value),
        Expression::Shared(_, expr) => as_bool(expr),
        _ => None,
    }
}

fn as_int(expr: &Expression) -> Option<(FieldElement, &Type)> {
    match expr {
        Expression::Literal(Literal::Integer(value, typ)) => Some((*value, typ)),
        Expression::Shared(_, expr) => as_int(expr),
        _ => None,
    }
}

fn as_array(expr: &Expression) -> Option<&ArrayLiteral> {
    match expr {
        Expression::Literal(Literal::Array(array)) => Some(array),
        Expression::Shared(_, expr) => as_array(expr),
        _ => None,
    }
}

fn is_zero(expr: &Expression) -> bool {
    as_int(expr).map_or(false, |(int, _)| int.is_zero())
}

fn is_one(expr: &Expression) -> bool {
    as_int(expr).map_or(false, |(int, _)| int.is_one())
}
