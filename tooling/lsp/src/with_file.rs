use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::Location;
use noirc_frontend::{
    ast::{
        ArrayLiteral, AsTraitPath, AssignStatement, BlockExpression, CallExpression,
        CastExpression, ConstrainExpression, ConstructorExpression, Documented, EnumVariant,
        Expression, ExpressionKind, ForBounds, ForLoopStatement, ForRange, FunctionDefinition,
        FunctionReturnType, GenericTypeArgs, Ident, IfExpression, IndexExpression, InfixExpression,
        LValue, Lambda, LetStatement, Literal, MatchExpression, MemberAccessExpression,
        MethodCallExpression, ModuleDeclaration, NoirEnumeration, NoirFunction, NoirStruct,
        NoirTrait, NoirTraitImpl, NoirTypeAlias, Param, Path, PathSegment, Pattern,
        PrefixExpression, Statement, StatementKind, StructField, TraitBound, TraitImplItem,
        TraitImplItemKind, TraitItem, TypeImpl, TypePath, UnresolvedGeneric,
        UnresolvedTraitConstraint, UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression,
        UseTree, UseTreeKind, WhileStatement,
    },
    parser::{Item, ItemKind, ParsedSubModule},
    token::{
        Attributes, FmtStrFragment, LocatedToken, MetaAttribute, SecondaryAttribute, Token, Tokens,
    },
    ParsedModule,
};

/// Returns a copy of the given ParsedModule with all FileIds present in its locations changed to the given FileId.
pub(super) fn parsed_module_with_file(parsed_module: ParsedModule, file: FileId) -> ParsedModule {
    ParsedModule {
        items: parsed_module.items.into_iter().map(|item| item_with_file(item, file)).collect(),
        inner_doc_comments: parsed_module.inner_doc_comments,
    }
}

fn item_with_file(item: Item, file: FileId) -> Item {
    Item {
        kind: item_kind_with_file(item.kind, file),
        location: item.location,
        doc_comments: item.doc_comments,
    }
}

fn item_kind_with_file(item_kind: ItemKind, file: FileId) -> ItemKind {
    match item_kind {
        ItemKind::Import(use_tree, item_visibility) => {
            ItemKind::Import(use_tree_with_file(use_tree, file), item_visibility)
        }
        ItemKind::Function(noir_function) => {
            ItemKind::Function(noir_function_with_file(noir_function, file))
        }
        ItemKind::Struct(noir_struct) => ItemKind::Struct(noir_struct_with_file(noir_struct, file)),
        ItemKind::Enum(noir_enumeration) => {
            ItemKind::Enum(noir_enumeration_with_file(noir_enumeration, file))
        }
        ItemKind::Trait(noir_trait) => ItemKind::Trait(noir_trait_with_file(noir_trait, file)),
        ItemKind::TraitImpl(noir_trait_impl) => {
            ItemKind::TraitImpl(noir_trait_impl_with_file(noir_trait_impl, file))
        }
        ItemKind::Impl(type_impl) => ItemKind::Impl(type_impl_with_file(type_impl, file)),
        ItemKind::TypeAlias(noir_type_alias) => {
            ItemKind::TypeAlias(noir_type_alias_with_file(noir_type_alias, file))
        }
        ItemKind::Global(let_statement, item_visibility) => {
            ItemKind::Global(let_statement_with_file(let_statement, file), item_visibility)
        }
        ItemKind::ModuleDecl(module_declaration) => {
            ItemKind::ModuleDecl(module_declaration_with_file(module_declaration, file))
        }
        ItemKind::Submodules(parsed_sub_module) => {
            ItemKind::Submodules(parsed_sub_module_with_file(parsed_sub_module, file))
        }
        ItemKind::InnerAttribute(secondary_attribute) => {
            ItemKind::InnerAttribute(secondary_attribute_with_file(secondary_attribute, file))
        }
    }
}

fn parsed_sub_module_with_file(module: ParsedSubModule, file: FileId) -> ParsedSubModule {
    ParsedSubModule {
        visibility: module.visibility,
        name: ident_with_file(module.name, file),
        contents: parsed_module_with_file(module.contents, file),
        outer_attributes: secondary_attributes_with_file(module.outer_attributes, file),
        is_contract: module.is_contract,
    }
}

fn module_declaration_with_file(module: ModuleDeclaration, file: FileId) -> ModuleDeclaration {
    ModuleDeclaration {
        visibility: module.visibility,
        ident: ident_with_file(module.ident, file),
        outer_attributes: secondary_attributes_with_file(module.outer_attributes, file),
        has_semicolon: module.has_semicolon,
    }
}

fn let_statement_with_file(let_statement: LetStatement, file: FileId) -> LetStatement {
    LetStatement {
        pattern: pattern_with_file(let_statement.pattern, file),
        r#type: unresolved_type_with_file(let_statement.r#type, file),
        expression: expression_with_file(let_statement.expression, file),
        attributes: secondary_attributes_with_file(let_statement.attributes, file),
        comptime: let_statement.comptime,
        is_global_let: let_statement.is_global_let,
    }
}

fn patterns_with_file(patterns: Vec<Pattern>, file: FileId) -> Vec<Pattern> {
    vecmap(patterns, |pattern| pattern_with_file(pattern, file))
}

