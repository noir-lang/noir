use std::meta::type_of;

pub struct Foo {
    x: Field,
    field: Field,
}

#[function_attr]
pub fn foo(
    w: i32,
    y: Field,
    Foo { x, field: some_field }: Foo,
    mut a: bool,
    (b, c): (i32, i32),
) -> i32 {
    let _ = (w, y, x, some_field, a, b, c);
    1
}

#[test]
#[deprecated]
#[check_named_attribute]
fn some_test() {}

#[check_visibility]
pub(crate) fn pub_crate_fn() {}

comptime fn function_attr(f: FunctionDefinition) {
    // Check FunctionDefinition::parameters
    let parameters = f.parameters();
    assert_eq(parameters.len(), 5);

    assert_eq(parameters[0].0, quote { w });
    assert_eq(parameters[1].0, quote { y });
    assert_eq(parameters[2].0, quote { Foo { x, field: some_field } });
    assert_eq(parameters[3].0, quote { mut a });
    assert_eq(parameters[4].0, quote { (b, c) });

    let an_i32: i32 = 0;

    assert_eq(parameters[0].1, type_of(an_i32));
    assert_eq(parameters[1].1, type_of(0));
    assert_eq(parameters[2].1, type_of(Foo { x: 0, field: 1 }));
    assert_eq(parameters[3].1, type_of(true));
    assert_eq(parameters[4].1, type_of((an_i32, an_i32)));

    // Check FunctionDefinition::return_type
    assert_eq(f.return_type(), type_of(an_i32));

    // Check FunctionDefinition::name
    assert_eq(f.name(), quote { foo });

    assert(f.has_named_attribute("function_attr"));

    assert_eq(f.visibility(), quote { pub })
}

comptime fn check_named_attribute(f: FunctionDefinition) {
    assert(f.has_named_attribute("test"));
    assert(f.has_named_attribute("deprecated"));
    assert_eq(f.visibility(), quote { })
}

comptime fn check_visibility(f: FunctionDefinition) {
    assert_eq(f.visibility(), quote { pub(crate) })
}

#[mutate_add_one]
fn add_one() {}

comptime fn mutate_add_one(f: FunctionDefinition) {
    // fn add_one(x: Field)
    assert_eq(f.parameters().len(), 0);
    f.set_parameters([(quote { x }, type_of(0))].as_vector());
    assert_eq(f.parameters().len(), 1);

    // fn add_one(x: Field) -> Field
    assert_eq(f.return_type(), type_of(()));
    f.set_return_type(type_of(0));
    assert_eq(f.return_type(), type_of(0));

    // fn add_one(x: Field) -> Field { x + 1 }
    f.set_body(quote { x + 1 }.as_expr().unwrap());
}

fn main() {
    assert_eq(add_one(41), 42);
    pub_crate_fn();
}

contract some_contract {
    // No pub on the return type is an error
    #[super::set_pub_return]
    pub fn bar() -> Field {
        1
    }
}

comptime fn set_pub_return(f: FunctionDefinition) {
    f.set_return_public(true);
}

mod foo {
    #[attr]
    pub fn some() {}

    comptime fn attr(f: FunctionDefinition) {
        assert_eq(f.module().name(), quote { foo });

        assert(!f.is_unconstrained());

        f.set_unconstrained(true);
        assert(f.is_unconstrained());

        f.set_unconstrained(false);
        assert(!f.is_unconstrained());
    }
}

mod test_as_typed_expr_1 {
    #![foo]

    pub fn method<T>(x: T) -> T {
        x
    }

    comptime fn foo(module: Module) -> Quoted {
        let method = module.functions().filter(|f| f.name() == quote { method })[0];
        let func = method.as_typed_expr();
        quote { 
            pub fn bar() -> i32 {
                $func(1)
            }
        }
    }

    pub fn test() {
        comptime {
            assert_eq(bar(), 1);
        }
    }
}

mod test_as_typed_expr_2 {
    #![foo]

    unconstrained fn method<T, let N: u32>(xs: [T; N]) -> u32 {
        xs.len()
    }

    comptime fn foo(module: Module) -> Quoted {
        let method = module.functions().filter(|f| f.name() == quote { method })[0];
        let func = method.as_typed_expr();
        quote { 
            pub fn bar() -> u32 {
                // Safety: test program
                unsafe { $func([1, 2, 3, 0]) }
            }
        }
    }

    pub fn test() {
        comptime {
            assert_eq(bar(), 4);
        }
    }
}

mod test_as_typed_expr_3 {
    #![foo]

    pub comptime fn method<T, U, let N: u32>(xs_ys: ([T; N], U)) -> u32 {
        let (xs, _ys) = xs_ys;
        xs.len()
    }

    comptime fn foo(module: Module) -> Quoted {
        let method = module.functions().filter(|f| f.name() == quote { method })[0];
        let func = method.as_typed_expr();
        quote { 
            pub fn bar() -> u32 {
                // Safety: test program
                comptime { $func(([1, 2, 3, 0], "a")) }
            }
        }
    }

    pub fn test() {
        comptime {
            assert_eq(bar(), 4);
        }
    }
}

mod test_as_typed_expr_4 {
    comptime fn foo(f: TypedExpr) -> Quoted {
        quote {
        $f()
    }
    }

    fn bar() -> Field {
        1
    }

    fn baz() -> Field {
        let x: Field = comptime {
            let bar_q = quote {
            bar
        };
            foo(bar_q.as_expr().unwrap().resolve(Option::none()))
        };
        x
    }

    pub fn test() {
        comptime {
            assert_eq(baz(), 1);
        }
    }
}

mod test_as_typed_expr_5 {
    trait Trait {}

    impl Trait for i32 {}

    trait Packable<let N: u32> {
        fn pack(self);
    }

    pub struct Foo<T> {}

    impl<T> Packable<10> for Foo<T>
    where
        T: Trait,
    {
        fn pack(self) {}
    }

    fn foo() {
        comptime {
            let foo = quote { Foo<i32> }.as_type();
            let t = quote { Packable<10> }.as_trait_constraint();
            let _ = foo.get_trait_impl(t).unwrap().methods().filter(|method| {
                method.name() == quote { pack }
            })[0]
                .as_typed_expr();
        }
    }
}
