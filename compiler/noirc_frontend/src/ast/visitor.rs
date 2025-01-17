use acvm::FieldElement;
use noirc_errors::Span;

use crate::{
    ast::{
        ArrayLiteral, AsTraitPath, AssignStatement, BlockExpression, CallExpression,
        CastExpression, ConstrainStatement, ConstructorExpression, Expression, ExpressionKind,
        ForLoopStatement, ForRange, Ident, IfExpression, IndexExpression, InfixExpression, LValue,
        Lambda, LetStatement, Literal, MemberAccessExpression, MethodCallExpression,
        ModuleDeclaration, NoirFunction, NoirStruct, NoirTrait, NoirTraitImpl, NoirTypeAlias, Path,
        PrefixExpression, Statement, StatementKind, TraitImplItem, TraitItem, TypeImpl, UseTree,
        UseTreeKind,
    },
    node_interner::{
        ExprId, InternedExpressionKind, InternedPattern, InternedStatementKind,
        InternedUnresolvedTypeData, QuotedTypeId,
    },
    parser::{Item, ItemKind, ParsedSubModule},
    token::{FmtStrFragment, MetaAttribute, SecondaryAttribute, Tokens},
    ParsedModule, QuotedType,
};

use super::{
    ForBounds, FunctionReturnType, GenericTypeArgs, IntegerBitSize, ItemVisibility, Pattern,
    Signedness, TraitBound, TraitImplItemKind, TypePath, UnresolvedGenerics,
    UnresolvedTraitConstraint, UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AttributeTarget {
    Module,
    Struct,
    Trait,
    Function,
    Let,
}

/// Implements the [Visitor pattern](https://en.wikipedia.org/wiki/Visitor_pattern) for Noir's AST.
///
/// In this implementation, methods must return a bool:
/// - true means children must be visited
/// - false means children must not be visited, either because the visitor implementation
///   will visit children of interest manually, or because no children are of interest
pub trait Visitor {
    fn visit_parsed_module(&mut self, _: &ParsedModule) -> bool {
        true
    }

    fn visit_item(&mut self, _: &Item) -> bool {
        true
    }

    fn visit_parsed_submodule(&mut self, _: &ParsedSubModule, _: Span) -> bool {
        true
    }

    fn visit_noir_function(&mut self, _: &NoirFunction, _: Span) -> bool {
        true
    }

    fn visit_noir_trait_impl(&mut self, _: &NoirTraitImpl, _: Span) -> bool {
        true
    }

    fn visit_type_impl(&mut self, _: &TypeImpl, _: Span) -> bool {
        true
    }

    fn visit_trait_impl_item(&mut self, _: &TraitImplItem) -> bool {
        true
    }

    fn visit_trait_impl_item_kind(&mut self, _: &TraitImplItemKind, _span: Span) -> bool {
        true
    }

    fn visit_trait_impl_item_function(&mut self, _: &NoirFunction, _span: Span) -> bool {
        true
    }

    fn visit_trait_impl_item_constant(
        &mut self,
        _name: &Ident,
        _typ: &UnresolvedType,
        _expression: &Expression,
        _span: Span,
    ) -> bool {
        true
    }

    fn visit_trait_impl_item_type(
        &mut self,
        _name: &Ident,
        _alias: &UnresolvedType,
        _span: Span,
    ) -> bool {
        true
    }

    fn visit_noir_trait(&mut self, _: &NoirTrait, _: Span) -> bool {
        true
    }

    fn visit_trait_item(&mut self, _: &TraitItem) -> bool {
        true
    }

    fn visit_trait_item_function(
        &mut self,
        _name: &Ident,
        _generics: &UnresolvedGenerics,
        _parameters: &[(Ident, UnresolvedType)],
        _return_type: &FunctionReturnType,
        _where_clause: &[UnresolvedTraitConstraint],
        _body: &Option<BlockExpression>,
    ) -> bool {
        true
    }

    fn visit_trait_item_constant(
        &mut self,
        _name: &Ident,
        _typ: &UnresolvedType,
        _default_value: &Option<Expression>,
    ) -> bool {
        true
    }

    fn visit_trait_item_type(&mut self, _: &Ident) {}

    fn visit_use_tree(&mut self, _: &UseTree) -> bool {
        true
    }

    fn visit_use_tree_path(&mut self, _: &UseTree, _ident: &Ident, _alias: &Option<Ident>) {}

    fn visit_use_tree_list(&mut self, _: &UseTree, _: &[UseTree]) -> bool {
        true
    }

    fn visit_noir_struct(&mut self, _: &NoirStruct, _: Span) -> bool {
        true
    }

    fn visit_noir_type_alias(&mut self, _: &NoirTypeAlias, _: Span) -> bool {
        true
    }

    fn visit_module_declaration(&mut self, _: &ModuleDeclaration, _: Span) {}

    fn visit_expression(&mut self, _: &Expression) -> bool {
        true
    }

    fn visit_literal(&mut self, _: &Literal, _: Span) -> bool {
        true
    }

    fn visit_literal_array(&mut self, _: &ArrayLiteral) -> bool {
        true
    }

    fn visit_literal_slice(&mut self, _: &ArrayLiteral) -> bool {
        true
    }

    fn visit_literal_bool(&mut self, _: bool) {}

    fn visit_literal_integer(&mut self, _value: FieldElement, _negative: bool) {}

    fn visit_literal_str(&mut self, _: &str) {}

    fn visit_literal_raw_str(&mut self, _: &str, _: u8) {}

    fn visit_literal_fmt_str(&mut self, _: &[FmtStrFragment], _length: u32) {}

    fn visit_literal_unit(&mut self) {}

    fn visit_block_expression(&mut self, _: &BlockExpression, _: Option<Span>) -> bool {
        true
    }

    fn visit_prefix_expression(&mut self, _: &PrefixExpression, _: Span) -> bool {
        true
    }

    fn visit_index_expression(&mut self, _: &IndexExpression, _: Span) -> bool {
        true
    }

    fn visit_call_expression(&mut self, _: &CallExpression, _: Span) -> bool {
        true
    }

    fn visit_method_call_expression(&mut self, _: &MethodCallExpression, _: Span) -> bool {
        true
    }

    fn visit_constructor_expression(&mut self, _: &ConstructorExpression, _: Span) -> bool {
        true
    }

    fn visit_member_access_expression(&mut self, _: &MemberAccessExpression, _: Span) -> bool {
        true
    }

    fn visit_cast_expression(&mut self, _: &CastExpression, _: Span) -> bool {
        true
    }

    fn visit_infix_expression(&mut self, _: &InfixExpression, _: Span) -> bool {
        true
    }

    fn visit_if_expression(&mut self, _: &IfExpression, _: Span) -> bool {
        true
    }

    fn visit_tuple(&mut self, _: &[Expression], _: Span) -> bool {
        true
    }

    fn visit_parenthesized(&mut self, _: &Expression, _: Span) -> bool {
        true
    }

    fn visit_unquote(&mut self, _: &Expression, _: Span) -> bool {
        true
    }

    fn visit_comptime_expression(&mut self, _: &BlockExpression, _: Span) -> bool {
        true
    }

    fn visit_unsafe(&mut self, _: &BlockExpression, _: Span) -> bool {
        true
    }

    fn visit_variable(&mut self, _: &Path, _: Span) -> bool {
        true
    }

    fn visit_quote(&mut self, _: &Tokens) {}

    fn visit_resolved_expression(&mut self, _expr_id: ExprId) {}

    fn visit_interned_expression(&mut self, _id: InternedExpressionKind) {}

    fn visit_error_expression(&mut self) {}

    fn visit_lambda(&mut self, _: &Lambda, _: Span) -> bool {
        true
    }

    fn visit_array_literal(&mut self, _: &ArrayLiteral) -> bool {
        true
    }

    fn visit_array_literal_standard(&mut self, _: &[Expression]) -> bool {
        true
    }

    fn visit_array_literal_repeated(
        &mut self,
        _repeated_element: &Expression,
        _length: &Expression,
    ) -> bool {
        true
    }

    fn visit_statement(&mut self, _: &Statement) -> bool {
        true
    }

    fn visit_import(&mut self, _: &UseTree, _: Span, _visibility: ItemVisibility) -> bool {
        true
    }

    fn visit_global(&mut self, _: &LetStatement, _: Span) -> bool {
        true
    }

    fn visit_let_statement(&mut self, _: &LetStatement) -> bool {
        true
    }

    fn visit_constrain_statement(&mut self, _: &ConstrainStatement) -> bool {
        true
    }

    fn visit_assign_statement(&mut self, _: &AssignStatement) -> bool {
        true
    }

    fn visit_for_loop_statement(&mut self, _: &ForLoopStatement) -> bool {
        true
    }

    fn visit_loop_statement(&mut self, _: &Expression) -> bool {
        true
    }

    fn visit_comptime_statement(&mut self, _: &Statement) -> bool {
        true
    }

    fn visit_break(&mut self) {}

    fn visit_continue(&mut self) {}

    fn visit_interned_statement(&mut self, _: InternedStatementKind) {}

    fn visit_error_statement(&mut self) {}

    fn visit_lvalue(&mut self, _: &LValue) -> bool {
        true
    }

    fn visit_lvalue_ident(&mut self, _: &Ident) {}

    fn visit_lvalue_member_access(
        &mut self,
        _object: &LValue,
        _field_name: &Ident,
        _span: Span,
    ) -> bool {
        true
    }

    fn visit_lvalue_index(&mut self, _array: &LValue, _index: &Expression, _span: Span) -> bool {
        true
    }

    fn visit_lvalue_dereference(&mut self, _lvalue: &LValue, _span: Span) -> bool {
        true
    }

    fn visit_lvalue_interned(&mut self, _id: InternedExpressionKind, _span: Span) {}

    fn visit_for_range(&mut self, _: &ForRange) -> bool {
        true
    }

    fn visit_as_trait_path(&mut self, _: &AsTraitPath, _: Span) -> bool {
        true
    }

    fn visit_type_path(&mut self, _: &TypePath, _: Span) -> bool {
        true
    }

    fn visit_unresolved_type(&mut self, _: &UnresolvedType) -> bool {
        true
    }

    fn visit_array_type(
        &mut self,
        _: &UnresolvedTypeExpression,
        _: &UnresolvedType,
        _: Span,
    ) -> bool {
        true
    }

    fn visit_slice_type(&mut self, _: &UnresolvedType, _: Span) -> bool {
        true
    }

    fn visit_parenthesized_type(&mut self, _: &UnresolvedType, _: Span) -> bool {
        true
    }

    fn visit_named_type(&mut self, _: &Path, _: &GenericTypeArgs, _: Span) -> bool {
        true
    }

    fn visit_trait_as_type(&mut self, _: &Path, _: &GenericTypeArgs, _: Span) -> bool {
        true
    }

    fn visit_mutable_reference_type(&mut self, _: &UnresolvedType, _: Span) -> bool {
        true
    }

    fn visit_tuple_type(&mut self, _: &[UnresolvedType], _: Span) -> bool {
        true
    }

    fn visit_function_type(
        &mut self,
        _args: &[UnresolvedType],
        _ret: &UnresolvedType,
        _env: &UnresolvedType,
        _unconstrained: bool,
        _span: Span,
    ) -> bool {
        true
    }

    fn visit_as_trait_path_type(&mut self, _: &AsTraitPath, _: Span) -> bool {
        true
    }

    fn visit_expression_type(&mut self, _: &UnresolvedTypeExpression, _: Span) {}

    fn visit_format_string_type(
        &mut self,
        _: &UnresolvedTypeExpression,
        _: &UnresolvedType,
        _: Span,
    ) -> bool {
        true
    }

    fn visit_string_type(&mut self, _: &UnresolvedTypeExpression, _: Span) {}

    fn visit_unspecified_type(&mut self, _: Span) {}

    fn visit_quoted_type(&mut self, _: &QuotedType, _: Span) {}

    fn visit_field_element_type(&mut self, _: Span) {}

    fn visit_integer_type(&mut self, _: Signedness, _: IntegerBitSize, _: Span) {}

    fn visit_bool_type(&mut self, _: Span) {}

    fn visit_unit_type(&mut self, _: Span) {}

    fn visit_resolved_type(&mut self, _: QuotedTypeId, _: Span) {}

    fn visit_interned_type(&mut self, _: InternedUnresolvedTypeData, _: Span) {}

    fn visit_error_type(&mut self, _: Span) {}

    fn visit_path(&mut self, _: &Path) {}

    fn visit_generic_type_args(&mut self, _: &GenericTypeArgs) -> bool {
        true
    }

    fn visit_function_return_type(&mut self, _: &FunctionReturnType) -> bool {
        true
    }

    fn visit_trait_bound(&mut self, _: &TraitBound) -> bool {
        true
    }

    fn visit_unresolved_trait_constraint(&mut self, _: &UnresolvedTraitConstraint) -> bool {
        true
    }

    fn visit_pattern(&mut self, _: &Pattern) -> bool {
        true
    }

    fn visit_identifier_pattern(&mut self, _: &Ident) {}

    fn visit_mutable_pattern(&mut self, _: &Pattern, _: Span, _is_synthesized: bool) -> bool {
        true
    }

    fn visit_tuple_pattern(&mut self, _: &[Pattern], _: Span) -> bool {
        true
    }

    fn visit_struct_pattern(&mut self, _: &Path, _: &[(Ident, Pattern)], _: Span) -> bool {
        true
    }

    fn visit_interned_pattern(&mut self, _: &InternedPattern, _: Span) {}

    fn visit_secondary_attribute(
        &mut self,
        _: &SecondaryAttribute,
        _target: AttributeTarget,
    ) -> bool {
        true
    }

    fn visit_meta_attribute(&mut self, _: &MetaAttribute, _target: AttributeTarget) -> bool {
        true
    }
}

impl ParsedModule {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_parsed_module(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        for item in &self.items {
            item.accept(visitor);
        }
    }
}

impl Item {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_item(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match &self.kind {
            ItemKind::Submodules(parsed_sub_module) => {
                parsed_sub_module.accept(self.span, visitor);
            }
            ItemKind::Function(noir_function) => noir_function.accept(self.span, visitor),
            ItemKind::TraitImpl(noir_trait_impl) => {
                noir_trait_impl.accept(self.span, visitor);
            }
            ItemKind::Impl(type_impl) => type_impl.accept(self.span, visitor),
            ItemKind::Global(let_statement, _visibility) => {
                if visitor.visit_global(let_statement, self.span) {
                    let_statement.accept(visitor);
                }
            }
            ItemKind::Trait(noir_trait) => noir_trait.accept(self.span, visitor),
            ItemKind::Import(use_tree, visibility) => {
                if visitor.visit_import(use_tree, self.span, *visibility) {
                    use_tree.accept(visitor);
                }
            }
            ItemKind::TypeAlias(noir_type_alias) => noir_type_alias.accept(self.span, visitor),
            ItemKind::Struct(noir_struct) => noir_struct.accept(self.span, visitor),
            ItemKind::ModuleDecl(module_declaration) => {
                module_declaration.accept(self.span, visitor);
            }
            ItemKind::InnerAttribute(attribute) => {
                attribute.accept(AttributeTarget::Module, visitor);
            }
        }
    }
}

impl ParsedSubModule {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        for attribute in &self.outer_attributes {
            attribute.accept(AttributeTarget::Module, visitor);
        }

        if visitor.visit_parsed_submodule(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.contents.accept(visitor);
    }
}

impl NoirFunction {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_noir_function(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        for attribute in self.secondary_attributes() {
            attribute.accept(AttributeTarget::Function, visitor);
        }

        for param in &self.def.parameters {
            param.typ.accept(visitor);
        }

        self.def.return_type.accept(visitor);

        for constraint in &self.def.where_clause {
            constraint.accept(visitor);
        }

        self.def.body.accept(None, visitor);
    }
}

impl NoirTraitImpl {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_noir_trait_impl(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.trait_name.accept(visitor);
        self.object_type.accept(visitor);

        for item in &self.items {
            item.item.accept(visitor);
        }
    }
}

impl TraitImplItem {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_trait_impl_item(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.kind.accept(self.span, visitor);
    }
}

impl TraitImplItemKind {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_trait_impl_item_kind(self, span) {
            self.accept_children(span, visitor);
        }
    }

    pub fn accept_children(&self, span: Span, visitor: &mut impl Visitor) {
        match self {
            TraitImplItemKind::Function(noir_function) => {
                if visitor.visit_trait_impl_item_function(noir_function, span) {
                    noir_function.accept(span, visitor);
                }
            }
            TraitImplItemKind::Constant(name, unresolved_type, expression) => {
                if visitor.visit_trait_impl_item_constant(name, unresolved_type, expression, span) {
                    unresolved_type.accept(visitor);
                    expression.accept(visitor);
                }
            }
            TraitImplItemKind::Type { name, alias } => {
                if visitor.visit_trait_impl_item_type(name, alias, span) {
                    alias.accept(visitor);
                }
            }
        }
    }
}