fn pattern_with_file(pattern: Pattern, file: FileId) -> Pattern {
    match pattern {
        Pattern::Identifier(ident) => Pattern::Identifier(ident_with_file(ident, file)),
        Pattern::Mutable(pattern, location, synthesized) => Pattern::Mutable(
            Box::new(pattern_with_file(*pattern, file)),
            location_with_file(location, file),
            synthesized,
        ),
        Pattern::Tuple(patterns, location) => {
            Pattern::Tuple(patterns_with_file(patterns, file), location_with_file(location, file))
        }
        Pattern::Struct(path, items, location) => Pattern::Struct(
            path_with_file(path, file),
            vecmap(items, |(ident, pattern)| {
                (ident_with_file(ident, file), pattern_with_file(pattern, file))
            }),
            location_with_file(location, file),
        ),
        Pattern::Interned(interned_pattern, location) => {
            Pattern::Interned(interned_pattern, location_with_file(location, file))
        }
    }
}

fn noir_type_alias_with_file(noir_type_alias: NoirTypeAlias, file: FileId) -> NoirTypeAlias {
    NoirTypeAlias {
        name: ident_with_file(noir_type_alias.name, file),
        generics: unresolved_generics_with_file(noir_type_alias.generics, file),
        typ: unresolved_type_with_file(noir_type_alias.typ, file),
        visibility: noir_type_alias.visibility,
        location: location_with_file(noir_type_alias.location, file),
    }
}

fn type_impl_with_file(type_impl: TypeImpl, file: FileId) -> TypeImpl {
    TypeImpl {
        object_type: unresolved_type_with_file(type_impl.object_type, file),
        type_location: location_with_file(type_impl.type_location, file),
        generics: unresolved_generics_with_file(type_impl.generics, file),
        where_clause: unresolved_trait_constraints_with_file(type_impl.where_clause, file),
        methods: documented_noir_functions_with_file(type_impl.methods, file),
    }
}

fn documented_noir_functions_with_file(
    methods: Vec<(Documented<NoirFunction>, Location)>,
    file: FileId,
) -> Vec<(Documented<NoirFunction>, Location)> {
    vecmap(methods, |(method, location)| {
        (documented_noir_function_with_file(method, file), location_with_file(location, file))
    })
}

fn documented_noir_function_with_file(
    function: Documented<NoirFunction>,
    file: FileId,
) -> Documented<NoirFunction> {
    Documented {
        item: noir_function_with_file(function.item, file),
        doc_comments: function.doc_comments,
    }
}

fn noir_trait_impl_with_file(noir_trait_impl: NoirTraitImpl, file: FileId) -> NoirTraitImpl {
    NoirTraitImpl {
        impl_generics: unresolved_generics_with_file(noir_trait_impl.impl_generics, file),
        trait_name: path_with_file(noir_trait_impl.trait_name, file),
        trait_generics: generic_type_args_with_file(noir_trait_impl.trait_generics, file),
        object_type: unresolved_type_with_file(noir_trait_impl.object_type, file),
        where_clause: unresolved_trait_constraints_with_file(noir_trait_impl.where_clause, file),
        items: documented_trait_impl_items_with_file(noir_trait_impl.items, file),
        is_synthetic: noir_trait_impl.is_synthetic,
    }
}

fn documented_trait_impl_items_with_file(
    items: Vec<Documented<TraitImplItem>>,
    file: FileId,
) -> Vec<Documented<TraitImplItem>> {
    vecmap(items, |item| documented_trait_impl_item_with_file(item, file))
}

fn documented_trait_impl_item_with_file(
    item: Documented<TraitImplItem>,
    file: FileId,
) -> Documented<TraitImplItem> {
    Documented { item: trait_impl_item_with_file(item.item, file), doc_comments: item.doc_comments }
}

fn trait_impl_item_with_file(item: TraitImplItem, file: FileId) -> TraitImplItem {
    TraitImplItem {
        kind: trait_impl_item_kind_with_file(item.kind, file),
        location: location_with_file(item.location, file),
    }
}

fn trait_impl_item_kind_with_file(kind: TraitImplItemKind, file: FileId) -> TraitImplItemKind {
    match kind {
        TraitImplItemKind::Function(noir_function) => {
            TraitImplItemKind::Function(noir_function_with_file(noir_function, file))
        }
        TraitImplItemKind::Constant(ident, typ, expression) => TraitImplItemKind::Constant(
            ident_with_file(ident, file),
            unresolved_type_with_file(typ, file),
            expression_with_file(expression, file),
        ),
        TraitImplItemKind::Type { name, alias } => TraitImplItemKind::Type {
            name: ident_with_file(name, file),
            alias: unresolved_type_with_file(alias, file),
        },
    }
}

fn noir_trait_with_file(noir_trait: NoirTrait, file: FileId) -> NoirTrait {
    NoirTrait {
        name: ident_with_file(noir_trait.name, file),
        generics: unresolved_generics_with_file(noir_trait.generics, file),
        bounds: trait_bounds_with_file(noir_trait.bounds, file),
        where_clause: unresolved_trait_constraints_with_file(noir_trait.where_clause, file),
        location: location_with_file(noir_trait.location, file),
        items: documented_trait_items_with_file(noir_trait.items, file),
        attributes: secondary_attributes_with_file(noir_trait.attributes, file),
        visibility: noir_trait.visibility,
        is_alias: noir_trait.is_alias,
    }
}

