use noirc_frontend::{
    ast::{
        ArrayLiteral, AssignStatement, BlockExpression, CallExpression, CastExpression,
        ConstrainStatement, ConstructorExpression, Expression, ExpressionKind, ForLoopStatement,
        ForRange, IfExpression, IndexExpression, InfixExpression, LValue, Lambda, LetStatement,
        Literal, MemberAccessExpression, MethodCallExpression, ModuleDeclaration, NoirFunction,
        NoirStruct, NoirTrait, NoirTraitImpl, NoirTypeAlias, PrefixExpression, Statement,
        StatementKind, TraitImplItem, TraitItem, TypeImpl, UseTree, UseTreeKind,
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
pub(crate) trait Visitor {
    fn visit_parsed_module(&mut self, _: &ParsedModule) -> bool {
        true
    }

    fn visit_item(&mut self, _: &Item) -> bool {
        true
    }

    fn visit_parsed_submodule(&mut self, _: &ParsedSubModule) -> bool {
        true
    }

    fn visit_noir_function(&mut self, _: &NoirFunction) -> bool {
        true
    }

    fn visit_noir_trait_impl(&mut self, _: &NoirTraitImpl) -> bool {
        true
    }

    fn visit_type_impl(&mut self, _: &TypeImpl) -> bool {
        true
    }

    fn visit_trait_impl_item(&mut self, _: &TraitImplItem) -> bool {
        true
    }

    fn visit_noir_trait(&mut self, _: &NoirTrait) -> bool {
        true
    }

    fn visit_trait_item(&mut self, _: &TraitItem) {}

    fn visit_use_tree(&mut self, _: &UseTree) -> bool {
        true
    }

    fn visit_noir_struct(&mut self, _: &NoirStruct) -> bool {
        true
    }

    fn visit_noir_type_alias(&mut self, _: &NoirTypeAlias) {}

    fn visit_module_declaration(&mut self, _: &ModuleDeclaration) {}

    fn visit_expression(&mut self, _: &Expression) -> bool {
        true
    }

    fn visit_literal(&mut self, _: &Literal) -> bool {
        true
    }

    fn visit_block_expression(&mut self, _: &BlockExpression) -> bool {
        true
    }

    fn visit_prefix_expression(&mut self, _: &PrefixExpression) -> bool {
        true
    }

    fn visit_index_expression(&mut self, _: &IndexExpression) -> bool {
        true
    }

    fn visit_call_expression(&mut self, _: &CallExpression) -> bool {
        true
    }

    fn visit_method_call_expression(&mut self, _: &MethodCallExpression) -> bool {
        true
    }

    fn visit_constructor_expression(&mut self, _: &ConstructorExpression) -> bool {
        true
    }

    fn visit_member_access_expression(&mut self, _: &MemberAccessExpression) -> bool {
        true
    }

    fn visit_cast_expression(&mut self, _: &CastExpression) -> bool {
        true
    }

    fn visit_infix_expression(&mut self, _: &InfixExpression) -> bool {
        true
    }

    fn visit_if_expression(&mut self, _: &IfExpression) -> bool {
        true
    }

    fn visit_lambda(&mut self, _: &Lambda) -> bool {
        true
    }

    fn visit_array_literal(&mut self, _: &ArrayLiteral) -> bool {
        true
    }

    fn visit_statement(&mut self, _: &Statement) -> bool {
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

    fn visit_lvalue(&mut self, _: &LValue) -> bool {
        true
    }

    fn visit_for_range(&mut self, _: &ForRange) -> bool {
        true
    }
}

pub(crate) trait Acceptor {
    fn accept(&self, visitor: &mut impl Visitor);
}

pub(crate) trait ChildrenAcceptor {
    fn accept_children(&self, visitor: &mut impl Visitor);
}

impl Acceptor for ParsedModule {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_parsed_module(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for ParsedModule {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        for item in &self.items {
            item.accept(visitor);
        }
    }
}

impl Acceptor for Item {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_item(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for Item {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        match &self.kind {
            ItemKind::Submodules(parsed_sub_module) => {
                parsed_sub_module.accept(visitor);
            }
            ItemKind::Function(noir_function) => noir_function.accept(visitor),
            ItemKind::TraitImpl(noir_trait_impl) => {
                noir_trait_impl.accept(visitor);
            }
            ItemKind::Impl(type_impl) => type_impl.accept(visitor),
            ItemKind::Global(let_statement) => let_statement.accept(visitor),
            ItemKind::Trait(noir_trait) => noir_trait.accept(visitor),
            ItemKind::Import(use_tree) => use_tree.accept(visitor),
            ItemKind::TypeAlias(noir_type_alias) => noir_type_alias.accept(visitor),
            ItemKind::Struct(noir_struct) => noir_struct.accept(visitor),
            ItemKind::ModuleDecl(module_declaration) => module_declaration.accept(visitor),
        }
    }
}

impl Acceptor for ParsedSubModule {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_parsed_submodule(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for ParsedSubModule {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.contents.accept(visitor);
    }
}

impl Acceptor for NoirFunction {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_noir_function(self) {
            self.accept_children(visitor);
        }
    }
}
impl ChildrenAcceptor for NoirFunction {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.def.body.accept(visitor);
    }
}

impl Acceptor for NoirTraitImpl {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_noir_trait_impl(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for NoirTraitImpl {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        for item in &self.items {
            item.accept(visitor);
        }
    }
}

impl Acceptor for TraitImplItem {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_trait_impl_item(self) {
            self.accept_children(visitor);
        }
    }
}
impl ChildrenAcceptor for TraitImplItem {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        match self {
            TraitImplItem::Function(noir_function) => noir_function.accept(visitor),
            TraitImplItem::Constant(..) => (),
            TraitImplItem::Type { .. } => (),
        }
    }
}

impl Acceptor for TypeImpl {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_type_impl(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for TypeImpl {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        for (method, _span) in &self.methods {
            method.accept(visitor);
        }
    }
}

impl Acceptor for NoirTrait {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_noir_trait(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for NoirTrait {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        for item in &self.items {
            item.accept(visitor);
        }
    }
}

impl Acceptor for TraitItem {
    fn accept(&self, visitor: &mut impl Visitor) {
        visitor.visit_trait_item(self);
    }
}

impl Acceptor for UseTree {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_use_tree(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for UseTree {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        match &self.kind {
            UseTreeKind::Path(..) => (),
            UseTreeKind::List(use_trees) => {
                for use_tree in use_trees {
                    use_tree.accept(visitor);
                }
            }
        }
    }
}

impl Acceptor for NoirStruct {
    fn accept(&self, visitor: &mut impl Visitor) {
        visitor.visit_noir_struct(self);
    }
}

impl Acceptor for NoirTypeAlias {
    fn accept(&self, visitor: &mut impl Visitor) {
        visitor.visit_noir_type_alias(self);
    }
}

impl Acceptor for ModuleDeclaration {
    fn accept(&self, visitor: &mut impl Visitor) {
        visitor.visit_module_declaration(self);
    }
}

impl Acceptor for Expression {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_expression(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for Expression {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        match &self.kind {
            ExpressionKind::Literal(literal) => literal.accept(visitor),
            ExpressionKind::Block(block_expression) => {
                block_expression.accept(visitor);
            }
            ExpressionKind::Prefix(prefix_expression) => {
                prefix_expression.accept(visitor);
            }
            ExpressionKind::Index(index_expression) => {
                index_expression.accept(visitor);
            }
            ExpressionKind::Call(call_expression) => {
                call_expression.accept(visitor);
            }
            ExpressionKind::MethodCall(method_call_expression) => {
                method_call_expression.accept(visitor);
            }
            ExpressionKind::Constructor(constructor_expression) => {
                constructor_expression.accept(visitor);
            }
            ExpressionKind::MemberAccess(member_access_expression) => {
                member_access_expression.accept(visitor);
            }
            ExpressionKind::Cast(cast_expression) => {
                cast_expression.accept(visitor);
            }
            ExpressionKind::Infix(infix_expression) => {
                infix_expression.accept(visitor);
            }
            ExpressionKind::If(if_expression) => {
                if_expression.accept(visitor);
            }
            ExpressionKind::Tuple(expressions) => {
                visit_expressions(expressions, visitor);
            }
            ExpressionKind::Lambda(lambda) => lambda.accept(visitor),
            ExpressionKind::Parenthesized(expression) => {
                expression.accept(visitor);
            }
            ExpressionKind::Unquote(expression) => {
                expression.accept(visitor);
            }
            ExpressionKind::Comptime(block_expression, _) => {
                block_expression.accept(visitor);
            }
            ExpressionKind::Unsafe(block_expression, _) => {
                block_expression.accept(visitor);
            }
            ExpressionKind::Variable(_)
            | ExpressionKind::AsTraitPath(_)
            | ExpressionKind::Quote(_)
            | ExpressionKind::Resolved(_)
            | ExpressionKind::Error => (),
        }
    }
}

impl Acceptor for Literal {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_literal(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for Literal {
    fn accept_children(&self, visitor: &mut impl Visitor) {
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

impl Acceptor for BlockExpression {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_block_expression(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for BlockExpression {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        for statement in &self.statements {
            statement.accept(visitor);
        }
    }
}

impl Acceptor for PrefixExpression {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_prefix_expression(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for PrefixExpression {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.rhs.accept(visitor);
    }
}

impl Acceptor for IndexExpression {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_index_expression(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for IndexExpression {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.collection.accept(visitor);
        self.index.accept(visitor);
    }
}

impl Acceptor for CallExpression {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_call_expression(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for CallExpression {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.func.accept(visitor);
        visit_expressions(&self.arguments, visitor);
    }
}

impl Acceptor for MethodCallExpression {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_method_call_expression(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for MethodCallExpression {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.object.accept(visitor);
        visit_expressions(&self.arguments, visitor);
    }
}

impl Acceptor for ConstructorExpression {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_constructor_expression(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for ConstructorExpression {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        for (_field_name, expression) in &self.fields {
            expression.accept(visitor);
        }
    }
}

impl Acceptor for MemberAccessExpression {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_member_access_expression(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for MemberAccessExpression {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.lhs.accept(visitor);
    }
}

impl Acceptor for CastExpression {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_cast_expression(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for CastExpression {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.lhs.accept(visitor);
    }
}

impl Acceptor for InfixExpression {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_infix_expression(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for InfixExpression {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.lhs.accept(visitor);
        self.rhs.accept(visitor);
    }
}

impl Acceptor for IfExpression {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_if_expression(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for IfExpression {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.condition.accept(visitor);
        self.consequence.accept(visitor);
        if let Some(alternative) = &self.alternative {
            alternative.accept(visitor);
        }
    }
}

impl Acceptor for Lambda {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_lambda(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for Lambda {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.body.accept(visitor);
    }
}

impl Acceptor for ArrayLiteral {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_array_literal(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for ArrayLiteral {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        match self {
            ArrayLiteral::Standard(expressions) => visit_expressions(expressions, visitor),
            ArrayLiteral::Repeated { repeated_element, length } => {
                repeated_element.accept(visitor);
                length.accept(visitor);
            }
        }
    }
}

impl Acceptor for Statement {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_statement(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for Statement {
    fn accept_children(&self, visitor: &mut impl Visitor) {
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
                statement.accept(visitor);
            }
            StatementKind::Semi(expression) => {
                expression.accept(visitor);
            }
            StatementKind::Break | StatementKind::Continue | StatementKind::Error => (),
        }
    }
}

impl Acceptor for LetStatement {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_let_statement(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for LetStatement {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.expression.accept(visitor);
    }
}

impl Acceptor for ConstrainStatement {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_constrain_statement(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for ConstrainStatement {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.0.accept(visitor);

        if let Some(exp) = &self.1 {
            exp.accept(visitor);
        }
    }
}

impl Acceptor for AssignStatement {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_assign_statement(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for AssignStatement {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.lvalue.accept(visitor);
        self.expression.accept(visitor);
    }
}

impl Acceptor for ForLoopStatement {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_for_loop_statement(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for ForLoopStatement {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        self.range.accept(visitor);
        self.block.accept(visitor);
    }
}

impl Acceptor for LValue {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_lvalue(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for LValue {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        match self {
            LValue::Ident(..) => (),
            LValue::MemberAccess { object, field_name: _, span: _ } => object.accept(visitor),
            LValue::Index { array, index, span: _ } => {
                array.accept(visitor);
                index.accept(visitor);
            }
            LValue::Dereference(lvalue, _) => lvalue.accept(visitor),
        }
    }
}

impl Acceptor for ForRange {
    fn accept(&self, visitor: &mut impl Visitor) {
        if visitor.visit_for_range(self) {
            self.accept_children(visitor);
        }
    }
}

impl ChildrenAcceptor for ForRange {
    fn accept_children(&self, visitor: &mut impl Visitor) {
        match self {
            ForRange::Range(start, end) => {
                start.accept(visitor);
                end.accept(visitor);
            }
            ForRange::Array(expression) => expression.accept(visitor),
        }
    }
}

fn visit_expressions(expressions: &[Expression], visitor: &mut impl Visitor) {
    for expression in expressions {
        expression.accept(visitor);
    }
}
