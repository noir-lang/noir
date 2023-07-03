use acvm::FieldElement;
use fm::FileId;
use noirc_errors::Location;

use crate::node_interner::{DefinitionId, ExprId, FuncId, NodeInterner, StmtId};
use crate::{BinaryOp, BinaryOpKind, Ident, Shared, UnaryOp};

use super::stmt::HirPattern;
use super::types::{StructType, Type};

/// A HirExpression is the result of an Expression in the AST undergoing
/// name resolution. It is almost identical to the Expression AST node, but
/// references other HIR nodes indirectly via IDs rather than directly via
/// boxing. Variables in HirExpressions are tagged with their DefinitionId
/// from the definition that refers to them so there is no ambiguity with names.
#[derive(Debug, Clone)]
pub enum HirExpression {
    Ident(HirIdent),
    Literal(HirLiteral),
    Block(HirBlockExpression),
    Prefix(HirPrefixExpression),
    Infix(HirInfixExpression),
    Index(HirIndexExpression),
    Constructor(HirConstructorExpression),
    MemberAccess(HirMemberAccess),
    Call(HirCallExpression),
    MethodCall(HirMethodCallExpression),
    Cast(HirCastExpression),
    For(HirForExpression),
    If(HirIfExpression),
    Tuple(Vec<ExprId>),
    Lambda(HirLambda),
    Error,
}

impl HirExpression {
    /// Returns an empty block expression
    pub const fn empty_block() -> HirExpression {
        HirExpression::Block(HirBlockExpression(vec![]))
    }
}

/// Corresponds to a variable in the source code
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HirIdent {
    pub location: Location,
    pub id: DefinitionId,
}

#[derive(Debug, Clone)]
pub struct HirForExpression {
    pub identifier: HirIdent,
    pub start_range: ExprId,
    pub end_range: ExprId,
    pub block: ExprId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HirBinaryOp {
    pub kind: BinaryOpKind,
    pub location: Location,
}

impl HirBinaryOp {
    pub fn new(op: BinaryOp, file: FileId) -> Self {
        let kind = op.contents;
        let location = Location::new(op.span(), file);
        HirBinaryOp { location, kind }
    }

    pub fn is_bitwise(&self) -> bool {
        use BinaryOpKind::*;
        matches!(self.kind, And | Or | Xor | ShiftRight | ShiftLeft)
    }
}

#[derive(Debug, Clone)]
pub enum HirLiteral {
    Array(HirArrayLiteral),
    Bool(bool),
    Integer(FieldElement),
    Str(String),
}

#[derive(Debug, Clone)]
pub enum HirArrayLiteral {
    Standard(Vec<ExprId>),
    Repeated { repeated_element: ExprId, length: Type },
}

#[derive(Debug, Clone)]
pub struct HirPrefixExpression {
    pub operator: UnaryOp,
    pub rhs: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirInfixExpression {
    pub lhs: ExprId,
    pub operator: HirBinaryOp,
    pub rhs: ExprId,
}

/// This is always a struct field access `mystruct.field`
/// and never a method call. The later is represented by HirMethodCallExpression.
#[derive(Debug, Clone)]
pub struct HirMemberAccess {
    pub lhs: ExprId,
    // This field is not an IdentId since the rhs of a field
    // access has no corresponding definition
    pub rhs: Ident,
}

#[derive(Debug, Clone)]
pub struct HirIfExpression {
    pub condition: ExprId,
    pub consequence: ExprId,
    pub alternative: Option<ExprId>,
}

// `lhs as type` in the source code
#[derive(Debug, Clone)]
pub struct HirCastExpression {
    pub lhs: ExprId,
    pub r#type: Type,
}

#[derive(Debug, Clone)]
pub struct HirCallExpression {
    pub func: ExprId,
    pub arguments: Vec<ExprId>,
    pub location: Location,
}

/// These nodes are temporary, they're
/// lowered into HirCallExpression nodes
/// after type checking resolves the object
/// type and the method it calls.
#[derive(Debug, Clone)]
pub struct HirMethodCallExpression {
    pub method: Ident,
    pub object: ExprId,
    pub arguments: Vec<ExprId>,
    pub location: Location,
}

impl HirMethodCallExpression {
    pub fn into_function_call(
        mut self,
        func: FuncId,
        location: Location,
        argument_types: &mut [(Type, noirc_errors::Span)],
        interner: &mut NodeInterner,
    ) -> (ExprId, HirExpression) {
        // Automatically add `&mut` if the method expects a mutable reference and
        // the object is not already one.
        if func != FuncId::dummy_id() {
            let func_meta = interner.function_meta(&func);
            self.try_add_mutable_reference_to_object(&func_meta.typ, argument_types, interner);
        }

        let mut arguments = vec![self.object];
        arguments.append(&mut self.arguments);

        let id = interner.function_definition_id(func);
        let ident = HirExpression::Ident(HirIdent { location, id });
        let func = interner.push_expr(ident);

        (func, HirExpression::Call(HirCallExpression { func, arguments, location }))
    }

    /// Check if the given function type requires a mutable reference to the object type, and check
    /// if the given object type is already a mutable reference. If not, add one.
    fn try_add_mutable_reference_to_object(
        &mut self,
        function_type: &Type,
        argument_types: &mut [(Type, noirc_errors::Span)],
        interner: &mut NodeInterner,
    ) {
        let expected_object_type = match function_type {
            Type::Function(args, _) => args.get(0),
            Type::Forall(_, typ) => match typ.as_ref() {
                Type::Function(args, _) => args.get(0),
                typ => unreachable!("Unexpected type for function: {typ}"),
            },
            typ => unreachable!("Unexpected type for function: {typ}"),
        };

        if let Some(expected_object_type) = expected_object_type {
            if matches!(expected_object_type.follow_bindings(), Type::MutableReference(_)) {
                let actual_type = argument_types[0].0.follow_bindings();
                if !matches!(actual_type, Type::MutableReference(_)) {
                    let new_type = Type::MutableReference(Box::new(actual_type));

                    argument_types[0].0 = new_type.clone();
                    self.object = interner.push_expr(HirExpression::Prefix(HirPrefixExpression {
                        operator: UnaryOp::MutableReference,
                        rhs: self.object,
                    }));
                    interner.push_expr_type(&self.object, new_type);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirConstructorExpression {
    pub r#type: Shared<StructType>,
    pub struct_generics: Vec<Type>,

    // NOTE: It is tempting to make this a BTreeSet to force ordering of field
    //       names (and thus remove the need to normalize them during type checking)
    //       but doing so would force the order of evaluation of field
    //       arguments to be alphabetical rather than the ordering the user
    //       included in the source code.
    pub fields: Vec<(Ident, ExprId)>,
}

/// Indexing, as in `array[index]`
#[derive(Debug, Clone)]
pub struct HirIndexExpression {
    pub collection: ExprId,
    pub index: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirBlockExpression(pub Vec<StmtId>);

impl HirBlockExpression {
    pub fn statements(&self) -> &[StmtId] {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct HirLambda {
    pub parameters: Vec<(HirPattern, Type)>,
    pub return_type: Type,
    pub body: ExprId,
}