fn documented_trait_items_with_file(
    items: Vec<Documented<TraitItem>>,
    file: FileId,
) -> Vec<Documented<TraitItem>> {
    vecmap(items, |item| documented_trait_item_with_file(item, file))
}

fn documented_trait_item_with_file(
    item: Documented<TraitItem>,
    file: FileId,
) -> Documented<TraitItem> {
    Documented { item: trait_item_with_file(item.item, file), doc_comments: item.doc_comments }
}

fn trait_item_with_file(item: TraitItem, file: FileId) -> TraitItem {
    match item {
        TraitItem::Function {
            is_unconstrained,
            visibility,
            is_comptime,
            name,
            generics,
            parameters,
            return_type,
            where_clause,
            body,
        } => TraitItem::Function {
            is_unconstrained,
            visibility,
            is_comptime,
            name: ident_with_file(name, file),
            generics: unresolved_generics_with_file(generics, file),
            parameters: vecmap(parameters, |(ident, typ)| {
                (ident_with_file(ident, file), unresolved_type_with_file(typ, file))
            }),
            return_type: function_return_type_with_file(return_type, file),
            where_clause: unresolved_trait_constraints_with_file(where_clause, file),
            body: body.map(|body| block_expression_with_file(body, file)),
        },
        TraitItem::Constant { name, typ, default_value } => TraitItem::Constant {
            name: ident_with_file(name, file),
            typ: unresolved_type_with_file(typ, file),
            default_value: default_value.map(|value| expression_with_file(value, file)),
        },
        TraitItem::Type { name } => TraitItem::Type { name: ident_with_file(name, file) },
    }
}

fn function_return_type_with_file(typ: FunctionReturnType, file: FileId) -> FunctionReturnType {
    match typ {
        FunctionReturnType::Default(location) => {
            FunctionReturnType::Default(location_with_file(location, file))
        }
        FunctionReturnType::Ty(typ) => FunctionReturnType::Ty(unresolved_type_with_file(typ, file)),
    }
}

fn noir_function_with_file(noir_function: NoirFunction, file: FileId) -> NoirFunction {
    NoirFunction {
        kind: noir_function.kind,
        def: function_definition_with_file(noir_function.def, file),
    }
}

fn noir_struct_with_file(noir_struct: NoirStruct, file: FileId) -> NoirStruct {
    NoirStruct {
        name: ident_with_file(noir_struct.name, file),
        attributes: secondary_attributes_with_file(noir_struct.attributes, file),
        visibility: noir_struct.visibility,
        generics: unresolved_generics_with_file(noir_struct.generics, file),
        fields: documented_struct_fields_with_file(noir_struct.fields, file),
        location: location_with_file(noir_struct.location, file),
    }
}

fn documented_struct_fields_with_file(
    fields: Vec<Documented<StructField>>,
    file: FileId,
) -> Vec<Documented<StructField>> {
    vecmap(fields, |field| documented_struct_field_with_file(field, file))
}

fn documented_struct_field_with_file(
    field: Documented<StructField>,
    file: FileId,
) -> Documented<StructField> {
    Documented { item: struct_field_with_file(field.item, file), doc_comments: field.doc_comments }
}

fn struct_field_with_file(field: StructField, file: FileId) -> StructField {
    StructField {
        visibility: field.visibility,
        name: ident_with_file(field.name, file),
        typ: unresolved_type_with_file(field.typ, file),
    }
}

fn noir_enumeration_with_file(noir_enumeration: NoirEnumeration, file: FileId) -> NoirEnumeration {
    NoirEnumeration {
        name: ident_with_file(noir_enumeration.name, file),
        attributes: secondary_attributes_with_file(noir_enumeration.attributes, file),
        visibility: noir_enumeration.visibility,
        generics: unresolved_generics_with_file(noir_enumeration.generics, file),
        variants: documented_enum_variants_with_file(noir_enumeration.variants, file),
        location: location_with_file(noir_enumeration.location, file),
    }
}

fn documented_enum_variants_with_file(
    variants: Vec<Documented<EnumVariant>>,
    file: FileId,
) -> Vec<Documented<EnumVariant>> {
    vecmap(variants, |variant| documented_enum_variant_with_file(variant, file))
}

fn documented_enum_variant_with_file(
    variant: Documented<EnumVariant>,
    file: FileId,
) -> Documented<EnumVariant> {
    Documented {
        item: enum_variant_with_file(variant.item, file),
        doc_comments: variant.doc_comments,
    }
}

fn enum_variant_with_file(variant: EnumVariant, file: FileId) -> EnumVariant {
    EnumVariant {
        name: ident_with_file(variant.name, file),
        parameters: variant.parameters.map(|params| unresolved_types_with_file(params, file)),
    }
}

