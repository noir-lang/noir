//! Implement a post-monomorphization pass where builtin/intrinsic/oracle functions used as values
//! are wrapped in a proxy function which replaces them as a value and forwards calls to them.
//!
//! The reason this exists is because by the time we get to the `defunctionalize` SSA pass it would
//! not be possible to deal with function pointers to foreign functions, because unlike for normal
//! functions, we don't have type information for them in the SSA. That is, we can't tell what their
//! parameter and return types are when they are just passed around as a value.
//!
//! The type information is still available in the monomorphized AST though, so we can find all
//! instances where a foreign function is referenced by an [`Ident`](crate::monomorphization::ast::Expression::Ident)
//! without actually being the target of a [`Call`](crate::monomorphization::ast::Expression::Call),
//! and replace them with a normal function, which will preserve the information we need create
//! dispatch functions for them in the `defunctionalize` pass.

use std::collections::HashMap;

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    hir_def::function::FunctionSignature,
    monomorphization::{
        ast::{
            Call, Definition, Expression, FuncId, Function, Ident, IdentId, InlineType, LocalId,
            Program, Type,
        },
        visitor::visit_expr_mut,
    },
    shared::Visibility,
};

impl Program {
    /// Create proxies for foreign functions used as values.
    ///
    /// See the [proxies](crate::monomorphization::proxies) for details.
    ///
    /// This should only be called once, before converting to SSA.
    pub fn create_foreign_proxies(mut self) -> Self {
        let mut context = ProxyContext::new(self.functions.len() as u32);

        // Replace foreign function identifier definitions with proxy function IDs.
        for function in self.functions.iter_mut() {
            context.visit_expr(&mut function.body);
        }

        // Add new functions.
        self.functions.extend(context.into_proxies());

        self
    }
}

struct ProxyContext {
    next_func_id: u32,
    replacements: HashMap<(Definition, /*unconstrained*/ bool), FuncId>,
    proxies: Vec<(FuncId, (Ident, /*unconstrained*/ bool))>,
}

impl ProxyContext {
    fn new(next_func_id: u32) -> Self {
        Self { next_func_id, replacements: HashMap::new(), proxies: Vec::new() }
    }

    fn next_func_id(&mut self) -> FuncId {
        let id = self.next_func_id;
        self.next_func_id += 1;
        FuncId(id)
    }

    /// Visit expressions and replace foreign function identifier definitions with newly allocated
    /// function IDs, collecting the type information along the way so that we can create those
    /// proxies after visiting all functions.
    fn visit_expr(&mut self, expr: &mut Expression) {
        visit_expr_mut(expr, &mut |expr| {
            // Note that if we see a function in `Call::func` then it will be an `Ident`, not a `Tuple`,
            // even though its `Ident::typ` will be a `Tuple([Function, Function])`, but since we only
            // handle tuples, we don't have to skip the `Call::func` to leave it in tact.

            // If this is a foreign function value, we want to replace it with proxies.
            let Some(mut pair) = ForeignFunctionValue::try_from(expr) else {
                return true;
            };

            // Create a separate proxy for the constrained and unconstrained version.
            pair.for_each(|ident, unconstrained| {
                let key = (ident.definition.clone(), unconstrained);

                let proxy_id = match self.replacements.get(&key) {
                    Some(id) => *id,
                    None => {
                        let func_id = self.next_func_id();
                        self.replacements.insert(key, func_id);
                        self.proxies.push((func_id, (ident.clone(), unconstrained)));
                        func_id
                    }
                };

                ident.definition = Definition::Function(proxy_id);
            });

            true
        });
    }

    /// Create proxy functions for the foreign function values we discovered.
    fn into_proxies(self) -> impl IntoIterator<Item = Function> {
        self.proxies
            .into_iter()
            .map(|(id, (ident, unconstrained))| make_proxy(id, ident, unconstrained))
    }
}

/// When function values are passed around in the monomorphized AST,
/// they appear as a pair (tuple) of a constrained and unconstrained
/// function.
struct ForeignFunctionValue<'a> {
    items: &'a mut Vec<Expression>,
}

impl<'a> ForeignFunctionValue<'a> {
    /// Check if we have a pair of identifiers of foreign functions with
    /// the same name, and return both `Ident`s for modification.
    fn try_from(expr: &'a mut Expression) -> Option<Self> {
        let Expression::Tuple(items) = expr else {
            return None;
        };
        if items.len() != 2 {
            return None;
        }
        let Expression::Ident(c) = &items[0] else {
            return None;
        };
        let Expression::Ident(u) = &items[1] else {
            return None;
        };
        if c.definition != u.definition
            || !is_foreign_func(&c.definition)
            || !is_foreign_func(&u.definition)
            || !is_func_pair(&c.typ)
            || !is_func_pair(&u.typ)
        {
            return None;
        }
        Some(Self { items })
    }

