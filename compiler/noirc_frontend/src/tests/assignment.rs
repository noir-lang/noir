//! Tests for assignment statements, focusing on:
//! 1. Side-effect ordering in complex lvalues
//! 2. Nested pattern matching, including nested tuples and dereferences
//! 3. Miscellaneous error cases such as comptime assignment and string indexing

use crate::tests::{assert_no_errors, assert_no_errors_and_to_string, check_errors};

// LValue side-effect ordering

#[test]
fn mutate_in_lvalue_block_expr() {
    let src = r#"
    fn main() {
        mutate_in_lvalue();
        comptime { mutate_in_lvalue() };
    }
    
    fn mutate_in_lvalue() {
        let mut a = ([1, 2], 3);
        a.0[{
        a = ([4, 5], 6);
        1
        }] = 7;

        assert_eq(a.0[0], 4);
        assert_eq(a.0[1], 7);
        assert_eq(a.1, 6);
    }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    // We want to make sure that the nested mutation of `a` occurs in the expected order.
    // The `a` mutation inside of the lvalue index should occur first.
    insta::assert_snapshot!(expanded, @r"
    fn main() {
        mutate_in_lvalue();
        ()
    }
    
    fn mutate_in_lvalue() {
        let mut a: ([Field; 2], Field) = ([1_Field, 2_Field], 3_Field);
        {
            let i_3: u32 = {
                a = ([4_Field, 5_Field], 6_Field);
                1_u32
            };
            a.0[i_3] = 7_Field;
        };
        assert(a.0[0_u32] == 4_Field);
        assert(a.0[1_u32] == 7_Field);
        assert(a.1 == 6_Field);
    }
    ");
}

#[test]
fn multiple_block_expressions_in_lvalue() {
    let src = r#"
        fn main() {
            let mut arr = [[1, 2], [3, 4]];

            arr[{
                arr = [[5, 6], [7, 8]];
                0
            }][{
                arr[0][0] = 9;
                1
            }] = 10;

            // First block sets arr to [[5, 6], [7, 8]]
            // Second block sets arr[0][0] to 9, so arr is [[9, 6], [7, 8]]
            // Final assignment sets arr[0][1] to 10, so arr is [[9, 10], [7, 8]]
            assert(arr[0][0] == 9);
            assert(arr[0][1] == 10);
            assert(arr[1][0] == 7);
            assert(arr[1][1] == 8);
        }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    // Verify that the block expressions are hoisted in left-to-right order
    insta::assert_snapshot!(expanded, @r"
    fn main() {
        let mut arr: [[Field; 2]; 2] = [[1_Field, 2_Field], [3_Field, 4_Field]];
        {
            let i_2: u32 = {
                arr = [[5_Field, 6_Field], [7_Field, 8_Field]];
                0_u32
            };
            let i_3: u32 = {
                arr[0_u32][0_u32] = 9_Field;
                1_u32
            };
            arr[i_2][i_3] = 10_Field;
        };
        assert(arr[0_u32][0_u32] == 9_Field);
        assert(arr[0_u32][1_u32] == 10_Field);
        assert(arr[1_u32][0_u32] == 7_Field);
        assert(arr[1_u32][1_u32] == 8_Field);
    }
    ");
}

#[test]
fn deeply_nested_block_expressions_in_lvalue() {
    // All blocks must mutate the same location to verify execution order
    let src = r#"
        fn main() {
            let mut arr = [[1, 2], [3, 4]];

            arr[{
                let x = {
                    arr = [[10, 20], [30, 40]];
                    0
                };
                arr = [[100, 200], [300, 400]];
                x
            }][{
                arr = [[1000, 2000], [3000, 4000]];
                1
            }] = 5000;

            // Innermost block: arr = [[10, 20], [30, 40]]
            // Outer part of first block: arr = [[100, 200], [300, 400]]
            // Second block: arr = [[1000, 2000], [3000, 4000]]
            // Final assignment: arr[0][1] = 5000, so arr = [[1000, 5000], [3000, 4000]]
            assert(arr[0][0] == 1000);
            assert(arr[0][1] == 5000);
            assert(arr[1][0] == 3000);
            assert(arr[1][1] == 4000);
        }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    // Verify that nested blocks are properly desugared and maintain evaluation order
    insta::assert_snapshot!(expanded, @r"
    fn main() {
        let mut arr: [[Field; 2]; 2] = [[1_Field, 2_Field], [3_Field, 4_Field]];
        {
            let i_3: u32 = {
                let x: u32 = {
                    arr = [[10_Field, 20_Field], [30_Field, 40_Field]];
                    0_u32
                };
                arr = [[100_Field, 200_Field], [300_Field, 400_Field]];
                x
            };
            let i_4: u32 = {
                arr = [[1000_Field, 2000_Field], [3000_Field, 4000_Field]];
                1_u32
            };
            arr[i_3][i_4] = 5000_Field;
        };
        assert(arr[0_u32][0_u32] == 1000_Field);
        assert(arr[0_u32][1_u32] == 5000_Field);
        assert(arr[1_u32][0_u32] == 3000_Field);
        assert(arr[1_u32][1_u32] == 4000_Field);
    }
    ");
}

#[test]
fn nested_array_index_side_effect_ordering() {
    // Tests ordering with three levels of nesting
    let src = r#"
        fn inc(c: &mut Field) -> u32 {
            let old = *c;
            *c = *c + 1;
            old as u32
        }

        fn main() {
            let mut counter = 0;
            let mut arr = [[[0; 2]; 2]; 2];

            arr[inc(&mut counter)][inc(&mut counter)][inc(&mut counter)] = 42;

            // All three indices evaluated left-to-right
            assert(counter == 3);
            assert(arr[0][1][2] == 42);
        }
    "#;
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    fn inc(c: &mut Field) -> u32 {
        let old: Field = *c;
        *c = *c + 1_Field;
        old as u32
    }
    
    fn main() {
        let mut counter: Field = 0_Field;
        let mut arr: [[[Field; 2]; 2]; 2] = [[[0_Field; 2]; 2]; 2];
        {
            let i_6: u32 = inc(&mut counter);
            let i_7: u32 = inc(&mut counter);
            let i_8: u32 = inc(&mut counter);
            arr[i_6][i_7][i_8] = 42_Field;
        };
        assert(counter == 3_Field);
        assert(arr[0_u32][1_u32][2_u32] == 42_Field);
    }
    ");
}

#[test]
fn member_access_then_array_index_ordering() {
    // Tests side-effect ordering when combining member access and array indexing
    let src = r#"
        struct Foo {
            arrays: [[Field; 2]; 2]
        }

        fn inc(c: &mut Field) -> u32 {
            let old = *c;
            *c = *c + 1;
            old as u32
        }

        fn main() {
            let mut counter = 0;
            let mut foo = Foo { arrays: [[0; 2]; 2] };

            foo.arrays[inc(&mut counter)][inc(&mut counter)] = 55;

            assert(counter == 2);
            assert(foo.arrays[0][1] == 55);
        }
    "#;
    assert_no_errors(src);
    let expanded = assert_no_errors_and_to_string(src);
    insta::assert_snapshot!(expanded, @r"
    struct Foo {
        arrays: [[Field; 2]; 2],
    }
    
    fn inc(c: &mut Field) -> u32 {
        let old: Field = *c;
        *c = *c + 1_Field;
        old as u32
    }
    
    fn main() {
        let mut counter: Field = 0_Field;
        let mut foo: Foo = Foo { arrays: [[0_Field; 2]; 2]};
        {
            let i_6: u32 = inc(&mut counter);
            let i_7: u32 = inc(&mut counter);
            foo.arrays[i_6][i_7] = 55_Field;
        };
        assert(counter == 2_Field);
        assert(foo.arrays[0_u32][1_u32] == 55_Field);
    }
    ");
}

// Nested Pattern Tests

#[test]
fn nested_tuple_pattern_destructuring() {
    let src = r#"
        fn main() {
            let tuple = ((1, 2), (3, 4));
            let ((a, b), (c, d)) = tuple;

            assert(a == 1);
            assert(b == 2);
            assert(c == 3);
            assert(d == 4);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_tuple_pattern_partial_destructuring() {
    let src = r#"
        fn main() {
            let tuple = ((1, 2), 3);
            let (_a, _b) = tuple;
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn deeply_nested_tuple_pattern() {
    // 3 levels nested depth
    let src = r#"
        fn main() {
            let nested = (((1, 2), 3), 4);
            let (((a, b), c), d) = nested;

            assert(a == 1);
            assert(b == 2);
            assert(c == 3);
            assert(d == 4);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_tuple_with_mixed_depths() {
    // Tuple patterns with varying nested depths
    let src = r#"
        fn main() {
            let mixed = ((1, 2), 3, (4, (5, 6)));
            let ((a, b), c, (d, (e, f))) = mixed;

            assert(a == 1);
            assert(b == 2);
            assert(c == 3);
            assert(d == 4);
            assert(e == 5);
            assert(f == 6);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn mutable_nested_tuple_pattern() {
    let src = r#"
        fn main() {
            let tuple = ((1, 2), 3);
            let mut ((a, b), c) = tuple;

            a = 10;
            b = 20;
            c = 30;

            assert(a == 10);
            assert(b == 20);
            assert(c == 30);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn mixed_mut_in_nested_pattern() {
    let src = r#"
        fn main() {
            let tuple = ((1, 2), 3);
            let ((mut a, b), c) = tuple;

            a = 10;
            // b and c are immutable

            assert(a == 10);
            assert(b == 2);
            assert(c == 3);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn struct_with_nested_tuple_pattern() {
    let src = r#"
        struct Foo {
            nested: ((Field, Field), Field)
        }

        fn main() {
            let foo = Foo { nested: ((1, 2), 3) };
            let Foo { nested: ((a, b), c) } = foo;

            assert(a == 1);
            assert(b == 2);
            assert(c == 3);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn nested_struct_patterns() {
    let src = r#"
        struct Inner {
            x: Field,
            y: Field
        }

        struct Outer {
            inner: Inner,
            z: Field
        }

        fn main() {
            let outer = Outer {
                inner: Inner { x: 1, y: 2 },
                z: 3
            };

            let Outer { inner: Inner { x, y }, z } = outer;

            assert(x == 1);
            assert(y == 2);
            assert(z == 3);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn struct_pattern_with_mismatched_type() {
    let src = r#"
    struct Foo { x: Field }
    struct Bar { y: u32 }

    fn main() {
        let value: Bar = Bar { y: 5 };
        let Foo { x } = value;
            ^^^^^^^^^ Cannot assign an expression of type Foo to a value of type Bar
        // x should have type Field (from Foo), so the next line should error
        let _check: u32 = x;
                          ^ Expected type u32, found type Field
    }
    "#;
    check_errors(src);
}

#[test]
fn struct_pattern_with_non_struct_type() {
    let src = r#"
    fn main() {
        let x: Field = 1;
        let MyStruct { a: _, b: _ } = x;
            ^^^^^^^^^^^^^^^^^^^^^^^ Cannot assign an expression of type MyStruct to a value of type Field
    }

    struct MyStruct {
        a: Field,
        b: Field,
    }
    "#;
    check_errors(src);
}

#[test]
fn struct_pattern_with_error_type() {
    let src = r#"
    fn main() {
        let value = MyStruct { x: 1, y: 2 };
        let MyStruct { x: _, y: _ }: DoesNotExist = value;
                                     ^^^^^^^^^^^^ Could not resolve 'DoesNotExist' in path
    }

    struct MyStruct {
        x: Field,
        y: Field, 
    }
    "#;
    check_errors(src);
}

#[test]
fn struct_pattern_with_error_type_and_missing_fields() {
    let src = r#"
    fn main() {
        let value = MyStruct { x: 1, y: 2 };
        let MyStruct { x: _ }: DoesNotExist = value;
                               ^^^^^^^^^^^^ Could not resolve 'DoesNotExist' in path
            ^^^^^^^^^^^^^^^^^ missing field y in struct MyStruct
    }

    struct MyStruct {
        x: Field,
        y: Field, 
    }
    "#;
    check_errors(src);
}

#[test]
fn struct_pattern_with_nonexistent_struct() {
    let src = r#"
    fn main() {
        let value = (1, 2);
        let NonExistent { x, y } = value;
            ^^^^^^^^^^^ Could not resolve 'NonExistent' in path
    }
    "#;
    check_errors(src);
}

#[test]
fn struct_pattern_with_non_struct_type_name() {
    let src = r#"
    fn main() {
        let x = 5;
        let Field { value } = x;
            ^^^^^ expected type got primitive type
    }
    "#;
    check_errors(src);
}

#[test]
fn parenthesized_pattern() {
    let src = r#"
        fn main() {
            // Parenthesized identifier
            let (x) = 5;
            assert(x == 5);

            // Parenthesized tuple pattern
            let ((a, b)) = (1, 2);
            assert(a == 1);
            assert(b == 2);

            // Multiple levels of parentheses
            let (((c))) = 10;
            assert(c == 10);

            // Parenthesized struct pattern
            let (Foo { x, y }) = Foo { x: 1, y: 2 };
            assert(x == 1);
            assert(y == 2);
        }

        struct Foo {
            x: Field,
            y: Field,
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn tuple_pattern_arity_mismatch() {
    let src = r#"
        fn main() {
            let tuple = (1, 2, 3);
            let (_a, _b) = tuple;
                ^^^^^^^^ Expected a tuple with 3 elements, found one with 2 elements
                ~~~~~~~~ The expression the tuple is assigned to has type `(Field,Field,Field)`
        }
    "#;
    check_errors(src);
}

#[test]
fn tuple_pattern_with_non_tuple_type() {
    let src = r#"
    fn main() {
        let x: Field = 1;
        let (_a, _b) = x;
            ^^^^^^^^ Cannot assign an expression of type (_, _) to a value of type Field
    }
    "#;
    check_errors(src);
}

#[test]
fn tuple_pattern_with_error_type() {
    // When the expected type is Error (from a previous type resolution failure),
    // we should not issue confusing cascading errors about invalid fields.
    let src = r#"
    fn main() {
        let x = 1;
        let (_a, _b): DoesNotExist = x;
                      ^^^^^^^^^^^^ Could not resolve 'DoesNotExist' in path
    }
    "#;
    check_errors(src);
}

#[test]
fn duplicated_mut_in_basic_let_pattern() {
    let src = r#"
    fn main() {
        let mut mut _x = 1;
                ^^^ `mut` on a binding cannot be repeated
    }
    "#;
    check_errors(src);
}

#[test]
fn duplicated_mut_in_nested_pattern() {
    let src = r#"
    fn main() {
        let mut (_a, mut (_b, _c)) = (1, (2, 3));
                     ^^^^^^^^^^^^ 'mut' here is not necessary
            ~~~~~~~~~~~~~~~~~~~~~~ Pattern was already made mutable from this 'mut'
    }
    "#;
    check_errors(src);
}

#[test]
fn dereference_in_lvalue() {
    let src = r#"
        fn main() {
            dereference_basic();
            dereference_nested();
        }

        fn dereference_basic() {
            let x = &mut 5;
            *x = 10;
            assert(*x == 10);
        }

        fn dereference_nested() {
            let y = &mut &mut 20;
            **y = 30;
            assert(**y == 30);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn reference_chain_in_tuple_member_access() {
    // Ensure that references appearing in the middle of a member access chain are properly dereferenced.
    //
    // 1. `x` has type `(&mut (u32, &mut (u32, u32)), u32)`
    // 2. Accessing `.0` yields `&mut (u32, &mut (u32, u32))`
    // 3. Must dereference to get `(u32, &mut (u32, u32))`
    // 4. Accessing `.1` yields `&mut (u32, u32)`
    // 5. Must dereference to get `(u32, u32)`
    // 6. Accessing `.0` yields `u32`
    let src = r#"
        fn main() {
            let inner = &mut (10, 20);
            let outer = &mut (5, inner);
            let mut x = (outer, 99);

            x.0.1.0 = 42;

            assert(x.0.1.0 == 42);
            assert(x.0.1.1 == 20);
            assert(x.0.0 == 5);
            assert(x.1 == 99);
        }
    "#;
    assert_no_errors(src);
}

#[test]
fn mut_comptime_variable_in_runtime() {
    let src = r#"
    fn main() {
        comptime let mut x = 5;
        x = 10;
        ^ Comptime variable `x` cannot be mutated in a non-comptime context
        ~ `x` mutated here
    }
    "#;
    check_errors(src);
}

#[test]
fn mut_comptime_variable_in_comptime() {
    let src = r#"
    fn main() {
        comptime let mut x = 5;
        let _ = comptime { x = 10; x };
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn assigning_to_string_index() {
    let src = r#"
        fn main() {
            let mut s = "hello";
            s[0] = "x";
            ^ Strings do not support indexed assignment
        }
    "#;
    check_errors(src);
}
