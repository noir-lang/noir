use std::{collections::{HashMap, BTreeMap}, rc::Rc};

use crate::{node_interner::{NodeInterner, self}, hir_def::{expr::*, function::Parameters, stmt::HirPattern}, TypeBinding, util::vecmap};

use self::ast::DefinitionId;

pub mod ast;

struct Monomorphiser {
    // Store monomorphised globals and locals separately,
    // only locals are cleared on each function call.
    globals: Definitions,
    locals: Definitions,

    /// Queue of functions to monomorphise next
    queue: Vec<(node_interner::FuncId, ast::Type)>,

    interner: NodeInterner,

    next_unique_id: u32,
}

// Conceptually the same as a HashMap<(node_interner::DefinitionId, HirType), Value>,
// but the nested hashmaps let us avoid cloning the HirType when calling .get()
type Definitions = HashMap<node_interner::DefinitionId, HashMap<HirType, Value>>;

type Definition = Rc<ast::Expression>;

type HirType = crate::Type;

enum Value {
    One(ast::DefinitionId),
    Many(Vec<Value>),
}

pub fn monomorphise(main: node_interner::FuncId, interner: NodeInterner) -> ast::Function {
    todo!()
}

impl Monomorphiser {
    fn next_definition_id(&mut self) -> DefinitionId {
        let id = self.next_unique_id;
        self.next_unique_id += 1;
        DefinitionId(id)
    }

    fn next_function_id(&mut self) -> ast::FuncId {
        let id = self.next_unique_id;
        self.next_unique_id += 1;
        ast::FuncId(id)
    }

    fn lookup(&mut self, id: node_interner::DefinitionId, typ: &HirType) -> Option<&Value> {
        if let Some(inner_map) = self.locals.get(&id) {
            if let Some(value) = inner_map.get(typ) {
                return Some(value);
            }
        }
        if let Some(inner_map) = self.globals.get(&id) {
            if let Some(value) = inner_map.get(typ) {
                return Some(value);
            }
        }
        None
    }

    fn define_local(&mut self, id: node_interner::DefinitionId, typ: HirType, value: Value) {
        self.locals.entry(id).or_default().insert(typ, value);
    }

    fn define_global(&mut self, id: node_interner::DefinitionId, typ: HirType, value: Value) {
        self.globals.entry(id).or_default().insert(typ, value);
    }

    fn function(&mut self, f: node_interner::FuncId) -> ast::Function {
        let meta = self.interner.function_meta(&f);
        let id = self.next_function_id();
        let parameters = self.parameters(meta.parameters);

        // TODO: Remove. Type should be determined by function callsite
        let return_type = meta.return_type();

        let body = self.expr(*self.interner.function(&f).as_expr(), return_type);

        ast::Function { id, parameters, body }
    }

    /// In addition to monomorphising parameters to concrete, non-generic types,
    /// we must also spread parameters to split struct/tuple parameters into one
    /// parameter for each field.
    fn parameters(&mut self, params: Parameters) -> Vec<(ast::DefinitionId, ast::Type)> {
        let mut new_params = Vec::with_capacity(params.len());
        for parameter in params {
            self.parameter(parameter.0, &parameter.1, &mut new_params);
        }
        new_params
    }

    fn parameter(&mut self, param: HirPattern, typ: &HirType, new_params: &mut Vec<(ast::DefinitionId, ast::Type)>) {
        match param {
            HirPattern::Identifier(ident) => {
                let value = self.expand_parameter(typ, new_params);
                self.define_local(ident.id, typ.clone(), value);
            },
            HirPattern::Mutable(pattern, _) => self.parameter(*pattern, typ, new_params),
            HirPattern::Tuple(fields, _) => {
                let tuple_field_types = unwrap_tuple_type(typ);

                for (field, typ) in fields.into_iter().zip(tuple_field_types) {
                    self.parameter(field, typ, new_params);
                }
            },
            HirPattern::Struct(_, fields, _) => {
                let struct_field_types = unwrap_struct_type(typ);

                for (name, field) in fields {
                    let typ = &struct_field_types[&name.0.contents];
                    self.parameter(field, typ, new_params);
                }
            },
        }
    }