impl TypeImpl {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_type_impl(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.object_type.accept(visitor);

        for (method, span) in &self.methods {
            method.item.accept(*span, visitor);
        }
    }
}

impl NoirTrait {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_noir_trait(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        for attribute in &self.attributes {
            attribute.accept(AttributeTarget::Trait, visitor);
        }

        for bound in &self.bounds {
            bound.accept(visitor);
        }

        for constraint in &self.where_clause {
            constraint.accept(visitor);
        }

        for item in &self.items {
            item.item.accept(visitor);
        }
    }
}

impl TraitItem {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_trait_item(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match self {
            TraitItem::Function {
                name,
                generics,
                parameters,
                return_type,
                where_clause,
                body,
                is_unconstrained: _,
                visibility: _,
                is_comptime: _,
            } => {
                if visitor.visit_trait_item_function(
                    name,
                    generics,
                    parameters,
                    return_type,
                    where_clause,
                    body,
                ) {
                    for (_name, unresolved_type) in parameters {
                        unresolved_type.accept(visitor);
                    }

                    return_type.accept(visitor);

                    for unresolved_trait_constraint in where_clause {
                        unresolved_trait_constraint.accept(visitor);
                    }

                    if let Some(body) = body {
                        body.accept(None, visitor);
                    }
                }
            }
            TraitItem::Constant { name, typ, default_value } => {
                if visitor.visit_trait_item_constant(name, typ, default_value) {
                    typ.accept(visitor);

                    if let Some(default_value) = default_value {
                        default_value.accept(visitor);
                    }
                }
            }
            TraitItem::Type { name } => visitor.visit_trait_item_type(name),
        }
    }
}

