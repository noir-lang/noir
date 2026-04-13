use crate::tests::{
    assert_no_errors, assert_no_errors_using_features, check_errors, check_errors_using_features,
    check_monomorphization_error,
};

#[test]
fn cannot_mutate_immutable_variable() {
    let src = r#"
    fn main() {
        let array = [1];
        mutate(&mut array);
                    ^^^^^ Cannot mutate immutable variable `array`
    }

    fn mutate(_: &mut [Field; 1]) {}
    "#;
    check_errors(src);
}

#[test]
fn cannot_mutate_immutable_variable_on_member_access() {
    let src = r#"
    struct Foo {
        x: Field
    }

    fn main() {
        let foo = Foo { x: 0 };
        mutate(&mut foo.x);
                    ^^^^^ Cannot mutate immutable variable `foo`
    }

    fn mutate(foo: &mut Field) {
        *foo = 1;
    }
    "#;
    check_errors(src);
}

#[test]
fn mutable_reference_to_field_of_mutable_reference() {
    let src = r#"
    struct Foo {
        x: Field
    }
    
    fn main() {
        let mut foo = Foo { x: 5 };
        let ref_foo = &mut foo;
        let ref_x = &mut ref_foo.x;
        *ref_x = 10;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn auto_dereferences_array_access() {
    let src = r#"
    fn main() {
        let ref_array = &mut &mut &mut [0, 1, 2];
        assert(ref_array[2] == 2);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_crash_when_passing_mutable_undefined_variable() {
    let src = r#"
    fn main() {
        mutate(&mut undefined);
                    ^^^^^^^^^ cannot find `undefined` in this scope
                    ~~~~~~~~~ not found in this scope
    }

    fn mutate(foo: &mut Field) {
        *foo = 1;
    }
    "#;
    check_errors(src);
}

#[test]
fn constrained_reference_to_unconstrained() {
    let src = r#"
    fn main(mut x: u32, y: pub u32) {
        let x_ref = &mut x;
        if x == 5  {
            // Safety: test context
            unsafe {
                mut_ref_input(x_ref, y);        
                              ^^^^^ Cannot pass a mutable reference from a constrained runtime to an unconstrained runtime
            }
        }

        assert(x == 10);
    }

    unconstrained fn mut_ref_input(x: &mut u32, y: u32) {
        *x = y;
    }
    "#;
    check_errors(src);
}

#[test]
fn immutable_references_with_ownership_feature_brillig() {
    let src = r#"
        unconstrained fn main() {
            let array = [1, 2, 3];
            borrow(&array);
        }

        fn borrow(_array: &[Field; 3]) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn immutable_references_with_ownership_feature() {
    let src = r#"
        fn main() {
            let array = [1, 2, 3];
            borrow(&array);
        }

        fn borrow(_array: &[Field; 3]) {}
     "#;
    assert_no_errors(src);
}

#[test]
fn calling_dereferenced_lambda_output_from_trait_impl() {
    let src = r#"
    trait Bar {
      fn bar(&mut self) -> &mut Self;
    }
    
    impl<T> Bar for T {
      fn bar(&mut self) -> &mut Self {
        self
      }
    }
    
    fn main() {
      let mut foo = |_x: ()| { true };
      let mut_ref_foo = &mut foo;
      assert((*mut_ref_foo.bar())(()))
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn calling_mutable_reference_to_lambda_output_from_trait_impl() {
    let src = r#"
    trait Bar {
      fn bar(&mut self) -> &mut Self;
    }
    
    impl<T> Bar for T {
      fn bar(&mut self) -> &mut Self {
        self
      }
    }
    
    fn main() {
      let mut foo = |_x: ()| { true };
      let mut_ref_foo = &mut foo;
      assert(mut_ref_foo.bar()(()))
             ^^^^^^^^^^^^^^^^^^^^^ Expected a function, but found a(n) &mut fn(()) -> bool
    }
    "#;
    check_errors(src);
}

#[test]
fn mutable_reference_behind_generics_returned_from_oracle() {
    let src = r#"
    unconstrained fn main() {
        let y = &mut 10;
        let add = |x: Field| { *y = *y + x; };
        let mul = |x: Field| { *y = *y * x; };

        let f = choose_func(add, mul);
                ^^^^^^^^^^^ Mutable reference `fn[(&mut Field,)](Field) -> ()` cannot be returned from an oracle function

        f(20);
    }

    #[oracle(choose_func)]
    unconstrained fn choose_func<Env>(
        f: fn[Env](Field) -> (),
        g: fn[Env](Field) -> (),
    ) -> fn[Env](Field) -> () {}
    "#;
    check_monomorphization_error(src);
}

#[test]
fn mutable_reference_behind_generics_returned_from_indirect_oracle() {
    let src = r#"
    unconstrained fn main() {
        foo::<&[(u8, u8); 3]>();
    }

    unconstrained fn foo<T>() {
        let f = get_array::<T>;
                ^^^^^^^^^ Mutable reference `[&[(u8, u8); 3]]` cannot be returned from an oracle function
        let _result = f();
    }

    #[oracle(get_array)]
    unconstrained fn get_array<T>() -> [T] {}
    "#;
    check_monomorphization_error(src);
}

#[test]
fn method_with_immutable_self_reference_does_not_require_mutable_variable() {
    let src = r#"
    struct S { inner: Field }

    impl S {
        fn ping(self: &S) -> Field {
            self.inner
        }
    }

    fn main() {
        let s = S { inner: 1 };
        assert(s.ping() == 1);
    }
    "#;
    assert_no_errors_using_features(src, &[]);
}

#[test]
fn disallows_mutating_non_mutable_ref_member_access() {
    let src = r#"
    fn main() {
        let s = (0,);
        let ps = &s;
        ps.0 = 1;
        ^^ `ps` is a `&` reference, so it cannot be written to
    }
    "#;
    check_errors_using_features(src, &[]);
}

#[test]
fn disallows_mutating_non_mutable_ref_array_index() {
    let src = r#"
    fn main() {
        let s = [0];
        let ps = &s;
        ps[0] = 1;
        ^^ `ps` is a `&` reference, so it cannot be written to
    }
    "#;
    check_errors_using_features(src, &[]);
}

#[test]
fn disallows_mutating_non_mutable_nested_reference_in_tuple_1() {
    let src = r#"
    fn main() {
        let x = (&(0,),);
        x.0.0 = 1;
        ^^^^^ Cannot assign to `x.0.0`, which is behind a `&` reference
    }
    "#;
    check_errors_using_features(src, &[]);
}

#[test]
fn allows_mutating_mutable_reference_inside_non_mutable_reference() {
    let src = r#"
    fn main() {
        let x = &(&mut (0,),);
        x.0.0 = 1;
    }
    "#;
    check_errors_using_features(src, &[]);
}

#[test]
fn disallows_mutating_non_mutable_reference_inside_mutable_reference() {
    let src = r#"
    fn main() {
        let x = &mut (&(0,),);
        x.0.0 = 1;
        ^^^^^ Cannot assign to `x.0.0`, which is behind a `&` reference
    }
    "#;
    check_errors_using_features(src, &[]);
}

#[test]
fn cannot_take_mut_ref_of_immutable_variable_in_deref() {
    let src = r#"
    fn main() {
        let x: Field = 5;
        let _y = *&mut x;
                       ^ Cannot mutate immutable variable `x`
    }
    "#;
    check_errors(src);
}

#[test]
fn generic_inference_through_mutable_reference() {
    let src = r#"
    struct Foo<let N: u32> {
        data: [Field; N],
    }

    fn by_mut_ref<let N: u32>(foo: &mut Foo<N>) -> Field {
        foo.data[0]
    }

    fn main() {
        let mut foo = Foo { data: [1, 2, 3] };
        assert(by_mut_ref(&mut foo) == 1);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn generic_inference_through_mutable_reference_auto_borrow_rejected() {
    // Auto-borrow is not supported: passing a value where &mut Foo<N> is expected
    // should be rejected by the compiler.
    let src = r#"
    struct Foo<let N: u32> {
        data: [Field; N],
    }

    fn by_mut_ref<let N: u32>(foo: &mut Foo<N>) -> Field {
        foo.data[0]
    }

    fn main() {
        let foo = Foo { data: [1, 2, 3] };
        assert(by_mut_ref(foo) == 1);
               ^^^^^^^^^^ Type annotation needed
               ~~~~~~~~~~ Could not determine the value of the generic argument `N` declared on the function `by_mut_ref`
                          ^^^ Expected type &mut Foo<_>, found type Foo<3>
    }
    "#;
    check_errors(src);
}

#[test]
fn mutable_reference_auto_borrow_rejected() {
    // Auto-borrow is not supported: passing a value where &mut Foo is expected
    // should be rejected by the compiler.
    let src = r#"
    struct Foo {
        data: Field,
    }

    fn by_mut_ref(foo: &mut Foo) -> Field {
        foo.data
    }

    fn main() {
        let foo = Foo { data: 1 };
        assert(by_mut_ref(foo) == 1);
                          ^^^ Expected type &mut Foo, found type Foo
    }
    "#;
    check_errors(src);
}

#[test]
fn generic_inference_through_immutable_reference() {
    let src = r#"
    struct Foo<let N: u32> {
        data: [Field; N],
    }

    fn by_ref<let N: u32>(foo: &Foo<N>) -> Field {
        foo.data[0]
    }

    unconstrained fn main() {
        let foo = Foo { data: [1, 2, 3] };
        assert(by_ref(&foo) == 1);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn generic_inference_through_immutable_reference_multiple_numeric_generics() {
    let src = r#"
    struct PublicCall<let M: u32, let N: u32, T> {
        name: str<M>,
        args: [Field; N],
        _phantom: T,
    }

    struct Caller {}

    impl Caller {
        unconstrained fn call<let M: u32, let N: u32, T>(_self: Caller, _call: &PublicCall<M, N, T>) {}
    }

    unconstrained fn main() {
        let caller = Caller {};
        let pc = PublicCall { name: "hello", args: [1, 2, 3], _phantom: 0 as Field };
        caller.call(&pc);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn generic_inference_through_immutable_reference_auto_borrow_rejected() {
    // Auto-borrow is not supported: passing a value where &Foo<N> is expected
    // should be rejected by the compiler.
    let src = r#"
    struct Foo<let N: u32> {
        data: [Field; N],
    }

    fn by_ref<let N: u32>(foo: &Foo<N>) -> Field {
        foo.data[0]
    }

    unconstrained fn main() {
        let foo = Foo { data: [1, 2, 3] };
        assert(by_ref(foo) == 1);
               ^^^^^^ Type annotation needed
               ~~~~~~ Could not determine the value of the generic argument `N` declared on the function `by_ref`
                      ^^^ Expected type &Foo<_>, found type Foo<3>
    }
    "#;
    check_errors(src);
}

#[test]
fn generic_inference_through_mutable_reference_method_auto_ref() {
    let src = r#"
    struct Caller {}

    struct PublicCall<let M: u32, let N: u32> {
        name: str<M>,
        args: [Field; N],
    }

    impl Caller {
        fn call<let M: u32, let N: u32>(_self: Caller, _call: &mut PublicCall<M, N>) {}
    }

    fn main() {
        let caller = Caller {};
        let mut pc = PublicCall { name: "hello", args: [1, 2, 3] };
        caller.call(&mut pc);
    }
    "#;
    assert_no_errors(src);
}