    /// Expand a tuple or struct parameter into one fresh parameter for each field
    fn expand_parameter(&mut self, typ: &HirType, new_params: &mut Vec<(ast::DefinitionId, ast::Type)>) -> Value {
        match typ {
            HirType::Tuple(fields) => {
                Value::Many(vecmap(fields, |field| {
                    self.expand_parameter(field, new_params)
                }))
            },

            HirType::Struct(def, args) => {
                let fields = def.borrow().get_fields(args);
                Value::Many(vecmap(fields, |(_, field)| {
                    self.expand_parameter(&field, new_params)
                }))
            },

            // Must also expand arrays of tuples/structs
            HirType::Array(size, element) => {
                let size = size.array_length().unwrap();
                let initial_len = new_params.len();

                // Keep the same Value structure, each Value::One within is
                // reinterpreted as an array of values instead of a single value.
                // [(a, b)] -> ([a], [b])
                let ret = self.expand_parameter(element, new_params);

                for (_, param_type) in new_params.iter_mut().skip(initial_len) {
                    *param_type = ast::Type::Array(size, Box::new(*param_type));
                }

                ret
            },

            HirType::PolymorphicInteger(_, binding) => {
                match &*binding.borrow() {
                    TypeBinding::Bound(binding) => self.expand_parameter(binding, new_params),
                    TypeBinding::Unbound(_) => todo!("Default integer type"),
                }
            },

            HirType::TypeVariable(binding) => {
                match &*binding.borrow() {
                    TypeBinding::Bound(binding) => self.expand_parameter(binding, new_params),
                    TypeBinding::Unbound(_) => todo!("Default type variable type"),
                }
            }

            HirType::Function(_, _, _) => todo!("Higher order functions"),

            HirType::FieldElement(_)
            | HirType::Unit
            | HirType::Bool(_)
            | HirType::Integer(_, _, _) => {
                let id = self.next_definition_id();
                new_params.push((id, self.convert_type_single(typ)));
                Value::One(self.next_definition_id())
            },

            HirType::Forall(_, _)
            | HirType::ArrayLength(_)
            | HirType::Error => unreachable!(),
        }
    }

    fn expr(&mut self, expr: node_interner::ExprId, typ: &HirType) -> ast::Expression {
        match self.interner.expression(&expr) {
            HirExpression::Ident(ident) => self.ident(ident, typ),
            HirExpression::Literal(_) => todo!(),
            HirExpression::Block(_) => todo!(),
            HirExpression::Prefix(_) => todo!(),
            HirExpression::Infix(_) => todo!(),
            HirExpression::Index(_) => todo!(),
            HirExpression::MemberAccess(_) => todo!(),
            HirExpression::Call(_) => todo!(),
            HirExpression::Cast(_) => todo!(),
            HirExpression::For(_) => todo!(),
            HirExpression::If(_) => todo!(),

            HirExpression::Tuple(fields) => {

            },
            HirExpression::Constructor(_) => todo!(),

            HirExpression::MethodCall(_)
            | HirExpression::Error => unreachable!(),
        }
    }

    fn ident(&mut self, ident: HirIdent, typ: &HirType) -> ast::Expression {
        let value = self.lookup(ident.id, typ).unwrap_or_else(|| {
            todo!()
        });

        let location = ident.location;
        // ast::Ident { location, id, definition }
        todo!()
    }

    /// Convert a non-tuple/struct type to a monomorphised type
    fn convert_type_single(&self, typ: &crate::Type) -> ast::Type {
        match typ {
            HirType::FieldElement(_) => ast::Type::Field,
            HirType::Integer(_, sign, bits) => ast::Type::Integer(*sign, *bits),
            HirType::Bool(_) => ast::Type::Bool,
            HirType::Unit => ast::Type::Unit,

            HirType::Array(_, _) => todo!(),

            HirType::PolymorphicInteger(_, binding)
            | HirType::TypeVariable(binding) => {
                match &*binding.borrow() {
                    TypeBinding::Bound(binding) => self.convert_type_single(binding),
                    TypeBinding::Unbound(_) => unreachable!(),
                }
            },

            HirType::Struct(_, _)
            | HirType::Tuple(_)
            | HirType::Function(_, _, _)
            | HirType::Forall(_, _)
            | HirType::ArrayLength(_)
            | HirType::Error => unreachable!(),
        }
    }
}

fn unwrap_tuple_type(typ: &HirType) -> &[HirType] {
    match typ {
        HirType::Tuple(fields) => fields,
        HirType::TypeVariable(binding) => {
            match &*binding.borrow() {
                TypeBinding::Bound(binding) => unwrap_tuple_type(binding),
                TypeBinding::Unbound(_) => unreachable!(),
            }
        },
        _ => unreachable!(),
    }
}

fn unwrap_struct_type(typ: &HirType) -> BTreeMap<String, HirType> {
    match typ {
        HirType::Struct(def, args) => {
            def.borrow().get_fields(args)
        },
        HirType::TypeVariable(binding) => {
            match &*binding.borrow() {
                TypeBinding::Bound(binding) => unwrap_struct_type(binding),
                TypeBinding::Unbound(_) => unreachable!(),
            }
        },
        _ => unreachable!(),
    }
}
