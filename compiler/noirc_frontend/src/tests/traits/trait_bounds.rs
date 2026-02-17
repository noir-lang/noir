//! Tests for trait bound checking and where clause validation.
//! Validates that trait bounds are satisfied and constraints on associated types are correctly checked.

use crate::tests::{UnstableFeature, assert_no_errors, check_errors, get_program_using_features};

#[test]
fn trait_impl_for_a_type_that_implements_another_trait() {
    let src = r#"
    trait One {
        fn one(self) -> i32;
    }

    impl One for i32 {
        fn one(self) -> i32 {
            self
        }
    }

    trait Two {
        fn two(self) -> i32;
    }

    impl<T> Two for T where T: One {
        fn two(self) -> i32 {
            self.one() + 1
        }
    }

    pub fn use_it<T>(t: T) -> i32 where T: Two {
        Two::two(t)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_impl_for_a_type_that_implements_another_trait_with_another_impl_used() {
    let src = r#"
    trait One {
        fn one(self) -> i32;
    }

    impl One for i32 {
        fn one(self) -> i32 {
            let _ = self;
            1
        }
    }

    trait Two {
        fn two(self) -> i32;
    }

    impl<T> Two for T where T: One {
        fn two(self) -> i32 {
            self.one() + 1
        }
    }

    impl Two for u32 {
        fn two(self) -> i32 {
            let _ = self;
            0
        }
    }

    pub fn use_it(t: u32) -> i32 {
        Two::two(t)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn check_trait_implemented_for_all_t() {
    let src = "
    trait Default2 {
        fn default2() -> Self;
    }

    trait Eq2 {
        fn eq2(self, other: Self) -> bool;
    }

    trait IsDefault {
        fn is_default(self) -> bool;
    }

    impl<T> IsDefault for T where T: Default2 + Eq2 {
        fn is_default(self) -> bool {
            self.eq2(T::default2())
        }
    }

    struct Foo {
        a: u64,
    }

    impl Eq2 for Foo {
        fn eq2(self, other: Foo) -> bool { self.a == other.a }
    }

    impl Default2 for u64 {
        fn default2() -> Self {
            0
        }
    }

    impl Default2 for Foo {
        fn default2() -> Self {
            Foo { a: Default2::default2() }
        }
    }

    fn main(a: Foo) -> pub bool {
        a.is_default()
    }";
    assert_no_errors(src);
}

#[test]
fn check_trait_as_type_as_fn_parameter() {
    let src = "
    trait Eq2 {
        fn eq2(self, other: Self) -> bool;
    }

    struct Foo {
        a: u64,
    }

    impl Eq2 for Foo {
        fn eq2(self, other: Foo) -> bool { self.a == other.a }
    }

    // `impl T` syntax is expected to be desugared to a `where` clause
    fn test_eq(x: impl Eq2) -> bool {
                       ^^^ `impl Trait` as a type is experimental
                       ~~~ Pass -Ztrait_as_type to nargo to enable this feature at your own risk.
        x.eq2(x)
    }

    fn main(a: Foo) -> pub bool {
        test_eq(a)
    }";
    check_errors(src);

    let src_without_errors: Vec<_> =
        src.lines().filter(|line| !line.contains(['^', '~'])).collect();
    let src_without_errors = src_without_errors.join("\n");
    let features = vec![UnstableFeature::TraitAsType];
    let (_, _, errors) = get_program_using_features(&src_without_errors, &features);
    assert_eq!(errors, vec![]);
}

#[test]
fn check_trait_as_type_as_two_fn_parameters() {
    let src = "
    trait Eq2 {
        fn eq2(self, other: Self) -> bool;
    }

    trait Test {
        fn test(self) -> bool;
    }

    struct Foo {
        a: u64,
    }

    impl Eq2 for Foo {
        fn eq2(self, other: Foo) -> bool { self.a == other.a }
    }

    impl Test for u64 {
        fn test(self) -> bool { self == self }
    }

    // `impl T` syntax is expected to be desugared to a `where` clause
    fn test_eq(x: impl Eq2, y: impl Test) -> bool {
                       ^^^ `impl Trait` as a type is experimental
                       ~~~ Pass -Ztrait_as_type to nargo to enable this feature at your own risk.
                                    ^^^^ `impl Trait` as a type is experimental
                                    ~~~~ Pass -Ztrait_as_type to nargo to enable this feature at your own risk.
        x.eq2(x) == y.test()
    }

    fn main(a: Foo, b: u64) -> pub bool {
        test_eq(a, b)
    }";
    check_errors(src);

    let src_without_errors: Vec<_> =
        src.lines().filter(|line| !line.contains(['^', '~'])).collect();
    let src_without_errors = src_without_errors.join("\n");
    let features = vec![UnstableFeature::TraitAsType];
    let (_, _, errors) = get_program_using_features(&src_without_errors, &features);
    assert_eq!(errors, vec![]);
}

#[test]
fn does_not_error_if_impl_trait_constraint_is_satisfied_for_concrete_type() {
    let src = r#"
        pub trait Greeter {
            fn greet(self);
        }

        pub trait Foo<T>
        where
            T: Greeter,
        {
            fn greet<U>(object: U)
            where
                U: Greeter,
            {
                object.greet();
            }
        }

        pub struct SomeGreeter;
        impl Greeter for SomeGreeter {
            fn greet(self) {}
        }

        pub struct Bar;

        impl Foo<SomeGreeter> for Bar {}
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_error_if_impl_trait_constraint_is_satisfied_for_type_variable() {
    let src = r#"
        pub trait Greeter {
            fn greet(self);
        }

        pub trait Foo<T> where T: Greeter {
            fn greet(object: T) {
                object.greet();
            }
        }

        pub struct Bar;

        impl<T> Foo<T> for Bar where T: Greeter {
        }

        fn main() {
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_if_impl_trait_constraint_is_not_satisfied() {
    let src = r#"
        pub trait Greeter {
            fn greet(self);
        }

        pub trait Foo<T>
        where
            T: Greeter,
               ~~~~~~~ required by this bound in `Foo`
        {
            fn greet<U>(object: U)
            where
                U: Greeter,
            {
                object.greet();
            }
        }

        pub struct SomeGreeter;

        pub struct Bar;

        impl Foo<SomeGreeter> for Bar {}
                                  ^^^ The trait bound `SomeGreeter: Greeter` is not satisfied
                                  ~~~ The trait `Greeter` is not implemented for `SomeGreeter`
    "#;
    check_errors(src);
}

#[test]
fn errors_on_unknown_type_in_trait_where_clause() {
    let src = r#"
        pub trait Foo<T> where T: Unknown {}
                                  ^^^^^^^ Could not resolve 'Unknown' in path

        fn main() {
        }
    "#;
    check_errors(src);
}

#[test]
fn trait_bounds_which_are_dependent_on_generic_types_are_resolved_correctly() {
    // Regression test for https://github.com/noir-lang/noir/issues/6420
    let src = r#"
        trait Foo {
            fn foo(self) -> Field;
        }

        trait Bar<T>: Foo {
            fn bar(self) -> Field {
                self.foo()
            }
        }

        struct MyStruct<T> {
            inner: Field,
        }

        trait MarkerTrait {}
        impl MarkerTrait for Field {}

        // `MyStruct<T>` implements `Foo` only when its generic type `T` implements `MarkerTrait`.
        impl<T> Foo for MyStruct<T>
        where
            T: MarkerTrait,
        {
            fn foo(self) -> Field {
                let _ = self;
                42
            }
        }

        // We expect this to succeed as `MyStruct<T>` satisfies `Bar`'s trait bounds
        // of implementing `Foo` when `T` implements `MarkerTrait`.
        impl<T> Bar<T> for MyStruct<T>
        where
            T: MarkerTrait,
        {
            fn bar(self) -> Field {
                31415
            }
        }

        fn main() {
            let foo: MyStruct<Field> = MyStruct { inner: 42 };
            let _ = foo.bar();
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_bound_with_associated_constant() {
    let src = r#"
    pub trait Other {
        let N: u32;
    }

    pub trait Trait<T>
    where
        T: Other,
    {}

    impl Other for Field {
        let N: u32 = 1;
    }

    impl Trait<Field> for i32 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_method_call_when_it_has_bounds_on_generic() {
    let src = r#"
    trait BigNum {}

    trait BigCurve<B>
    where
        B: BigNum,
    {
        fn new() -> Self;
    }

    pub fn foo<B: BigNum, Curve: BigCurve<B>>() {
        let _: Curve = BigCurve::new();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_bound_constraining_two_generics() {
    let src = r#"
    pub trait Foo<U> {}

    pub trait Baz<T, U>
    where
        T: Foo<U>,
    {}

    pub struct HasFoo1 {}
    impl Foo<()> for HasFoo1 {}

    pub struct HasBaz1 {}
    impl Baz<HasFoo1, ()> for HasBaz1 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_where_clause_associated_type_constraint_expected_order() {
    let src = r#"
    pub trait BarTrait {}

    pub trait Foo {
        type Bar;
    }

    pub trait Baz<T>
    where
        T: Foo,
        <T as Foo>::Bar: BarTrait,
    {}

    pub struct HasBarTrait1 {}
    impl BarTrait for HasBarTrait1 {}

    pub struct HasFoo1 {}
    impl Foo for HasFoo1 {
        type Bar = HasBarTrait1;
    }

    pub struct HasBaz1 {}
    impl Baz<HasFoo1> for HasBaz1 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_where_clause_associated_type_constraint_unexpected_order() {
    let src = r#"
    pub trait BarTrait {}

    pub trait Foo {
        type Bar;
    }

    pub trait Baz<T>
    where
        <T as Foo>::Bar: BarTrait,
        T: Foo,
    {}

    pub struct HasBarTrait1 {}
    impl BarTrait for HasBarTrait1 {}

    pub struct HasFoo1 {}
    impl Foo for HasFoo1 {
        type Bar = HasBarTrait1;
    }

    pub struct HasBaz1 {}
    impl Baz<HasFoo1> for HasBaz1 {}
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_bound_on_implementing_type() {
    let src = r#"
    struct GenericStruct<T> {
        inner: T,
    }

    trait Foo {
        fn foo() {}
    }

    impl Foo for Field {}

    impl<T: Foo> Foo for GenericStruct<T> {}

    trait Bar {
        fn bar();
    }

    impl<T> Bar for GenericStruct<T>
    where
        GenericStruct<T>: Foo,
    {
        fn bar() {
            <Self as Foo>::foo()
        }
    }

    fn main() {
        GenericStruct::<Field>::bar();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_constraint_on_tuple_type() {
    let src = r#"
    trait Foo<A> {
        fn foo(self, x: A) -> bool;
    }

    pub fn bar<T, U, V>(x: (T, U), y: V) -> bool where (T, U): Foo<V> {
        x.foo(y)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn trait_constraint_on_tuple_type_pub_crate() {
    let src = r#"
    pub(crate) trait Foo<A> {
        fn foo(self, x: A) -> bool;
    }

    pub fn bar<T, U, V>(x: (T, U), y: V) -> bool where (T, U): Foo<V> {
        x.foo(y)
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn short_syntax_for_trait_constraint_on_trait_generic() {
    let src = r#"
    pub trait Other {
        fn other(self) {
            let _ = self;
        }
    }

    pub trait Trait<T: Other> {
        fn foo(x: T) {
            x.other();
        }
    }

    fn main() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_error_if_type_parameter_is_used_in_trait_bound_named_generic() {
    let src = r#"
    pub trait SomeTrait {}
    pub trait AnotherTrait {
        type AssocType;
    }

    impl<T, U> SomeTrait for T where T: AnotherTrait<AssocType=U> {}
    "#;
    assert_no_errors(src);
}

// TODO(https://github.com/noir-lang/noir/issues/11499): Fail with an error that mentions some type of "overflow" error
#[test]
fn errors_on_mutually_recursive_impls() {
    let src = r#"
    trait Foo {
        fn foo(self) {
            let _ = self;
        }
    }

    pub struct Bar {}
    pub struct Baz {}

    impl Foo for Bar where Baz: Foo {
                                ^^^ Constraint for `Baz: Foo` is not needed, another matching impl is already in scope
                                ~~~ Unnecessary trait constraint in where clause
        fn foo(self) {
            (Baz {}).foo()
        }
    }

    impl Foo for Baz where Bar: Foo {
                                ^^^ Constraint for `Bar: Foo` is not needed, another matching impl is already in scope
                                ~~~ Unnecessary trait constraint in where clause
        fn foo(self) {
            (Bar {}).foo()
        }
    }

    fn main() {
        (Bar {}).foo();
        ^^^^^^^^^^^^ No matching impl found for `Bar: Foo`
        ~~~~~~~~~~~~ No impl for `Bar: Foo`

        (Baz {}).foo();
        ^^^^^^^^^^^^ No matching impl found for `Baz: Foo`
        ~~~~~~~~~~~~ No impl for `Baz: Foo`
    }
    "#;
    check_errors(src);
}

// Regression test for https://github.com/noir-lang/noir/issues/11514
#[test]
fn where_clause_on_generic_struct_parameter() {
    let src = r#"
    pub trait E {
        fn e(self);
    }

    pub struct A<F> {
        f: F,
    }

    pub struct F<G> {
        g: G,
    }

    pub fn f<X>(w: A<F<X>>)
    where
        F<X>: E,
    {
        w.f.e();
    }
    "#;
    assert_no_errors(src);
}

// Regression test for https://github.com/noir-lang/noir/issues/11514 (simplified)
#[test]
fn where_clause_on_self_type_with_generic() {
    let src = r#"
    pub trait E {
        fn e(self);
    }

    pub struct A<F> {
        f: F,
    }

    pub fn f<X>(a: A<X>)
    where
        A<X>: E,
    {
        a.e();
    }
    "#;
    assert_no_errors(src);
}

// Regression test for https://github.com/noir-lang/noir/issues/11553
#[test]
fn nested_angle_brackets_in_type_position() {
    let src = r#"
    pub trait HasKey {
        type Key;
    }

    pub struct Store<K> {
        key: K,
    }

    pub fn make_store<T>(key: <T as HasKey>::Key) -> Store<<T as HasKey>::Key>
    where
        T: HasKey,
    {
        Store { key }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn where_clause_on_constructed_generic_type() {
    let src = r#"
    trait Serialize {
        fn serialize(self) -> [Field; 1];
    }

    struct Envelope<T> {
        payload: T,
    }

    impl Serialize for Envelope<Field> {
        fn serialize(self) -> [Field; 1] {
            [self.payload]
        }
    }

    fn process<X>(e: Envelope<X>) -> [Field; 1] where Envelope<X>: Serialize {
        e.serialize()
    }

    fn main() {
        let e = Envelope { payload: 42 as Field };
        let _ = process(e);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn where_clause_nested_constructed_type() {
    let src = r#"
    trait Encode {
        fn encode(self) -> Field;
    }

    struct Inner<T> {
        val: T,
    }

    struct Outer<T> {
        inner: Inner<T>,
    }

    impl Encode for Inner<Field> {
        fn encode(self) -> Field {
            self.val
        }
    }

    fn extract<X>(o: Outer<X>) -> Field where Inner<X>: Encode {
        o.inner.encode()
    }

    fn main() {
        let o = Outer { inner: Inner { val: 99 as Field } };
        let _ = extract(o);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn where_clause_on_array_of_generic() {
    let src = r#"
    trait Summable {
        fn to_field(self) -> Field;
    }

    impl Summable for Field {
        fn to_field(self) -> Field { self }
    }

    fn sum_array<T, let N: u32>(arr: [T; N]) -> Field where T: Summable {
        let mut s: Field = 0;
        for i in 0..N {
            s += arr[i].to_field();
        }
        s
    }

    fn main() {
        let arr = [1, 2, 3, 4, 5];
        assert(sum_array(arr) == 15);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn multiple_where_clauses_mixing_generic_and_numeric() {
    let src = r#"
    trait Hashable {
        fn hash(self) -> Field;
    }

    trait HasLen {
        let LEN: u32;
    }

    impl Hashable for Field {
        fn hash(self) -> Field { self }
    }

    struct Container<T, let N: u32> {
        data: [T; N],
    }

    impl<T, let N: u32> HasLen for Container<T, N> {
        let LEN: u32 = N;
    }

    fn hash_first<T, let N: u32>(c: Container<T, N>) -> Field
    where
        T: Hashable,
        Container<T, N>: HasLen,
    {
        c.data[0].hash()
    }

    fn main() {
        let c = Container { data: [42 as Field, 0] };
        assert(hash_first(c) == 42);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn generic_function_with_associated_type_equality_constraint() {
    let src = r#"
    trait Producer {
        type Output;
        fn produce(self) -> Self::Output;
    }

    impl Producer for u32 {
        type Output = Field;
        fn produce(self) -> Self::Output {
            self as Field
        }
    }

    fn produce_field<T>(t: T) -> Field
    where
        T: Producer<Output = Field>,
    {
        t.produce()
    }

    fn main() {
        assert(produce_field(42 as u32) == 42);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn where_clause_on_associated_type_fully_qualified() {
    // Fully-qualified syntax <I as Iter>::Item: Display works
    let src = r#"
    trait Iter {
        type Item;
        fn next(self) -> Self::Item;
    }

    trait Display {
        fn show(self) -> Field;
    }

    struct Range {
        val: u32,
    }

    impl Iter for Range {
        type Item = u32;
        fn next(self) -> Self::Item {
            self.val
        }
    }

    impl Display for u32 {
        fn show(self) -> Field {
            self as Field
        }
    }

    fn show_next<I>(iter: I) -> Field
    where
        I: Iter,
        <I as Iter>::Item: Display,
    {
        iter.next().show()
    }

    fn main() {
        let r = Range { val: 5 };
        assert(show_next(r) == 5);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn where_clause_on_struct_with_numeric_generic_field() {
    let src = r#"
    trait Process {
        fn process(self) -> Field;
    }

    struct Arr<let N: u32> {
        data: [Field; N],
    }

    impl<let N: u32> Process for Arr<N> {
        fn process(self) -> Field {
            self.data[0]
        }
    }

    struct Wrapper<T> {
        inner: T,
    }

    fn process_wrapper<T>(w: Wrapper<T>) -> Field where T: Process {
        w.inner.process()
    }

    fn main() {
        let w = Wrapper { inner: Arr { data: [42, 0, 0] } };
        assert(process_wrapper(w) == 42);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn recursive_trait_bound_on_generic() {
    let src = r#"
    trait Eq {
        fn eq(self, other: Self) -> bool;
    }

    impl Eq for Field {
        fn eq(self, other: Self) -> bool {
            self == other
        }
    }

    struct Pair<T> {
        fst: T,
        snd: T,
    }

    impl<T> Eq for Pair<T> where T: Eq {
        fn eq(self, other: Self) -> bool {
            self.fst.eq(other.fst) & self.snd.eq(other.snd)
        }
    }

    fn are_equal<T>(a: T, b: T) -> bool where T: Eq {
        a.eq(b)
    }

    fn main() {
        let p1 = Pair { fst: 1 as Field, snd: 2 as Field };
        let p2 = Pair { fst: 1 as Field, snd: 2 as Field };
        assert(are_equal(p1, p2));
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn deeply_nested_generic_where_clause() {
    let src = r#"
    trait Hash {
        fn hash(self) -> Field;
    }

    impl Hash for Field {
        fn hash(self) -> Field {
            self
        }
    }

    struct Box<T> {
        val: T,
    }

    impl<T> Hash for Box<T> where T: Hash {
        fn hash(self) -> Field {
            self.val.hash()
        }
    }

    fn hash_nested<T>(b: Box<Box<T>>) -> Field where T: Hash {
        b.hash()
    }

    fn main() {
        let nested = Box { val: Box { val: 42 as Field } };
        assert(hash_nested(nested) == 42);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn generic_impl_with_multiple_trait_bounds() {
    let src = r#"
    trait Hashable {
        fn hash(self) -> Field;
    }

    trait Comparable {
        fn compare(self, other: Self) -> bool;
    }

    impl Hashable for Field {
        fn hash(self) -> Field { self }
    }

    impl Comparable for Field {
        fn compare(self, other: Self) -> bool { self == other }
    }

    struct Set<T> {
        element: T,
    }

    impl<T> Set<T> where T: Hashable + Comparable {
        fn contains(self, item: T) -> bool {
            self.element.compare(item)
        }

        fn hash_element(self) -> Field {
            self.element.hash()
        }
    }

    fn main() {
        let s = Set { element: 42 as Field };
        assert(s.contains(42));
        assert(s.hash_element() == 42);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn triple_nested_generic_trait_bound() {
    let src = r#"
    trait Mappable {
        fn map_field(self) -> Field;
    }

    impl Mappable for Field {
        fn map_field(self) -> Field { self }
    }

    struct Layer1<T> { val: T }
    struct Layer2<T> { val: T }
    struct Layer3<T> { val: T }

    impl<T> Mappable for Layer1<T> where T: Mappable {
        fn map_field(self) -> Field { self.val.map_field() }
    }

    impl<T> Mappable for Layer2<T> where T: Mappable {
        fn map_field(self) -> Field { self.val.map_field() }
    }

    impl<T> Mappable for Layer3<T> where T: Mappable {
        fn map_field(self) -> Field { self.val.map_field() }
    }

    fn deep_map<T>(t: T) -> Field where T: Mappable {
        t.map_field()
    }

    fn main() {
        let nested = Layer1 { val: Layer2 { val: Layer3 { val: 42 as Field } } };
        assert(deep_map(nested) == 42);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn comptime_fn_with_generic_trait_constraint() {
    let src = r#"
    trait Named {
        comptime fn type_name() -> str<5>;
    }

    struct Foo {}
    struct Bar {}

    impl Named for Foo {
        comptime fn type_name() -> str<5> {
            "Foo__"
        }
    }

    impl Named for Bar {
        comptime fn type_name() -> str<5> {
            "Bar__"
        }
    }

    comptime fn get_name<T>() -> str<5> where T: Named {
        T::type_name()
    }

    fn main() {
        comptime {
            let _ = get_name::<Foo>();
            let _ = get_name::<Bar>();
        }
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn associated_type_as_generic_trait_param_spaced() {
    // Associated type used as parameter of a generic trait (space avoids << parse issue)
    let src = r#"
    trait HasKey {
        type Key;
    }

    trait Store<K> {
        fn get(self, key: K) -> Field;
    }

    struct Map {
        val: Field,
    }

    impl HasKey for Map {
        type Key = u32;
    }

    impl Store<u32> for Map {
        fn get(self, _key: u32) -> Field {
            self.val
        }
    }

    fn fetch<T>(store: T, key: <T as HasKey>::Key) -> Field
    where
        T: HasKey + Store< <T as HasKey>::Key >,
    {
        store.get(key)
    }

    fn main() {
        let m = Map { val: 42 };
        assert(fetch(m, 0 as u32) == 42);
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11546
#[test]
fn where_clause_on_associated_type_shorthand_ignored() {
    let src = r#"
    trait Collection {
        type Elem;
        fn first(self) -> Self::Elem;
    }

    trait Printable {
        fn to_field(self) -> Field;
    }

    struct Bag {
        item: Field,
    }

    impl Collection for Bag {
        type Elem = Field;
        fn first(self) -> Self::Elem {
            self.item
        }
    }

    impl Printable for Field {
        fn to_field(self) -> Field {
            self
        }
    }

    fn first_as_field<C>(c: C) -> Field
    where
        C: Collection,
        C::Elem: Printable,
    {
        c.first().to_field()
    }

    fn main() {
        let b = Bag { item: 42 };
        assert(first_as_field(b) == 42);
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11546
#[test]
fn where_clause_on_associated_type_shorthand_in_function() {
    let src = r#"
    trait Transform {
        type Output;
        fn transform(self) -> Self::Output;
    }

    trait Validate {
        fn validate(self) -> bool;
    }

    impl Transform for Field {
        type Output = bool;
        fn transform(self) -> Self::Output {
            self != 0
        }
    }

    impl Validate for bool {
        fn validate(self) -> bool {
            self
        }
    }

    fn transform_and_validate<T>(t: T) -> bool
    where
        T: Transform,
        T::Output: Validate,
    {
        t.transform().validate()
    }

    fn main() {
        assert(transform_and_validate(1 as Field));
    }
    "#;
    assert_no_errors(src);
}

/// Regression for https://github.com/noir-lang/noir/issues/11546
#[test]
fn multiple_associated_types_in_where_clause() {
    let src = r#"
    trait Pair {
        type First;
        type Second;
        fn first(self) -> Self::First;
        fn second(self) -> Self::Second;
    }

    trait ToField {
        fn to_field(self) -> Field;
    }

    impl ToField for Field {
        fn to_field(self) -> Field { self }
    }

    impl ToField for bool {
        fn to_field(self) -> Field { if self { 1 } else { 0 } }
    }

    struct TwoFields {
        a: Field,
        b: bool,
    }

    impl Pair for TwoFields {
        type First = Field;
        type Second = bool;
        fn first(self) -> Self::First { self.a }
        fn second(self) -> Self::Second { self.b }
    }

    fn sum_pair<P>(p: P) -> Field
    where
        P: Pair,
        P::First: ToField,
        P::Second: ToField,
    {
        p.first().to_field() + p.second().to_field()
    }

    fn main() {
        let t = TwoFields { a: 10, b: true };
        assert(sum_pair(t) == 11);
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11546
#[test]
fn where_clause_on_associated_type_of_generic_in_trait_impl() {
    let src = r#"
    trait HasOutput {
        type Output;
        fn output(self) -> Self::Output;
    }

    trait Render {
        fn render(self) -> Field;
    }

    impl Render for Field {
        fn render(self) -> Field { self }
    }

    trait Renderer {
        fn render_output(self) -> Field;
    }

    struct Widget<T> {
        source: T,
    }

    impl HasOutput for u32 {
        type Output = Field;
        fn output(self) -> Self::Output {
            self as Field
        }
    }

    impl<T> Renderer for Widget<T>
    where
        T: HasOutput,
        T::Output: Render,
    {
        fn render_output(self) -> Field {
            self.source.output().render()
        }
    }

    fn main() {
        let w = Widget { source: 42 as u32 };
        assert(w.render_output() == 42);
    }
    "#;
    assert_no_errors(src);
}

/// Regression test for https://github.com/noir-lang/noir/issues/11553
#[test]
fn associated_type_as_generic_trait_param_with_nested_angle_brackets() {
    // Bug: Parser fails on << in type position: Store<<T as HasKey>::Key>
    let src = r#"
    trait HasKey {
        type Key;
    }

    trait Store<K> {
        fn get(self, key: K) -> Field;
    }

    struct Map {
        val: Field,
    }

    impl HasKey for Map {
        type Key = u32;
    }

    impl Store<u32> for Map {
        fn get(self, _key: u32) -> Field {
            self.val
        }
    }

    fn fetch<T>(store: T, key: <T as HasKey>::Key) -> Field
    where
        T: HasKey,
        T: Store<<T as HasKey>::Key>,
    {
        store.get(key)
    }

    fn main() {
        let m = Map { val: 42 };
        assert(fetch(m, 0 as u32) == 42);
    }
    "#;
    assert_no_errors(src);
}