impl UseTree {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_use_tree(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match &self.kind {
            UseTreeKind::Path(ident, alias) => visitor.visit_use_tree_path(self, ident, alias),
            UseTreeKind::List(use_trees) => {
                if visitor.visit_use_tree_list(self, use_trees) {
                    for use_tree in use_trees {
                        use_tree.accept(visitor);
                    }
                }
            }
        }
    }
}

impl NoirStruct {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_noir_struct(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        for attribute in &self.attributes {
            attribute.accept(AttributeTarget::Struct, visitor);
        }

        for field in &self.fields {
            field.item.typ.accept(visitor);
        }
    }
}

impl NoirTypeAlias {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_noir_type_alias(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.typ.accept(visitor);
    }
}

impl ModuleDeclaration {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        for attribute in &self.outer_attributes {
            attribute.accept(AttributeTarget::Module, visitor);
        }

        visitor.visit_module_declaration(self, span);
    }
}

impl Expression {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_expression(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match &self.kind {
            ExpressionKind::Literal(literal) => literal.accept(self.span, visitor),
            ExpressionKind::Block(block_expression) => {
                block_expression.accept(Some(self.span), visitor);
            }
            ExpressionKind::Prefix(prefix_expression) => {
                prefix_expression.accept(self.span, visitor);
            }
            ExpressionKind::Index(index_expression) => {
                index_expression.accept(self.span, visitor);
            }
            ExpressionKind::Call(call_expression) => {
                call_expression.accept(self.span, visitor);
            }
            ExpressionKind::MethodCall(method_call_expression) => {
                method_call_expression.accept(self.span, visitor);
            }
            ExpressionKind::Constructor(constructor_expression) => {
                constructor_expression.accept(self.span, visitor);
            }
            ExpressionKind::MemberAccess(member_access_expression) => {
                member_access_expression.accept(self.span, visitor);
            }
            ExpressionKind::Cast(cast_expression) => {
                cast_expression.accept(self.span, visitor);
            }
            ExpressionKind::Infix(infix_expression) => {
                infix_expression.accept(self.span, visitor);
            }
            ExpressionKind::If(if_expression) => {
                if_expression.accept(self.span, visitor);
            }
            ExpressionKind::Tuple(expressions) => {
                if visitor.visit_tuple(expressions, self.span) {
                    visit_expressions(expressions, visitor);
                }
            }
            ExpressionKind::Lambda(lambda) => lambda.accept(self.span, visitor),
            ExpressionKind::Parenthesized(expression) => {
                if visitor.visit_parenthesized(expression, self.span) {
                    expression.accept(visitor);
                }
            }
            ExpressionKind::Unquote(expression) => {
                if visitor.visit_unquote(expression, self.span) {
                    expression.accept(visitor);
                }
            }
            ExpressionKind::Comptime(block_expression, _) => {
                if visitor.visit_comptime_expression(block_expression, self.span) {
                    block_expression.accept(None, visitor);
                }
            }
            ExpressionKind::Unsafe(block_expression, _) => {
                if visitor.visit_unsafe(block_expression, self.span) {
                    block_expression.accept(None, visitor);
                }
            }
            ExpressionKind::Variable(path) => {
                if visitor.visit_variable(path, self.span) {
                    path.accept(visitor);
                }
            }
            ExpressionKind::AsTraitPath(as_trait_path) => {
                as_trait_path.accept(self.span, visitor);
            }
            ExpressionKind::TypePath(path) => path.accept(self.span, visitor),
            ExpressionKind::Quote(tokens) => visitor.visit_quote(tokens),
            ExpressionKind::Resolved(expr_id) => visitor.visit_resolved_expression(*expr_id),
            ExpressionKind::Interned(id) => visitor.visit_interned_expression(*id),
            ExpressionKind::InternedStatement(id) => visitor.visit_interned_statement(*id),
            ExpressionKind::Error => visitor.visit_error_expression(),
        }
    }
}

