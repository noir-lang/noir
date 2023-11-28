use std::path::Path;

use crate::{
    ast, graph::CrateId, hir::def_map::Visibility, parser::SortedModule, AssignStatement,
    BlockExpression, CallExpression, CastExpression, Expression as NoirExpression, ExpressionKind,
    FunctionDefinition as NoirFunctionDefinition, FunctionReturnType, Ident as NoirIdent,
    IfExpression, IndexExpression, InfixExpression, LValue, LetStatement, Literal,
    MemberAccessExpression, MethodCallExpression, NoirFunction, Path as NoirPath, Pattern,
    PrefixExpression, Statement as NoirStatement, StatementKind, UnaryOp, UnresolvedType,
    UnresolvedTypeData,
};
use acvm::FieldElement;
use fm::{FileId, FileManager};
use noirc_errors::{Span, Spanned};
use solang_parser::{
    parse,
    pt::{
        ContractPart, Expression as SolExpression, FunctionDefinition as SolFunction,
        Identifier as SolIdent, Parameter as SolParameter, ParameterList, SourceUnitPart,
        Statement as SolStatement,
    },
};

use crate::{parser::ParserError, BinaryOpKind};

pub fn parse_sol_file(fm: &FileManager, file_id: FileId) -> (SortedModule, Vec<ParserError>) {
    let file = fm.fetch_file(file_id);

    // TODO: bring in errors
    let errors = vec![];
    (parse_sol(file.source()), errors)
}

