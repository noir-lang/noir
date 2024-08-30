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
    parser::{Item, ItemKind, ParsedSubModule},
    ParsedModule,
};

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

    fn visit_noir_function(&mut self, _: &NoirFunction, _: Option<Span>) -> bool {
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

    fn visit_noir_trait(&mut self, _: &NoirTrait, _: Span) -> bool {
        true
    }

    fn visit_trait_item(&mut self, _: &TraitItem) -> bool {
        true
    }

    fn visit_use_tree(&mut self, _: &UseTree) -> bool {
        true
    }

    fn visit_use_tree_path(&mut self, _: &UseTree, _ident: &Ident, _alias: &Option<Ident>) {}

    fn visit_use_tree_list(&mut self, _: &UseTree, _: &[UseTree]) -> bool {
        true
    }

    fn visit_noir_struct(&mut self, _: &NoirStruct, _: Span) {}

    fn visit_noir_type_alias(&mut self, _: &NoirTypeAlias, _: Span) {}

    fn visit_module_declaration(&mut self, _: &ModuleDeclaration, _: Span) {}

    fn visit_expression(&mut self, _: &Expression) -> bool {
        true
    }

    fn visit_literal(&mut self, _: &Literal, _: Span) -> bool {
        true
    }

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

    fn visit_variable(&mut self, _: &Path, _: Span) {}

    fn visit_lambda(&mut self, _: &Lambda, _: Span) -> bool {
        true
    }

    fn visit_array_literal(&mut self, _: &ArrayLiteral) -> bool {
        true
    }

    fn visit_statement(&mut self, _: &Statement) -> bool {
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

    fn visit_comptime_statement(&mut self, _: &Statement) -> bool {
        true
    }

    fn visit_lvalue(&mut self, _: &LValue) -> bool {
        true
    }

    fn visit_lvalue_ident(&mut self, _: &Ident) {}

    fn visit_for_range(&mut self, _: &ForRange) -> bool {
        true
    }

    fn visit_as_trait_path(&mut self, _: &AsTraitPath, _: Span) {}
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
            ItemKind::Function(noir_function) => noir_function.accept(Some(self.span), visitor),
            ItemKind::TraitImpl(noir_trait_impl) => {
                noir_trait_impl.accept(self.span, visitor);
            }
            ItemKind::Impl(type_impl) => type_impl.accept(self.span, visitor),
            ItemKind::Global(let_statement) => {
                if visitor.visit_global(let_statement, self.span) {
                    let_statement.accept(visitor)
                }
            }
            ItemKind::Trait(noir_trait) => noir_trait.accept(self.span, visitor),
            ItemKind::Import(use_tree) => use_tree.accept(visitor),
            ItemKind::TypeAlias(noir_type_alias) => noir_type_alias.accept(self.span, visitor),
            ItemKind::Struct(noir_struct) => noir_struct.accept(self.span, visitor),
            ItemKind::ModuleDecl(module_declaration) => {
                module_declaration.accept(self.span, visitor)
            }
        }
    }
}

impl ParsedSubModule {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        if visitor.visit_parsed_submodule(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
        self.contents.accept(visitor);
    }
}

impl NoirFunction {
    pub fn accept(&self, span: Option<Span>, visitor: &mut impl Visitor) {
        if visitor.visit_noir_function(self, span) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
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
        for item in &self.items {
            item.accept(visitor);
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
        match self {
            TraitImplItem::Function(noir_function) => noir_function.accept(None, visitor),
            TraitImplItem::Constant(..) => (),
            TraitImplItem::Type { .. } => (),
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
        for (method, span) in &self.methods {
            method.accept(Some(*span), visitor);
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
        for item in &self.items {
            item.accept(visitor);
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
                name: _,
                generics: _,
                parameters: _,
                return_type: _,
                where_clause: _,
                body,
            } => {
                if let Some(body) = body {
                    body.accept(None, visitor);
                }
            }
            TraitItem::Constant { name: _, typ: _, default_value } => {
                if let Some(default_value) = default_value {
                    default_value.accept(visitor);
                }
            }
            TraitItem::Type { name: _ } => (),
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
        visitor.visit_noir_struct(self, span);
    }
}

impl NoirTypeAlias {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        visitor.visit_noir_type_alias(self, span);
    }
}

impl ModuleDeclaration {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
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
                visitor.visit_variable(path, self.span);
            }
            ExpressionKind::AsTraitPath(as_trait_path) => {
                as_trait_path.accept(self.span, visitor);
            }
            ExpressionKind::Quote(_)
            | ExpressionKind::Resolved(_)
            | ExpressionKind::Interned(_)
            | ExpressionKind::Error => (),
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
            Literal::Array(array_literal) | Literal::Slice(array_literal) => {
                array_literal.accept(visitor);
            }
            Literal::Bool(_)
            | Literal::Integer(_, _)
            | Literal::Str(_)
            | Literal::RawStr(_, _)
            | Literal::FmtStr(_)
            | Literal::Unit => (),
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
            ArrayLiteral::Standard(expressions) => visit_expressions(expressions, visitor),
            ArrayLiteral::Repeated { repeated_element, length } => {
                repeated_element.accept(visitor);
                length.accept(visitor);
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
            StatementKind::Comptime(statement) => {
                if visitor.visit_comptime_statement(statement) {
                    statement.accept(visitor);
                }
            }
            StatementKind::Semi(expression) => {
                expression.accept(visitor);
            }
            StatementKind::Break
            | StatementKind::Continue
            | StatementKind::Interned(_)
            | StatementKind::Error => (),
        }
    }
}

impl LetStatement {
    pub fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_let_statement(self) {
            self.accept_children(visitor);
        }
    }

    pub fn accept_children(&self, visitor: &mut impl Visitor) {
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
        self.0.accept(visitor);

        if let Some(exp) = &self.1 {
            exp.accept(visitor);
        }
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
            LValue::MemberAccess { object, field_name: _, span: _ } => object.accept(visitor),
            LValue::Index { array, index, span: _ } => {
                array.accept(visitor);
                index.accept(visitor);
            }
            LValue::Dereference(lvalue, _) => lvalue.accept(visitor),
            LValue::Interned(..) => (),
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
            ForRange::Range(start, end) => {
                start.accept(visitor);
                end.accept(visitor);
            }
            ForRange::Array(expression) => expression.accept(visitor),
        }
    }
}

impl AsTraitPath {
    pub fn accept(&self, span: Span, visitor: &mut impl Visitor) {
        visitor.visit_as_trait_path(self, span);
    }
}

fn visit_expressions(expressions: &[Expression], visitor: &mut impl Visitor) {
    for expression in expressions {
        expression.accept(visitor);
    }
}