fn function_definition_with_file(func: FunctionDefinition, file: FileId) -> FunctionDefinition {
    FunctionDefinition {
        name: ident_with_file(func.name, file),
        attributes: attributes_with_file(func.attributes, file),
        is_unconstrained: func.is_unconstrained,
        is_comptime: func.is_comptime,
        visibility: func.visibility,
        generics: unresolved_generics_with_file(func.generics, file),
        parameters: params_with_file(func.parameters, file),
        body: block_expression_with_file(func.body, file),
        location: location_with_file(func.location, file),
        where_clause: unresolved_trait_constraints_with_file(func.where_clause, file),
        return_type: function_return_type_with_file(func.return_type, file),
        return_visibility: func.return_visibility,
    }
}

fn params_with_file(params: Vec<Param>, file: FileId) -> Vec<Param> {
    vecmap(params, |param| param_with_file(param, file))
}

fn param_with_file(param: Param, file: FileId) -> Param {
    Param {
        visibility: param.visibility,
        pattern: pattern_with_file(param.pattern, file),
        typ: unresolved_type_with_file(param.typ, file),
        location: location_with_file(param.location, file),
    }
}

fn attributes_with_file(attributes: Attributes, file: FileId) -> Attributes {
    Attributes {
        function: attributes.function,
        secondary: secondary_attributes_with_file(attributes.secondary, file),
    }
}

fn use_tree_with_file(use_tree: UseTree, file: FileId) -> UseTree {
    UseTree {
        prefix: path_with_file(use_tree.prefix, file),
        kind: use_tree_kind_with_file(use_tree.kind, file),
        location: location_with_file(use_tree.location, file),
    }
}

fn use_tree_kind_with_file(kind: UseTreeKind, file: FileId) -> UseTreeKind {
    match kind {
        UseTreeKind::Path(ident, alias) => UseTreeKind::Path(
            ident_with_file(ident, file),
            alias.map(|alias| ident_with_file(alias, file)),
        ),
        UseTreeKind::List(use_trees) => {
            UseTreeKind::List(vecmap(use_trees, |use_tree| use_tree_with_file(use_tree, file)))
        }
    }
}

fn path_with_file(path: Path, file: FileId) -> Path {
    Path {
        segments: vecmap(path.segments, |segment| path_segment_with_file(segment, file)),
        kind: path.kind,
        location: location_with_file(path.location, file),
    }
}

fn path_segment_with_file(segment: PathSegment, file: FileId) -> PathSegment {
    PathSegment {
        ident: ident_with_file(segment.ident, file),
        generics: segment.generics.map(|generics| unresolved_types_with_file(generics, file)),
        location: location_with_file(segment.location, file),
    }
}

fn unresolved_types_with_file(types: Vec<UnresolvedType>, file: FileId) -> Vec<UnresolvedType> {
    vecmap(types, |typ| unresolved_type_with_file(typ, file))
}

fn unresolved_type_with_file(typ: UnresolvedType, file: FileId) -> UnresolvedType {
    UnresolvedType {
        typ: unresolved_type_data_with_file(typ.typ, file),
        location: location_with_file(typ.location, file),
    }
}

fn unresolved_type_data_with_file(typ: UnresolvedTypeData, file: FileId) -> UnresolvedTypeData {
    match typ {
        UnresolvedTypeData::Array(length, typ) => UnresolvedTypeData::Array(
            unresolved_type_expression_with_file(length, file),
            Box::new(unresolved_type_with_file(*typ, file)),
        ),
        UnresolvedTypeData::Slice(typ) => {
            UnresolvedTypeData::Slice(Box::new(unresolved_type_with_file(*typ, file)))
        }
        UnresolvedTypeData::Expression(expr) => {
            UnresolvedTypeData::Expression(unresolved_type_expression_with_file(expr, file))
        }
        UnresolvedTypeData::String(expr) => {
            UnresolvedTypeData::String(unresolved_type_expression_with_file(expr, file))
        }
        UnresolvedTypeData::FormatString(expr, typ) => UnresolvedTypeData::FormatString(
            unresolved_type_expression_with_file(expr, file),
            Box::new(unresolved_type_with_file(*typ, file)),
        ),
        UnresolvedTypeData::Parenthesized(typ) => {
            UnresolvedTypeData::Parenthesized(Box::new(unresolved_type_with_file(*typ, file)))
        }
        UnresolvedTypeData::Named(path, generic_type_args, synthesized) => {
            UnresolvedTypeData::Named(
                path_with_file(path, file),
                generic_type_args_with_file(generic_type_args, file),
                synthesized,
            )
        }
        UnresolvedTypeData::TraitAsType(path, generic_type_args) => {
            UnresolvedTypeData::TraitAsType(
                path_with_file(path, file),
                generic_type_args_with_file(generic_type_args, file),
            )
        }
        UnresolvedTypeData::MutableReference(typ) => {
            UnresolvedTypeData::MutableReference(Box::new(unresolved_type_with_file(*typ, file)))
        }
        UnresolvedTypeData::Tuple(types) => {
            UnresolvedTypeData::Tuple(unresolved_types_with_file(types, file))
        }
        UnresolvedTypeData::Function(args, ret, env, unconstrained) => {
            UnresolvedTypeData::Function(
                unresolved_types_with_file(args, file),
                Box::new(unresolved_type_with_file(*ret, file)),
                Box::new(unresolved_type_with_file(*env, file)),
                unconstrained,
            )
        }
        UnresolvedTypeData::AsTraitPath(as_trait_path) => {
            UnresolvedTypeData::AsTraitPath(Box::new(as_trait_path_with_file(*as_trait_path, file)))
        }
        UnresolvedTypeData::Quoted(..)
        | UnresolvedTypeData::Resolved(..)
        | UnresolvedTypeData::Interned(..)
        | UnresolvedTypeData::Unit
        | UnresolvedTypeData::Bool
        | UnresolvedTypeData::Integer(..)
        | UnresolvedTypeData::FieldElement
        | UnresolvedTypeData::Unspecified
        | UnresolvedTypeData::Error => typ,
    }
}