pub fn parse_sol(text: &str) -> SortedModule {
    let (tree, _) = parse(&text, 0).unwrap();

    let mut ast = SortedModule::default();

    for part in &tree.0 {
        match part {
            SourceUnitPart::ContractDefinition(def) => {
                println!("found contract {:?}", def.name);
                for part in &def.parts {
                    match part {
                        ContractPart::VariableDefinition(def) => {
                            println!("variable {:?}", def.name);
                        }
                        ContractPart::FunctionDefinition(def) => {
                            println!("function {:?}", def.name);
                            let transformed = transform_function(&def);
                            ast.functions.push(transformed);
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    ast
}

fn transform_function(sol_function: &SolFunction) -> NoirFunction {
    let params = transform_parameters(&sol_function.params);

    // TODO: yeet clone
    let body = transform_body(sol_function.body.clone());
    let return_type = transform_return_type(&sol_function.returns.as_ref());

    // Ignore generics and trait bounds
    let generics = Vec::new();
    let trait_bounds = Vec::new();

    // TODO: remove unwrap
    let name_ident = transform_ident(&sol_function.name.as_ref().unwrap());

    let mut noir_function = NoirFunctionDefinition::normal(
        &name_ident,
        &generics,
        &params,
        &body,
        &trait_bounds,
        &return_type,
    );

    // If we have a return type then we need to make it public; this is not handled by the normal function definition above
    // TODO: probably more sense to match this on the name being main
    if !matches!(return_type, FunctionReturnType::Default(_)) {
        noir_function.return_visibility = ast::Visibility::Public;
    }

    NoirFunction::normal(noir_function)
}

fn transform_ident(identifier: &SolIdent) -> NoirIdent {
    NoirIdent::new(identifier.name.clone(), Span::default())
}

fn transform_parameters(sol_params: &ParameterList) -> Vec<(NoirIdent, UnresolvedType)> {
    // Filter out the spans
    let params: Vec<&SolParameter> =
        sol_params.iter().map(|param| &param.1).filter_map(|v| v.as_ref()).collect();

    let mut out_params = Vec::new();

    for param in params {
        let name = transform_ident(&param.name.as_ref().expect("Must have a name?"));
        let ty = make_type(UnresolvedTypeData::FieldElement);
        out_params.push((name, ty));
    }

    out_params
}

fn transform_body(sol_body: Option<SolStatement>) -> BlockExpression {
    let sol_body = sol_body.expect("Must have a body");

    let statements = resolve_statement(sol_body);
    BlockExpression(statements)
}

fn resolve_statement(sol_body: SolStatement) -> Vec<NoirStatement> {
    let mut collected_statements: Vec<NoirStatement> = Vec::new();
    match sol_body {
        SolStatement::Block { loc, unchecked, statements } => {
            for statement in statements {
                collected_statements.append(&mut resolve_statement(statement));
            }
        }
        SolStatement::Expression(_, sol_expression) => {
            let expression = resolve_expression(sol_expression);
            let express_statement = semi_expression(expression);
            collected_statements.push(express_statement);
        }
        SolStatement::Return(_, sol_expression) => {
            if let Some(return_exp) = sol_expression {
                let expression = resolve_expression(return_exp);
                let express_statement = statement_expression(expression);
                collected_statements.push(express_statement);
            }
        }
        SolStatement::VariableDefinition(_, var_def, expression_opt) => {
            // TODO: resolve the type, just use field for now
            let ty = make_type(UnresolvedTypeData::FieldElement);
            let name = &var_def.name.unwrap().name;

            let assign = if let Some(expression) = expression_opt {
                let exp = resolve_expression(expression);
                mutable_assignment(name, exp)
            } else {
                let val = make_numeric_literal("0".to_string());
                mutable_assignment(name, val)
            };
            collected_statements.push(assign);
        }
        SolStatement::If(_, expr, inner, outer) => {
            // TODO Note if in an if statement
            // Early return is NOT supported

            let expr = resolve_expression(expr);
            let inner2 = block_expression(resolve_statement(*inner));
            let outer2 = outer.clone();
            let outer3 = outer2
                .clone()
                .and(Some(block_expression(resolve_statement(*(outer2.unwrap().clone())))));

            collected_statements.push(make_if(expr, inner2, outer3));
        }
        _ => panic!("Not implemented statement, {sol_body}"),
    }
    collected_statements
}

fn resolve_expression(sol_expression: SolExpression) -> NoirExpression {
    dbg!(&sol_expression);
    match sol_expression {
        SolExpression::Add(_, lhs, rhs) => {
            let lhs = resolve_expression(*lhs);
            let rhs = resolve_expression(*rhs);
            let op = BinaryOpKind::Add;
            infix_expression(lhs, rhs, op)
        }
        SolExpression::Subtract(_, lhs, rhs) => {
            let lhs = resolve_expression(*lhs);
            let rhs = resolve_expression(*rhs);
            let op = BinaryOpKind::Subtract;
            infix_expression(lhs, rhs, op)
        }
        SolExpression::Variable(ident) => {
            let ident = transform_ident(&ident);
            variable_ident(ident)
        }
        // TODO: support exp / unit
        // Value is the most common
        // exp is if the number is exponented?
        // unit is days / ether that can follow
        SolExpression::NumberLiteral(_, val, _exp, _unit) => make_numeric_literal(val),
        SolExpression::Equal(_, lhs, rhs) => {
            let lhs = resolve_expression(*lhs);
            let rhs = resolve_expression(*rhs);
            let op = BinaryOpKind::Equal;
            infix_expression(lhs, rhs, op)
        }
        SolExpression::Assign(_, lhs, rhs) => {
            dbg!("assignment");
            dbg!(&lhs);
            dbg!(&rhs);
            let lhs = resolve_expression(*lhs);
            let rhs = resolve_expression(*rhs);

            dbg!(&lhs);
            dbg!(&rhs);

            // yuck
            block_expression(vec![var_assignment(lhs, rhs)])
        }

        _ => panic!("Not implemented expression, {sol_expression}"),
    }
}

// fn transform_statement() -> NoirStatement {}

fn transform_return_type(sol_params: &ParameterList) -> FunctionReturnType {
    // Filter out the spans
    let params: Vec<&SolParameter> =
        sol_params.iter().map(|param| &param.1).filter_map(|v| v.as_ref()).collect();

    dbg!(&params);

    if params.len() > 0 {
        let ty = make_type(UnresolvedTypeData::FieldElement);
        FunctionReturnType::Ty(ty)
    } else {
        FunctionReturnType::Default(Span::default())
    }
}

//
//
//
//
//
//
//
//
//             Helpers for creating noir ast nodes
//
fn make_ident(name: &str) -> NoirIdent {
    NoirIdent::new(name.to_string(), Span::default())
}

fn ident_path(name: &str) -> NoirPath {
    NoirPath::from_ident(make_ident(name))
}

fn path(ident: NoirIdent) -> NoirPath {
    NoirPath::from_ident(ident)
}

fn expression(kind: ExpressionKind) -> NoirExpression {
    NoirExpression::new(kind, Span::default())
}

fn block_expression(statements: Vec<NoirStatement>) -> NoirExpression {
    expression(ExpressionKind::Block(BlockExpression(statements)))
}

fn infix_expression(
    lhs: NoirExpression,
    rhs: NoirExpression,
    operator: BinaryOpKind,
) -> NoirExpression {
    expression(ExpressionKind::Infix(Box::new(InfixExpression {
        lhs,
        rhs,
        operator: Spanned::from(Span::default(), operator),
    })))
}

fn variable(name: &str) -> NoirExpression {
    expression(ExpressionKind::Variable(ident_path(name)))
}

fn variable_ident(identifier: NoirIdent) -> NoirExpression {
    expression(ExpressionKind::Variable(path(identifier)))
}

fn variable_path(path: NoirPath) -> NoirExpression {
    expression(ExpressionKind::Variable(path))
}

fn method_call(
    object: NoirExpression,
    method_name: &str,
    arguments: Vec<NoirExpression>,
) -> NoirExpression {
    expression(ExpressionKind::MethodCall(Box::new(MethodCallExpression {
        object,
        method_name: make_ident(method_name),
        arguments,
    })))
}

fn call(func: NoirExpression, arguments: Vec<NoirExpression>) -> NoirExpression {
    expression(ExpressionKind::Call(Box::new(CallExpression { func: Box::new(func), arguments })))
}

fn pattern(name: &str) -> Pattern {
    Pattern::Identifier(make_ident(name))
}

fn mutable(name: &str) -> Pattern {
    Pattern::Mutable(Box::new(pattern(name)), Span::default())
}

fn mutable_assignment(name: &str, assigned_to: NoirExpression) -> NoirStatement {
    make_statement(StatementKind::Let(LetStatement {
        pattern: mutable(name),
        r#type: make_type(UnresolvedTypeData::Unspecified),
        expression: assigned_to,
    }))
}

fn mutable_reference(variable_name: &str) -> NoirExpression {
    expression(ExpressionKind::Prefix(Box::new(PrefixExpression {
        operator: UnaryOp::MutableReference,
        rhs: variable(variable_name),
    })))
}

fn let_assignment(name: &str, assigned_to: NoirExpression) -> NoirStatement {
    make_statement(StatementKind::Let(LetStatement {
        pattern: pattern(name),
        r#type: make_type(UnresolvedTypeData::Unspecified),
        expression: assigned_to,
    }))
}

fn var_assignment(var: NoirExpression, assigned_to: NoirExpression) -> NoirStatement {
    // TODO: yuck
    let name = match var.kind {
        ExpressionKind::Variable(path) => path.segments.last().unwrap().0.contents.clone(),
        _ => panic!("Not a variable"),
    };
    make_statement(StatementKind::Assign(AssignStatement {
        lvalue: LValue::Ident(make_ident(&name)),
        expression: assigned_to,
    }))
}
fn assignment(name: &str, assigned_to: NoirExpression) -> NoirStatement {
    make_statement(StatementKind::Assign(AssignStatement {
        lvalue: LValue::Ident(make_ident(name)),
        expression: assigned_to,
    }))
}

fn statement_expression(expression: NoirExpression) -> NoirStatement {
    make_statement(StatementKind::Expression(expression))
}

// This is the most likely in this context
fn semi_expression(expression: NoirExpression) -> NoirStatement {
    make_statement(StatementKind::Semi(expression))
}

fn member_access(lhs: &str, rhs: &str) -> NoirExpression {
    expression(ExpressionKind::MemberAccess(Box::new(MemberAccessExpression {
        lhs: variable(lhs),
        rhs: make_ident(rhs),
    })))
}

fn make_statement(kind: StatementKind) -> NoirStatement {
    NoirStatement { span: Span::default(), kind }
}

fn make_if(
    condition: NoirExpression,
    consequence: NoirExpression,
    alternative: Option<NoirExpression>,
) -> NoirStatement {
    make_statement(StatementKind::Expression(expression(ExpressionKind::If(Box::new(
        IfExpression { condition, consequence, alternative },
    )))))
}

fn make_numeric_literal(number: String) -> NoirExpression {
    // expression(ExpressionKind::Literal(Literal::Integer(FieldElement::from_hex(&number).unwrap())))
    // convert from string to number
    let number = number.parse::<u128>().unwrap();
    expression(ExpressionKind::Literal(Literal::Integer(FieldElement::from(number))))
}

macro_rules! chained_path {
    ( $base:expr $(, $tail:expr)* ) => {
        {
            let mut base_path = ident_path($base);
            $(
                base_path.segments.push(ident($tail));
            )*
            base_path
        }
    }
}

macro_rules! chained_dep {
    ( $base:expr $(, $tail:expr)* ) => {
        {
            let mut base_path = ident_path($base);
            base_path.kind = PathKind::Dep;
            $(
                base_path.segments.push(ident($tail));
            )*
            base_path
        }
    }
}

fn cast(lhs: NoirExpression, ty: UnresolvedTypeData) -> NoirExpression {
    expression(ExpressionKind::Cast(Box::new(CastExpression { lhs, r#type: make_type(ty) })))
}

fn make_type(typ: UnresolvedTypeData) -> UnresolvedType {
    UnresolvedType { typ, span: None }
}

fn index_array(array: NoirIdent, index: &str) -> NoirExpression {
    expression(ExpressionKind::Index(Box::new(IndexExpression {
        collection: variable_path(path(array)),
        index: variable(index),
    })))
}

fn index_array_variable(array: NoirExpression, index: &str) -> NoirExpression {
    expression(ExpressionKind::Index(Box::new(IndexExpression {
        collection: array,
        index: variable(index),
    })))
}
