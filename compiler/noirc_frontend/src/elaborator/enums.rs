use acvm::AcirField;
use fxhash::FxHashMap as HashMap;
use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    ast::{EnumVariant, Expression, FunctionKind, NoirEnumeration, UnresolvedType, Visibility},
    hir::resolution::errors::ResolverError,
    hir_def::{
        expr::{
            HirArrayLiteral, HirBlockExpression, HirEnumConstructorExpression, HirExpression,
            HirIdent, HirLiteral, HirMatchExpression,
        },
        function::{FuncMeta, FunctionBody, HirFunction, Parameters},
        stmt::{HirLetStatement, HirPattern, HirStatement},
    },
    node_interner::{DefinitionId, DefinitionKind, ExprId, FunctionModifiers, GlobalValue, TypeId},
    token::Attributes,
    DataType, Shared, Type,
};

use super::Elaborator;

impl Elaborator<'_> {
    /// Defines the value of an enum variant that we resolve an enum
    /// variant expression to. E.g. `Foo::Bar` in `Foo::Bar(baz)`.
    ///
    /// If the variant requires arguments we should define a function,
    /// otherwise we define a polymorphic global containing the tag value.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn define_enum_variant_constructor(
        &mut self,
        enum_: &NoirEnumeration,
        type_id: TypeId,
        variant: &EnumVariant,
        variant_arg_types: Option<Vec<Type>>,
        variant_index: usize,
        datatype: &Shared<DataType>,
        self_type: &Type,
        self_type_unresolved: UnresolvedType,
    ) {
        match variant_arg_types {
            Some(args) => self.define_enum_variant_function(
                enum_,
                type_id,
                variant,
                args,
                variant_index,
                datatype,
                self_type,
                self_type_unresolved,
            ),
            None => self.define_enum_variant_global(
                enum_,
                type_id,
                variant,
                variant_index,
                datatype,
                self_type,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn define_enum_variant_global(
        &mut self,
        enum_: &NoirEnumeration,
        type_id: TypeId,
        variant: &EnumVariant,
        variant_index: usize,
        datatype: &Shared<DataType>,
        self_type: &Type,
    ) {
        let name = &variant.name;
        let location = Location::new(variant.name.span(), self.file);

        let global_id = self.interner.push_empty_global(
            name.clone(),
            type_id.local_module_id(),
            type_id.krate(),
            self.file,
            Vec::new(),
            false,
            false,
        );

        let mut typ = self_type.clone();
        if !datatype.borrow().generics.is_empty() {
            let typevars = vecmap(&datatype.borrow().generics, |generic| generic.type_var.clone());
            typ = Type::Forall(typevars, Box::new(typ));
        }

        let definition_id = self.interner.get_global(global_id).definition_id;
        self.interner.push_definition_type(definition_id, typ.clone());

        let no_parameters = Parameters(Vec::new());
        let global_body =
            self.make_enum_variant_constructor(datatype, variant_index, &no_parameters, location);
        let let_statement = crate::hir_def::stmt::HirStatement::Expression(global_body);

        let statement_id = self.interner.get_global(global_id).let_statement;
        self.interner.replace_statement(statement_id, let_statement);

        self.interner.get_global_mut(global_id).value = GlobalValue::Resolved(
            crate::hir::comptime::Value::Enum(variant_index, Vec::new(), typ),
        );

        Self::get_module_mut(self.def_maps, type_id.module_id())
            .declare_global(name.clone(), enum_.visibility, global_id)
            .ok();
    }

    #[allow(clippy::too_many_arguments)]
    fn define_enum_variant_function(
        &mut self,
        enum_: &NoirEnumeration,
        type_id: TypeId,
        variant: &EnumVariant,
        variant_arg_types: Vec<Type>,
        variant_index: usize,
        datatype: &Shared<DataType>,
        self_type: &Type,
        self_type_unresolved: UnresolvedType,
    ) {
        let name_string = variant.name.to_string();
        let datatype_ref = datatype.borrow();
        let location = Location::new(variant.name.span(), self.file);

        let id = self.interner.push_empty_fn();

        let modifiers = FunctionModifiers {
            name: name_string.clone(),
            visibility: enum_.visibility,
            attributes: Attributes { function: None, secondary: Vec::new() },
            is_unconstrained: false,
            generic_count: datatype_ref.generics.len(),
            is_comptime: false,
            name_location: location,
        };
        let definition_id =
            self.interner.push_function_definition(id, modifiers, type_id.module_id(), location);

        let hir_name = HirIdent::non_trait_method(definition_id, location);
        let parameters = self.make_enum_variant_parameters(variant_arg_types, location);

        let body =
            self.make_enum_variant_constructor(datatype, variant_index, &parameters, location);
        self.interner.update_fn(id, HirFunction::unchecked_from_expr(body));

        let function_type =
            datatype_ref.variant_function_type_with_forall(variant_index, datatype.clone());
        self.interner.push_definition_type(definition_id, function_type.clone());

        let meta = FuncMeta {
            name: hir_name,
            kind: FunctionKind::Normal,
            parameters,
            parameter_idents: Vec::new(),
            return_type: crate::ast::FunctionReturnType::Ty(self_type_unresolved),
            return_visibility: Visibility::Private,
            typ: function_type,
            direct_generics: datatype_ref.generics.clone(),
            all_generics: datatype_ref.generics.clone(),
            location,
            has_body: false,
            trait_constraints: Vec::new(),
            type_id: Some(type_id),
            trait_id: None,
            trait_impl: None,
            enum_variant_index: Some(variant_index),
            is_entry_point: false,
            has_inline_attribute: false,
            function_body: FunctionBody::Resolved,
            source_crate: self.crate_id,
            source_module: type_id.local_module_id(),
            source_file: self.file,
            self_type: None,
        };

        self.interner.push_fn_meta(meta, id);
        self.interner.add_method(self_type, name_string, id, None);

        let name = variant.name.clone();
        Self::get_module_mut(self.def_maps, type_id.module_id())
            .declare_function(name, enum_.visibility, id)
            .ok();
    }

    // Given:
    // ```
    // enum FooEnum { Foo(u32, u8), ... }
    //
    // fn Foo(a: u32, b: u8) -> FooEnum {}
    // ```
    // Create (pseudocode):
    // ```
    // fn Foo(a: u32, b: u8) -> FooEnum {
    //     // This can't actually be written directly in Noir
    //     FooEnum {
    //         tag: Foo_tag,
    //         Foo: (a, b),
    //         // fields from other variants are zeroed in monomorphization
    //     }
    // }
    // ```
    fn make_enum_variant_constructor(
        &mut self,
        self_type: &Shared<DataType>,
        variant_index: usize,
        parameters: &Parameters,
        location: Location,
    ) -> ExprId {
        // Each parameter of the enum variant function is used as a parameter of the enum
        // constructor expression
        let arguments = vecmap(&parameters.0, |(pattern, typ, _)| match pattern {
            HirPattern::Identifier(ident) => {
                let id = self.interner.push_expr(HirExpression::Ident(ident.clone(), None));
                self.interner.push_expr_type(id, typ.clone());
                self.interner.push_expr_location(id, location.span, location.file);
                id
            }
            _ => unreachable!(),
        });

        let constructor = HirExpression::EnumConstructor(HirEnumConstructorExpression {
            r#type: self_type.clone(),
            arguments,
            variant_index,
        });

        let body = self.interner.push_expr(constructor);
        let enum_generics = self_type.borrow().generic_types();
        let typ = Type::DataType(self_type.clone(), enum_generics);
        self.interner.push_expr_type(body, typ);
        self.interner.push_expr_location(body, location.span, location.file);
        body
    }

    fn make_enum_variant_parameters(
        &mut self,
        parameter_types: Vec<Type>,
        location: Location,
    ) -> Parameters {
        Parameters(vecmap(parameter_types.into_iter().enumerate(), |(i, parameter_type)| {
            let name = format!("${i}");
            let parameter = DefinitionKind::Local(None);
            let id = self.interner.push_definition(name, false, false, parameter, location);
            let pattern = HirPattern::Identifier(HirIdent::non_trait_method(id, location));
            (pattern, parameter_type, Visibility::Private)
        }))
    }

    /// Compiles the rows of a match expression, outputting a decision tree for the match.
    ///
    /// This is an adaptation of https://github.com/yorickpeterse/pattern-matching-in-rust/tree/main/jacobs2021
    /// which is an implementation of https://julesjacobs.com/notes/patternmatching/patternmatching.pdf
    fn elaborate_match_rows(&mut self, mut rows: Vec<Row>) -> Result<Decision, ResolverError> {
        if rows.is_empty() {
            return Err(todo!("missing case"));
        }

        self.push_tests_against_bare_variables(&mut rows);

        // If the first row is a match-all we match it and the remaining rows are ignored.
        if rows.first().map_or(false, |row| row.columns.is_empty()) {
            let row = rows.remove(0);

            return Ok(match row.guard {
                None => Decision::Success(row.body),
                Some(expr) => {
                    let remaining = self.elaborate_match_rows(rows)?;
                    Decision::Guard(expr, row.body, Box::new(remaining))
                }
            });
        }

        let branch_var = self.branch_variable(&rows);

        match self.interner.definition_type(branch_var) {
            Type::FieldElement | Type::Integer(_, _) => {
                let (cases, fallback) = self.compile_int_cases(rows, branch_var);

                Ok(Decision::Switch(branch_var, cases, Some(fallback)))
            }

            Type::Array(_, _) => todo!(),
            Type::Slice(_) => todo!(),
            Type::Bool => todo!(),
            Type::String(_) => todo!(),
            Type::FmtString(_, _) => todo!(),
            Type::Unit => todo!(),
            Type::Tuple(_) => todo!(),
            Type::DataType(_, _) => todo!(),
            Type::Alias(_, _) => todo!(),
            Type::TypeVariable(_) => todo!(),
            Type::TraitAsType(_, _, _) => todo!(),
            Type::NamedGeneric(_, _) => todo!(),
            Type::CheckedCast { from, to } => todo!(),
            Type::Function(_, _, _, _) => todo!(),
            Type::MutableReference(_) => todo!(),
            Type::Forall(_, _) => todo!(),
            Type::Constant(_, _) => todo!(),
            Type::Quoted(_) => todo!(),
            Type::InfixExpr(_, _, _, _) => todo!(),
            Type::Error => todo!(),
        }
    }

    /// Compiles the cases and fallback cases for integer and range patterns.
    ///
    /// Integers have an infinite number of constructors, so we specialise the
    /// compilation of integer and range patterns.
    fn compile_int_cases(
        &mut self,
        rows: Vec<Row>,
        branch_var: Variable,
    ) -> (Vec<Case>, Box<Decision>) {
        let mut raw_cases: Vec<(Constructor, Vec<Variable>, Vec<Row>)> = Vec::new();
        let mut fallback_rows = Vec::new();
        let mut tested: HashMap<(i64, i64), usize> = HashMap::new();

        for mut row in rows {
            if let Some(col) = row.remove_column(&branch_var) {
                let (key, cons) = match col.pattern {
                    Pattern::Int(val) => ((val, val), Constructor::Int(val)),
                    Pattern::Range(start, stop) => ((start, stop), Constructor::Range(start, stop)),
                    _ => unreachable!(),
                };

                if let Some(index) = tested.get(&key) {
                    raw_cases[*index].2.push(row);
                    continue;
                }

                tested.insert(key, raw_cases.len());

                let mut rows = fallback_rows.clone();

                rows.push(row);
                raw_cases.push((cons, Vec::new(), rows));
            } else {
                for (_, _, rows) in &mut raw_cases {
                    rows.push(row.clone());
                }

                fallback_rows.push(row);
            }
        }

        let cases = raw_cases
            .into_iter()
            .map(|(cons, vars, rows)| Case::new(cons, vars, self.compile_rows(rows)))
            .collect();

        (cases, Box::new(self.compile_rows(fallback_rows)))
    }

    /// Compiles the cases and sub cases for the constructor located at the
    /// column of the branching variable.
    ///
    /// What exactly this method does may be a bit hard to understand from the
    /// code, as there's simply quite a bit going on. Roughly speaking, it does
    /// the following:
    ///
    /// 1. It takes the column we're branching on (based on the branching
    ///    variable) and removes it from every row.
    /// 2. We add additional columns to this row, if the constructor takes any
    ///    arguments (which we'll handle in a nested match).
    /// 3. We turn the resulting list of rows into a list of cases, then compile
    ///    those into decision (sub) trees.
    ///
    /// If a row didn't include the branching variable, we simply copy that row
    /// into the list of rows for every constructor to test.
    ///
    /// For this to work, the `cases` variable must be prepared such that it has
    /// a triple for every constructor we need to handle. For an ADT with 10
    /// constructors, that means 10 triples. This is needed so this method can
    /// assign the correct sub matches to these constructors.
    ///
    /// Types with infinite constructors (e.g. int and string) are handled
    /// separately; they don't need most of this work anyway.
    fn compile_constructor_cases(
        &mut self,
        rows: Vec<Row>,
        branch_var: DefinitionId,
        mut cases: Vec<(Pattern, Vec<DefinitionId>, Vec<Row>)>,
    ) -> Vec<Case> {
        for mut row in rows {
            if let Some(col) = row.remove_column(branch_var) {
                if let Pattern::Constructor(cons, args) = col.pattern {
                    let idx = cons.variant_index();
                    let mut cols = row.columns;

                    for (var, pat) in cases[idx].1.iter().zip(args.into_iter()) {
                        cols.push(Column::new(*var, pat));
                    }

                    cases[idx].2.push(Row::new(cols, row.guard, row.body));
                }
            } else {
                for (_, _, rows) in &mut cases {
                    rows.push(row.clone());
                }
            }
        }

        cases
            .into_iter()
            .map(|(cons, vars, rows)| Case::new(cons, vars, self.compile_rows(rows)))
            .collect()
    }

    /// Return the variable that was referred to the most in `rows`
    fn branch_variable(&mut self, rows: &[Row]) -> DefinitionId {
        let mut counts = HashMap::default();

        for row in rows {
            for col in &row.columns {
                *counts.entry(&col.variable_to_match).or_insert(0_usize) += 1
            }
        }

        rows[0]
            .columns
            .iter()
            .map(|col| col.variable_to_match)
            .max_by_key(|var| counts[var])
            .unwrap()
    }

    fn push_tests_against_bare_variables(&self, rows: &mut Vec<Row>) {
        for row in rows {
            row.columns.retain(|col| {
                if let Pattern::Binding(variable) = col.pattern {
                    row.body = self.let_binding(variable, col.variable_to_match, row.body);
                    false
                } else {
                    true
                }
            });
        }
    }

    /// Creates:
    /// `{ let <variable> = <rhs>; <body> }`
    fn let_binding(&mut self, variable: DefinitionId, rhs: DefinitionId, body: ExprId) -> ExprId {
        let location = self.interner.definition(rhs).location;

        let r#type = self.interner.definition_type(variable);
        let variable = HirIdent::non_trait_method(variable, location);

        // TODO: push locs and types
        let rhs = HirExpression::Ident(HirIdent::non_trait_method(rhs, location), None);
        let rhs = self.interner.push_expr(rhs);

        let let_ = HirStatement::Let(HirLetStatement {
            pattern: HirPattern::Identifier(variable),
            r#type,
            expression: rhs,
            attributes: Vec::new(),
            comptime: false,
            is_global_let: false,
        });

        let let_ = self.interner.push_stmt(let_);
        let body = self.interner.push_stmt(HirStatement::Expression(body));

        let block = HirExpression::Block(HirBlockExpression { statements: vec![let_, body] });
        self.interner.push_expr(block)
    }
}

type MatchRules = Vec<(ExprId, Expression)>;

/// Patterns are represented as resolved expressions currently.
/// This type alias just makes code involving them more clear.
enum Pattern {
    /// A pattern such as `Some(42)`.
    Constructor(Constructor, Vec<Pattern>),
    Int(i64),
    Binding(DefinitionId),
    Or(Vec<Pattern>),
    Range(i64, i64),
}

enum Constructor {
    True,
    False,
    Int(i64),
    Tuple(Vec<Type>),
    Variant(TypeId, usize),
    Range(i64, i64),
}

impl Constructor {
    fn variant_index(&self) -> usize {
        match self {
            Constructor::False
            | Constructor::Int(_)
            | Constructor::Tuple(_)
            | Constructor::Range(_, _) => 0,
            Constructor::True => 1,
            Constructor::Variant(_, index) => *index,
        }
    }
}

/// The RHS/branch of a `pattern -> branch` rule.
type Body = ExprId;

struct Column {
    variable_to_match: DefinitionId,
    pattern: Pattern,
}

struct Row {
    columns: Vec<Column>,
    guard: Option<ExprId>,
    body: ExprId,
}

impl Row {
    fn remove_column(&mut self, variable: DefinitionId) -> Option<Column> {
        self.columns
            .iter()
            .position(|c| c.variable_to_match == variable)
            .map(|idx| self.columns.remove(idx))
    }
}

struct Case {
    constructor: Constructor,

    arguments: Vec<DefinitionId>,

    body: Decision,
}

enum Decision {
    Success(Body),

    Failure,

    /// Run `Body` if the given expression is true.
    /// Otherwise continue with the given decision tree.
    Guard(ExprId, Body, Box<Decision>),

    /// Switch on the given variable with the given cases to test.
    /// The final argument is an optional match-all case to take if
    /// none of the cases matched.
    Switch(DefinitionId, Vec<Case>, Option<Box<Decision>>),
}