impl Literal {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_literal(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match self {
            Literal::Array(array_literal) => {
                if visitor.visit_literal_array(array_literal) {
                    array_literal.accept(visitor);
                }
            }
            Literal::Slice(array_literal) => {
                if visitor.visit_literal_slice(array_literal) {
                    array_literal.accept(visitor);
                }
            }
            Literal::Bool(value) => visitor.visit_literal_bool(*value),
            Literal::Integer(value, negative) => visitor.visit_literal_integer(*value, *negative),
            Literal::Str(str) => visitor.visit_literal_str(str),
            Literal::RawStr(str, length) => visitor.visit_literal_raw_str(str, *length),
            Literal::FmtStr(fragments, length) => visitor.visit_literal_fmt_str(fragments, *length),
            Literal::Unit => visitor.visit_literal_unit(),
        }
    }
}

impl BlockExpression {
    pub fn accept(&self, span: Option<Span>, visitor: &mut impl Visitor) {
        if visitor.visit_block_expression(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        for statement in &self.statements {
            statement.accept(visitor);
        }
    }
}

impl PrefixExpression {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_prefix_expression(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.rhs.accept(visitor);
    }
}

impl IndexExpression {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_index_expression(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.collection.accept(visitor);
        self.index.accept(visitor);
    }
}

impl CallExpression {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_call_expression(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.func.accept(visitor);
        visit_expressions(&self.arguments, visitor);
    }
}

impl MethodCallExpression {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_method_call_expression(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.object.accept(visitor);
        visit_expressions(&self.arguments, visitor);
    }
}

impl ConstructorExpression {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_constructor_expression(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.typ.accept(visitor);

        for (_field_name, expression) in &self.fields {
            expression.accept(visitor);
        }
    }
}

impl MemberAccessExpression {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_member_access_expression(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.lhs.accept(visitor);
    }
}

impl CastExpression {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_cast_expression(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.lhs.accept(visitor);
    }
}

impl InfixExpression {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_infix_expression(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.lhs.accept(visitor);
        self.rhs.accept(visitor);
    }
}

impl IfExpression {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_if_expression(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.condition.accept(visitor);
        self.consequence.accept(visitor);
        if let Some(alternative) = &self.alternative {
            alternative.accept(visitor);
        }
    }
}

impl Lambda {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_lambda(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        for (_, unresolved_type) in &self.parameters {
            unresolved_type.accept(visitor);
        }

        self.body.accept(visitor);
    }
}

impl ArrayLiteral {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_array_literal(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match self {
            ArrayLiteral::Standard(expressions) => {
                if visitor.visit_array_literal_standard(expressions) {
                    visit_expressions(expressions, visitor);
                }
            }
            ArrayLiteral::Repeated { repeated_element, length } => {
                if visitor.visit_array_literal_repeated(repeated_element, length) {
                    repeated_element.accept(visitor);
                    length.accept(visitor);
                }
            }
        }
    }
}

impl Statement {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_statement(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match &self.kind {
            StatementKind::Let(let_statement) => {
                let_statement.accept(visitor);
            }
            StatementKind::Constrain(constrain_statement) => {
                constrain_statement.accept(visitor);
            }
            StatementKind::Expression(expression) => {
                expression.accept(visitor);
            }
            StatementKind::Assign(assign_statement) => {
                assign_statement.accept(visitor);
            }
            StatementKind::For(for_loop_statement) => {
                for_loop_statement.accept(visitor);
            }
            StatementKind::Loop(block) => {
                if visitor.visit_loop_statement(block) {
                    block.accept(visitor);
                }
            }
            StatementKind::Comptime(statement) => {
                if visitor.visit_comptime_statement(statement) {
                    statement.accept(visitor);
                }
            }
            StatementKind::Semi(expression) => {
                expression.accept(visitor);
            }
            StatementKind::Break => visitor.visit_break(),
            StatementKind::Continue => visitor.visit_continue(),
            StatementKind::Interned(id) => visitor.visit_interned_statement(*id),
            StatementKind::Error => visitor.visit_error_statement(),
        }
    }
}

impl LetStatement {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        for attribute in &self.attributes {
            attribute.accept(AttributeTarget::Let, visitor);
        }