    /// Apply a function on the constrained and unconstrained identifier.
    fn for_each(&mut self, mut f: impl FnMut(&mut Ident, bool)) {
        let Expression::Ident(c) = &mut self.items[0] else { unreachable!() };
        f(c, false);
        let Expression::Ident(u) = &mut self.items[1] else { unreachable!() };
        f(u, true);
    }
}

/// Check if the definition is that of a function defined by a "name" rather than an ID.
fn is_foreign_func(definition: &Definition) -> bool {
    matches!(definition, Definition::Builtin(_) | Definition::LowLevel(_) | Definition::Oracle(_))
}

/// Check that the identifier is of a pair of constrained and unconstrained function types.
fn is_func_pair(typ: &Type) -> bool {
    let Type::Tuple(types) = typ else {
        return false;
    };
    types.len() == 2
        && matches!(types[0], Type::Function(_, _, _, false))
        && matches!(types[1], Type::Function(_, _, _, true))
}

/// Create a proxy function definition for a foreign function based on an identifier that got replaced.
///
/// The body of the function will be a single forwarding call to the original.
fn make_proxy(id: FuncId, ident: Ident, unconstrained: bool) -> Function {
    let Type::Tuple(items) = &ident.typ else {
        unreachable!("ICE: expected pair of functions; got {}", ident.typ);
    };

    // Pick the version of the function that we need to forward to.
    let func_idx = if unconstrained { 1 } else { 0 };
    let func_typ = items[func_idx].clone();
    let Type::Function(args, ret, _env, _) = func_typ else {
        unreachable!("ICE: expected function type; got {}", ident.typ);
    };

    let name = format!("{}_proxy", ident.name);

    let mut next_ident_id = 0u32;
    let mut next_ident_id = || {
        let id = next_ident_id;
        next_ident_id += 1;
        IdentId(id)
    };

    let parameters = vecmap(args.into_iter().enumerate(), |(i, typ)| {
        let id = LocalId(i as u32);
        let mutable = false;
        let name = format!("p{i}");
        let vis = Visibility::Private;
        (id, mutable, name, typ, vis)
    });

    // The function signature only matters for entry points.
    let func_sig = FunctionSignature::default();

    let call = {
        let func = Ident {
            id: next_ident_id(),
            location: None,
            definition: ident.definition,
            mutable: ident.mutable,
            name: ident.name,
            // The ident type still carries both function types in its definition.
            typ: ident.typ,
        };

        let arguments = vecmap(&parameters, |(id, mutable, name, typ, _)| {
            let parameter_ident = Ident {
                location: None,
                definition: Definition::Local(*id),
                mutable: *mutable,
                name: name.clone(),
                typ: typ.clone(),
                id: next_ident_id(),
            };
            Expression::Ident(parameter_ident)
        });

        Call {
            func: Box::new(Expression::Ident(func)),
            arguments,
            return_type: *ret.clone(),
            location: Location::dummy(),
        }
    };

    Function {
        id,
        name,
        parameters,
        body: Expression::Call(call),
        return_type: *ret,
        return_visibility: Visibility::Private,
        unconstrained,
        inline_type: InlineType::InlineAlways,
        func_sig,
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::get_monomorphized_no_emit_test;

    #[test]
    fn creates_proxies_for_oracle() {
        let src = "
        unconstrained fn main() {
            foo(bar);
        }

        unconstrained fn foo(f: unconstrained fn(Field) -> ()) {
          f(0);
        }

        #[oracle(my_oracle)]
        unconstrained fn bar(f: Field) {
        }
        ";

        let program = get_monomorphized_no_emit_test(src).unwrap();
        insta::assert_snapshot!(program, @r"
        unconstrained fn main$f0() -> () {
            foo$f1((bar$f2, bar$f3));
        }
        unconstrained fn foo$f1(f$l0: (fn(Field) -> (), unconstrained fn(Field) -> ())) -> () {
            f$l0.1(0);
        }
        #[inline_always]
        fn bar_proxy$f2(p0$l0: Field) -> () {
            bar$my_oracle(p0$l0)
        }
        #[inline_always]
        unconstrained fn bar_proxy$f3(p0$l0: Field) -> () {
            bar$my_oracle(p0$l0)
        }
        ");
    }
}