fn unresolved_type_expression_with_file(
    type_expr: UnresolvedTypeExpression,
    file: FileId,
) -> UnresolvedTypeExpression {
    match type_expr {
        UnresolvedTypeExpression::Variable(path) => {
            UnresolvedTypeExpression::Variable(path_with_file(path, file))
        }
        UnresolvedTypeExpression::Constant(field_element, location) => {
            UnresolvedTypeExpression::Constant(field_element, location_with_file(location, file))
        }
        UnresolvedTypeExpression::BinaryOperation(lhs, op, rhs, location) => {
            UnresolvedTypeExpression::BinaryOperation(
                Box::new(unresolved_type_expression_with_file(*lhs, file)),
                op,
                Box::new(unresolved_type_expression_with_file(*rhs, file)),
                location_with_file(location, file),
            )
        }
        UnresolvedTypeExpression::AsTraitPath(as_trait_path) => {
            UnresolvedTypeExpression::AsTraitPath(Box::new(as_trait_path_with_file(
                *as_trait_path,
                file,
            )))
        }
    }
}

fn as_trait_path_with_file(as_trait_path: AsTraitPath, file: FileId) -> AsTraitPath {
    AsTraitPath {
        typ: unresolved_type_with_file(as_trait_path.typ, file),
        trait_path: path_with_file(as_trait_path.trait_path, file),
        trait_generics: generic_type_args_with_file(as_trait_path.trait_generics, file),
        impl_item: ident_with_file(as_trait_path.impl_item, file),
    }
}

fn generic_type_args_with_file(generics: GenericTypeArgs, file: FileId) -> GenericTypeArgs {
    GenericTypeArgs {
        ordered_args: unresolved_types_with_file(generics.ordered_args, file),
        named_args: vecmap(generics.named_args, |(ident, typ)| {
            (ident_with_file(ident, file), unresolved_type_with_file(typ, file))
        }),
        kinds: generics.kinds,
    }
}

fn ident_with_file(ident: Ident, file: FileId) -> Ident {
    let location = location_with_file(ident.location(), file);
    Ident::new(ident.0.contents, location)
}

fn secondary_attributes_with_file(
    attributes: Vec<SecondaryAttribute>,
    file: FileId,
) -> Vec<SecondaryAttribute> {
    vecmap(attributes, |attribute| secondary_attribute_with_file(attribute, file))
}

fn secondary_attribute_with_file(
    secondary_attribute: SecondaryAttribute,
    file: FileId,
) -> SecondaryAttribute {
    match secondary_attribute {
        SecondaryAttribute::Meta(meta_attribute) => {
            SecondaryAttribute::Meta(meta_attribute_with_file(meta_attribute, file))
        }
        SecondaryAttribute::Deprecated(_)
        | SecondaryAttribute::ContractLibraryMethod
        | SecondaryAttribute::Export
        | SecondaryAttribute::Field(_)
        | SecondaryAttribute::Tag(..)
        | SecondaryAttribute::Abi(_)
        | SecondaryAttribute::Varargs
        | SecondaryAttribute::UseCallersScope
        | SecondaryAttribute::Allow(_) => secondary_attribute,
    }
}

fn meta_attribute_with_file(meta_attribute: MetaAttribute, file: FileId) -> MetaAttribute {
    MetaAttribute {
        name: path_with_file(meta_attribute.name, file),
        arguments: expressions_with_file(meta_attribute.arguments, file),
        location: location_with_file(meta_attribute.location, file),
    }
}

fn expressions_with_file(expressions: Vec<Expression>, file: FileId) -> Vec<Expression> {
    vecmap(expressions, |expr| expression_with_file(expr, file))
}

fn expression_with_file(expr: Expression, file: FileId) -> Expression {
    Expression {
        kind: expression_kind_with_file(expr.kind, file),
        location: location_with_file(expr.location, file),
    }
}