        if visitor.visit_let_statement(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.pattern.accept(visitor);
        self.r#type.accept(visitor);
        self.expression.accept(visitor);
    }
}

impl ConstrainStatement {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_constrain_statement(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        visit_expressions(&self.arguments, visitor);
    }
}

impl AssignStatement {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_assign_statement(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.lvalue.accept(visitor);
        self.expression.accept(visitor);
    }
}

impl ForLoopStatement {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_for_loop_statement(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.range.accept(visitor);
        self.block.accept(visitor);
    }
}

impl LValue {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_lvalue(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match self {
            LValue::Ident(ident) => visitor.visit_lvalue_ident(ident),
            LValue::MemberAccess { object, field_name, span } => {
                if visitor.visit_lvalue_member_access(object, field_name, *span) {
                    object.accept(visitor);
                }
            }
            LValue::Index { array, index, span } => {
                if visitor.visit_lvalue_index(array, index, *span) {
                    array.accept(visitor);
                    index.accept(visitor);
                }
            }
            LValue::Dereference(lvalue, span) => {
                if visitor.visit_lvalue_dereference(lvalue, *span) {
                    lvalue.accept(visitor);
                }
            }
            LValue::Interned(id, span) => visitor.visit_lvalue_interned(*id, *span),
        }
    }
}

impl ForRange {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_for_range(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match self {
            ForRange::Range(ForBounds { start, end, inclusive: _ }) => {
                start.accept(visitor);
                end.accept(visitor);
            }
            ForRange::Array(expression) => expression.accept(visitor),
        }
    }
}

impl AsTraitPath {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_as_trait_path(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.trait_path.accept(visitor);
        self.trait_generics.accept(visitor);
    }
}

impl TypePath {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_type_path(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.typ.accept(visitor);
        if let Some(turbofish) = &self.turbofish {
            turbofish.accept(visitor);
        }
    }
}

impl UnresolvedType {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_unresolved_type(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match &self.typ {
            UnresolvedTypeData::Array(unresolved_type_expression, unresolved_type) => {
                if visitor.visit_array_type(unresolved_type_expression, unresolved_type, self.span)
                {
                    unresolved_type.accept(visitor);
                }
            }
            UnresolvedTypeData::Slice(unresolved_type) => {
                if visitor.visit_slice_type(unresolved_type, self.span) {
                    unresolved_type.accept(visitor);
                }
            }
            UnresolvedTypeData::Parenthesized(unresolved_type) => {
                if visitor.visit_parenthesized_type(unresolved_type, self.span) {
                    unresolved_type.accept(visitor);
                }
            }
            UnresolvedTypeData::Named(path, generic_type_args, _) => {
                if visitor.visit_named_type(path, generic_type_args, self.span) {
                    path.accept(visitor);
                    generic_type_args.accept(visitor);
                }
            }
            UnresolvedTypeData::TraitAsType(path, generic_type_args) => {
                if visitor.visit_trait_as_type(path, generic_type_args, self.span) {
                    path.accept(visitor);
                    generic_type_args.accept(visitor);
                }
            }
            UnresolvedTypeData::MutableReference(unresolved_type) => {
                if visitor.visit_mutable_reference_type(unresolved_type, self.span) {
                    unresolved_type.accept(visitor);
                }
            }
            UnresolvedTypeData::Tuple(unresolved_types) => {
                if visitor.visit_tuple_type(unresolved_types, self.span) {
                    visit_unresolved_types(unresolved_types, visitor);
                }
            }
            UnresolvedTypeData::Function(args, ret, env, unconstrained) => {
                if visitor.visit_function_type(args, ret, env, *unconstrained, self.span) {
                    visit_unresolved_types(args, visitor);
                    ret.accept(visitor);
                    env.accept(visitor);
                }
            }
            UnresolvedTypeData::AsTraitPath(as_trait_path) => {
                if visitor.visit_as_trait_path_type(as_trait_path, self.span) {
                    as_trait_path.accept(self.span, visitor);
                }
            }
            UnresolvedTypeData::Expression(expr) => visitor.visit_expression_type(expr, self.span),
            UnresolvedTypeData::FormatString(expr, typ) => {
                if visitor.visit_format_string_type(expr, typ, self.span) {
                    typ.accept(visitor);
                }
            }
            UnresolvedTypeData::String(expr) => visitor.visit_string_type(expr, self.span),
            UnresolvedTypeData::Unspecified => visitor.visit_unspecified_type(self.span),
            UnresolvedTypeData::Quoted(typ) => visitor.visit_quoted_type(typ, self.span),
            UnresolvedTypeData::FieldElement => visitor.visit_field_element_type(self.span),
            UnresolvedTypeData::Integer(signdness, size) => {
                visitor.visit_integer_type(*signdness, *size, self.span);
            }
            UnresolvedTypeData::Bool => visitor.visit_bool_type(self.span),
            UnresolvedTypeData::Unit => visitor.visit_unit_type(self.span),
            UnresolvedTypeData::Resolved(id) => visitor.visit_resolved_type(*id, self.span),
            UnresolvedTypeData::Interned(id) => visitor.visit_interned_type(*id, self.span),
            UnresolvedTypeData::Error => visitor.visit_error_type(self.span),
        }
    }
}

impl Path {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        visitor.visit_path(self);
    }
}