fn expression_kind_with_file(kind: ExpressionKind, file: FileId) -> ExpressionKind {
    match kind {
        ExpressionKind::Literal(literal) => {
            ExpressionKind::Literal(literal_with_file(literal, file))
        }
        ExpressionKind::Block(block_expression) => {
            ExpressionKind::Block(block_expression_with_file(block_expression, file))
        }
        ExpressionKind::Prefix(prefix_expression) => {
            ExpressionKind::Prefix(Box::new(prefix_expression_with_file(*prefix_expression, file)))
        }
        ExpressionKind::Index(index_expression) => {
            ExpressionKind::Index(Box::new(index_expression_with_file(*index_expression, file)))
        }
        ExpressionKind::Call(call_expression) => {
            ExpressionKind::Call(Box::new(call_expression_with_file(*call_expression, file)))
        }
        ExpressionKind::MethodCall(method_call_expression) => ExpressionKind::MethodCall(Box::new(
            method_call_expression_with_file(*method_call_expression, file),
        )),
        ExpressionKind::Constrain(constrain_expression) => {
            ExpressionKind::Constrain(constrain_expression_with_file(constrain_expression, file))
        }
        ExpressionKind::Constructor(constructor_expression) => ExpressionKind::Constructor(
            Box::new(constructor_expression_with_file(*constructor_expression, file)),
        ),
        ExpressionKind::MemberAccess(member_access_expression) => ExpressionKind::MemberAccess(
            Box::new(member_access_expression_with_file(*member_access_expression, file)),
        ),
        ExpressionKind::Cast(cast_expression) => {
            ExpressionKind::Cast(Box::new(cast_expression_with_file(*cast_expression, file)))
        }
        ExpressionKind::Infix(infix_expression) => {
            ExpressionKind::Infix(Box::new(infix_expression_with_file(*infix_expression, file)))
        }
        ExpressionKind::If(if_expression) => {
            ExpressionKind::If(Box::new(if_expression_with_file(*if_expression, file)))
        }
        ExpressionKind::Match(match_expression) => {
            ExpressionKind::Match(Box::new(match_expression_with_file(*match_expression, file)))
        }
        ExpressionKind::Variable(path) => ExpressionKind::Variable(path_with_file(path, file)),
        ExpressionKind::Tuple(expressions) => {
            ExpressionKind::Tuple(expressions_with_file(expressions, file))
        }
        ExpressionKind::Lambda(lambda) => {
            ExpressionKind::Lambda(Box::new(lambda_with_file(*lambda, file)))
        }
        ExpressionKind::Parenthesized(expression) => {
            ExpressionKind::Parenthesized(Box::new(expression_with_file(*expression, file)))
        }
        ExpressionKind::Quote(tokens) => ExpressionKind::Quote(tokens_with_file(tokens, file)),
        ExpressionKind::Unquote(expression) => {
            ExpressionKind::Unquote(Box::new(expression_with_file(*expression, file)))
        }
        ExpressionKind::Comptime(block_expression, location) => ExpressionKind::Comptime(
            block_expression_with_file(block_expression, file),
            location_with_file(location, file),
        ),
        ExpressionKind::Unsafe(block_expression, location) => ExpressionKind::Unsafe(
            block_expression_with_file(block_expression, file),
            location_with_file(location, file),
        ),
        ExpressionKind::AsTraitPath(as_trait_path) => {
            ExpressionKind::AsTraitPath(as_trait_path_with_file(as_trait_path, file))
        }
        ExpressionKind::TypePath(type_path) => {
            ExpressionKind::TypePath(type_path_with_file(type_path, file))
        }
        ExpressionKind::Resolved(..)
        | ExpressionKind::Interned(..)
        | ExpressionKind::InternedStatement(..)
        | ExpressionKind::Error => kind,
    }
}

fn type_path_with_file(type_path: TypePath, file: FileId) -> TypePath {
    TypePath {
        typ: unresolved_type_with_file(type_path.typ, file),
        item: ident_with_file(type_path.item, file),
        turbofish: type_path.turbofish.map(|args| generic_type_args_with_file(args, file)),
    }
}

fn tokens_with_file(tokens: Tokens, file: FileId) -> Tokens {
    Tokens(vecmap(tokens.0, |token| {
        let location = location_with_file(token.to_location(), file);
        LocatedToken::new(token_with_location(token.into_token(), file), location)
    }))
}

fn token_with_location(token: Token, file: FileId) -> Token {
    if let Token::FmtStr(fragments, length) = token {
        Token::FmtStr(
            vecmap(fragments, |fragment| fmt_str_fragment_with_file(fragment, file)),
            length,
        )
    } else {
        token
    }
}

fn fmt_str_fragment_with_file(fragment: FmtStrFragment, file: FileId) -> FmtStrFragment {
    match fragment {
        FmtStrFragment::Interpolation(string, span, _) => {
            FmtStrFragment::Interpolation(string, span, file)
        }
        FmtStrFragment::String(_) => fragment,
    }
}

fn lambda_with_file(lambda: Lambda, file: FileId) -> Lambda {
    Lambda {
        parameters: vecmap(lambda.parameters, |(pattern, typ)| {
            (pattern_with_file(pattern, file), unresolved_type_with_file(typ, file))
        }),
        return_type: unresolved_type_with_file(lambda.return_type, file),
        body: expression_with_file(lambda.body, file),
    }
}

fn match_expression_with_file(expr: MatchExpression, file: FileId) -> MatchExpression {
    MatchExpression {
        expression: expression_with_file(expr.expression, file),
        rules: vecmap(expr.rules, |(condition, body)| {
            (expression_with_file(condition, file), expression_with_file(body, file))
        }),
    }
}

fn if_expression_with_file(expr: IfExpression, file: FileId) -> IfExpression {
    IfExpression {
        condition: expression_with_file(expr.condition, file),
        consequence: expression_with_file(expr.consequence, file),
        alternative: expr.alternative.map(|alternative| expression_with_file(alternative, file)),
    }
}

fn infix_expression_with_file(expr: InfixExpression, file: FileId) -> InfixExpression {
    InfixExpression {
        lhs: expression_with_file(expr.lhs, file),
        rhs: expression_with_file(expr.rhs, file),
        operator: expr.operator,
    }
}

fn cast_expression_with_file(expr: CastExpression, file: FileId) -> CastExpression {
    CastExpression {
        lhs: expression_with_file(expr.lhs, file),
        r#type: unresolved_type_with_file(expr.r#type, file),
    }
}

fn member_access_expression_with_file(
    expr: MemberAccessExpression,
    file: FileId,
) -> MemberAccessExpression {
    MemberAccessExpression {
        lhs: expression_with_file(expr.lhs, file),
        rhs: ident_with_file(expr.rhs, file),
    }
}

fn constructor_expression_with_file(
    expr: ConstructorExpression,
    file: FileId,
) -> ConstructorExpression {
    ConstructorExpression {
        typ: unresolved_type_with_file(expr.typ, file),
        fields: vecmap(expr.fields, |(ident, expression)| {
            (ident_with_file(ident, file), expression_with_file(expression, file))
        }),
        struct_type: expr.struct_type,
    }
}

fn constrain_expression_with_file(expr: ConstrainExpression, file: FileId) -> ConstrainExpression {
    ConstrainExpression {
        kind: expr.kind,
        arguments: expressions_with_file(expr.arguments, file),
        location: location_with_file(expr.location, file),
    }
}

fn method_call_expression_with_file(
    expr: MethodCallExpression,
    file: FileId,
) -> MethodCallExpression {
    MethodCallExpression {
        object: expression_with_file(expr.object, file),
        method_name: ident_with_file(expr.method_name, file),
        generics: expr.generics.map(|generics| unresolved_types_with_file(generics, file)),
        arguments: expressions_with_file(expr.arguments, file),
        is_macro_call: expr.is_macro_call,
    }
}

fn call_expression_with_file(expr: CallExpression, file: FileId) -> CallExpression {
    CallExpression {
        func: Box::new(expression_with_file(*expr.func, file)),
        arguments: expressions_with_file(expr.arguments, file),
        is_macro_call: expr.is_macro_call,
    }
}

fn index_expression_with_file(expr: IndexExpression, file: FileId) -> IndexExpression {
    IndexExpression {
        collection: expression_with_file(expr.collection, file),
        index: expression_with_file(expr.index, file),
    }
}

fn prefix_expression_with_file(expr: PrefixExpression, file: FileId) -> PrefixExpression {
    PrefixExpression { operator: expr.operator, rhs: expression_with_file(expr.rhs, file) }
}

fn literal_with_file(literal: Literal, file: FileId) -> Literal {
    match literal {
        Literal::Array(array_literal) => {
            Literal::Array(array_literal_with_file(array_literal, file))
        }
        Literal::Slice(array_literal) => {
            Literal::Slice(array_literal_with_file(array_literal, file))
        }
        Literal::FmtStr(fragments, length) => Literal::FmtStr(
            vecmap(fragments, |fragment| fmt_str_fragment_with_file(fragment, file)),
            length,
        ),
        Literal::Bool(..)
        | Literal::Integer(..)
        | Literal::Str(..)
        | Literal::RawStr(..)
        | Literal::Unit => literal,
    }
}

fn array_literal_with_file(array_literal: ArrayLiteral, file: FileId) -> ArrayLiteral {
    match array_literal {
        ArrayLiteral::Standard(expressions) => {
            ArrayLiteral::Standard(expressions_with_file(expressions, file))
        }
        ArrayLiteral::Repeated { repeated_element, length } => ArrayLiteral::Repeated {
            repeated_element: Box::new(expression_with_file(*repeated_element, file)),
            length: Box::new(expression_with_file(*length, file)),
        },
    }
}

fn block_expression_with_file(block: BlockExpression, file: FileId) -> BlockExpression {
    BlockExpression { statements: statements_with_file(block.statements, file) }
}

fn statements_with_file(statements: Vec<Statement>, file: FileId) -> Vec<Statement> {
    vecmap(statements, |statement| statement_with_file(statement, file))
}

fn statement_with_file(statement: Statement, file: FileId) -> Statement {
    Statement {
        kind: statement_kind_with_file(statement.kind, file),
        location: location_with_file(statement.location, file),
    }
}