impl GenericTypeArgs {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_generic_type_args(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        visit_unresolved_types(&self.ordered_args, visitor);
        for (_name, typ) in &self.named_args {
            typ.accept(visitor);
        }
    }
}

impl FunctionReturnType {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_function_return_type(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match self {
            FunctionReturnType::Default(_) => (),
            FunctionReturnType::Ty(unresolved_type) => {
                unresolved_type.accept(visitor);
            }
        }
    }
}

impl TraitBound {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_trait_bound(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.trait_path.accept(visitor);
        self.trait_generics.accept(visitor);
    }
}

impl UnresolvedTraitConstraint {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_unresolved_trait_constraint(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.typ.accept(visitor);
        self.trait_bound.accept(visitor);
    }
}

impl Pattern {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_pattern(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        match self {
            Pattern::Identifier(ident) => visitor.visit_identifier_pattern(ident),
            Pattern::Mutable(pattern, span, is_synthesized) => {
                if visitor.visit_mutable_pattern(pattern, *span, *is_synthesized) {
                    pattern.accept(visitor);
                }
            }
            Pattern::Tuple(patterns, span) => {
                if visitor.visit_tuple_pattern(patterns, *span) {
                    for pattern in patterns {
                        pattern.accept(visitor);
                    }
                }
            }
            Pattern::Struct(path, fields, span) => {
                if visitor.visit_struct_pattern(path, fields, *span) {
                    path.accept(visitor);
                    for (_, pattern) in fields {
                        pattern.accept(visitor);
                    }
                }
            }
            Pattern::Interned(id, span) => {
                visitor.visit_interned_pattern(id, *span);
            }
        }
    }
}

impl SecondaryAttribute {
    pub fn accept(&self, target: AttributeTarget, visitor: &mut impl Visitor) {
        if visitor.visit_secondary_attribute(self, target) {
            self.accept_children(target, visitor);
        }
    }

    pub fn accept_children(&self, target: AttributeTarget, visitor: &mut impl Visitor) {
        if let SecondaryAttribute::Meta(meta_attribute) = self {
            meta_attribute.accept(target, visitor);
        }
    }
}

impl MetaAttribute {
    pub fn accept(&self, target: AttributeTarget, visitor: &mut impl Visitor) {
        if visitor.visit_meta_attribute(self, target) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.name.accept(visitor);
        visit_expressions(&self.arguments, visitor);
    }
}

fn visit_expressions(expressions: &[Expression], visitor: &mut impl Visitor) {
    for expression in expressions {
        expression.accept(visitor);
    }
}

fn visit_unresolved_types(unresolved_type: &[UnresolvedType], visitor: &mut impl Visitor) {
    for unresolved_type in unresolved_type {
        unresolved_type.accept(visitor);
    }
}