fn statement_kind_with_file(kind: StatementKind, file: FileId) -> StatementKind {
    match kind {
        StatementKind::Let(let_statement) => {
            StatementKind::Let(let_statement_with_file(let_statement, file))
        }
        StatementKind::Expression(expression) => {
            StatementKind::Expression(expression_with_file(expression, file))
        }
        StatementKind::Assign(assign_statement) => {
            StatementKind::Assign(assign_statement_with_file(assign_statement, file))
        }
        StatementKind::For(for_loop_statement) => {
            StatementKind::For(for_loop_statement_with_file(for_loop_statement, file))
        }
        StatementKind::While(while_) => StatementKind::While(WhileStatement {
            condition: expression_with_file(while_.condition, file),
            body: expression_with_file(while_.body, file),
            while_keyword_location: while_.while_keyword_location,
        }),
        StatementKind::Loop(expression, location) => StatementKind::Loop(
            expression_with_file(expression, file),
            location_with_file(location, file),
        ),
        StatementKind::Comptime(statement) => {
            StatementKind::Comptime(Box::new(statement_with_file(*statement, file)))
        }
        StatementKind::Semi(expression) => {
            StatementKind::Semi(expression_with_file(expression, file))
        }
        StatementKind::Interned(..)
        | StatementKind::Break
        | StatementKind::Continue
        | StatementKind::Error => kind,
    }
}

fn for_loop_statement_with_file(for_loop: ForLoopStatement, file: FileId) -> ForLoopStatement {
    ForLoopStatement {
        identifier: ident_with_file(for_loop.identifier, file),
        range: for_range_with_file(for_loop.range, file),
        block: expression_with_file(for_loop.block, file),
        location: location_with_file(for_loop.location, file),
    }
}

fn for_range_with_file(range: ForRange, file: FileId) -> ForRange {
    match range {
        ForRange::Range(for_bounds) => ForRange::Range(for_bounds_with_file(for_bounds, file)),
        ForRange::Array(expression) => ForRange::Array(expression_with_file(expression, file)),
    }
}

fn for_bounds_with_file(for_bounds: ForBounds, file: FileId) -> ForBounds {
    ForBounds {
        start: expression_with_file(for_bounds.start, file),
        end: expression_with_file(for_bounds.end, file),
        inclusive: for_bounds.inclusive,
    }
}

fn assign_statement_with_file(assign: AssignStatement, file: FileId) -> AssignStatement {
    AssignStatement {
        lvalue: lvalue_with_file(assign.lvalue, file),
        expression: expression_with_file(assign.expression, file),
    }
}

fn lvalue_with_file(lvalue: LValue, file: FileId) -> LValue {
    match lvalue {
        LValue::Ident(ident) => LValue::Ident(ident_with_file(ident, file)),
        LValue::MemberAccess { object, field_name, location } => LValue::MemberAccess {
            object: Box::new(lvalue_with_file(*object, file)),
            field_name: ident_with_file(field_name, file),
            location: location_with_file(location, file),
        },
        LValue::Index { array, index, location } => LValue::Index {
            array: Box::new(lvalue_with_file(*array, file)),
            index: expression_with_file(index, file),
            location: location_with_file(location, file),
        },
        LValue::Dereference(lvalue, location) => LValue::Dereference(
            Box::new(lvalue_with_file(*lvalue, file)),
            location_with_file(location, file),
        ),
        LValue::Interned(interned_expression_kind, location) => {
            LValue::Interned(interned_expression_kind, location_with_file(location, file))
        }
    }
}

fn unresolved_generics_with_file(
    generics: Vec<UnresolvedGeneric>,
    file: FileId,
) -> Vec<UnresolvedGeneric> {
    vecmap(generics, |generic| unresolved_generic_with_file(generic, file))
}

fn unresolved_generic_with_file(generic: UnresolvedGeneric, file: FileId) -> UnresolvedGeneric {
    match generic {
        UnresolvedGeneric::Variable(ident) => {
            UnresolvedGeneric::Variable(ident_with_file(ident, file))
        }
        UnresolvedGeneric::Numeric { ident, typ } => UnresolvedGeneric::Numeric {
            ident: ident_with_file(ident, file),
            typ: unresolved_type_with_file(typ, file),
        },
        UnresolvedGeneric::Resolved(quoted_type_id, location) => {
            UnresolvedGeneric::Resolved(quoted_type_id, location_with_file(location, file))
        }
    }
}

fn unresolved_trait_constraints_with_file(
    constraints: Vec<UnresolvedTraitConstraint>,
    file: FileId,
) -> Vec<UnresolvedTraitConstraint> {
    vecmap(constraints, |constraint| unresolved_trait_constraint_with_file(constraint, file))
}

fn unresolved_trait_constraint_with_file(
    constraint: UnresolvedTraitConstraint,
    file: FileId,
) -> UnresolvedTraitConstraint {
    UnresolvedTraitConstraint {
        typ: unresolved_type_with_file(constraint.typ, file),
        trait_bound: trait_bound_with_file(constraint.trait_bound, file),
    }
}

fn trait_bounds_with_file(trait_bounds: Vec<TraitBound>, file: FileId) -> Vec<TraitBound> {
    vecmap(trait_bounds, |bound| trait_bound_with_file(bound, file))
}

fn trait_bound_with_file(trait_bound: TraitBound, file: FileId) -> TraitBound {
    TraitBound {
        trait_path: path_with_file(trait_bound.trait_path, file),
        trait_id: trait_bound.trait_id,
        trait_generics: generic_type_args_with_file(trait_bound.trait_generics, file),
    }
}

fn location_with_file(location: Location, file: FileId) -> Location {
    Location { file, ..location }
}
