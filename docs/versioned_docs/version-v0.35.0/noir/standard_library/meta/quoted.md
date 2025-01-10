---
title: Quoted
---

`std::meta::quoted` contains methods on the built-in `Quoted` type which represents
quoted token streams and is the result of the `quote { ... }` expression.

## Methods

### as_expr

```rust title="as_expr" showLineNumbers 
comptime fn as_expr(self) -> Option<Expr> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/quoted.nr#L6-L8" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/quoted.nr#L6-L8</a></sub></sup>


Parses the quoted token stream as an expression. Returns `Option::none()` if
the expression failed to parse.

Example:

```rust title="as_expr_example" showLineNumbers 
#[test]
    fn test_expr_as_function_call() {
        comptime
        {
            let expr = quote { foo(42) }.as_expr().unwrap();
            let (_function, args) = expr.as_function_call().unwrap();
            assert_eq(args.len(), 1);
            assert_eq(args[0].as_integer().unwrap(), (42, false));
        }
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/noir_test_success/comptime_expr/src/main.nr#L360-L371" target="_blank" rel="noopener noreferrer">Source code: test_programs/noir_test_success/comptime_expr/src/main.nr#L360-L371</a></sub></sup>


### as_module

```rust title="as_module" showLineNumbers 
comptime fn as_module(self) -> Option<Module> {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/quoted.nr#L11-L13" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/quoted.nr#L11-L13</a></sub></sup>


Interprets this token stream as a module path leading to the name of a module.
Returns `Option::none()` if the module isn't found or this token stream cannot be parsed as a path.

Example:

```rust title="as_module_example" showLineNumbers 
mod baz {
    pub mod qux {}
}

#[test]
fn as_module_test() {
    comptime
    {
        let my_mod = quote { baz::qux }.as_module().unwrap();
        assert_eq(my_mod.name(), quote { qux });
    }
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_module/src/main.nr#L116-L129" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_module/src/main.nr#L116-L129</a></sub></sup>


### as_trait_constraint

```rust title="as_trait_constraint" showLineNumbers 
comptime fn as_trait_constraint(self) -> TraitConstraint {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/quoted.nr#L16-L18" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/quoted.nr#L16-L18</a></sub></sup>


Interprets this token stream as a trait constraint (without an object type).
Note that this function panics instead of returning `Option::none()` if the token
stream does not parse and resolve to a valid trait constraint.

Example:

```rust title="implements_example" showLineNumbers 
fn function_with_where<T>(_x: T) where T: SomeTrait<i32> {
    comptime
    {
        let t = quote { T }.as_type();
        let some_trait_i32 = quote { SomeTrait<i32> }.as_trait_constraint();
        assert(t.implements(some_trait_i32));

        assert(t.get_trait_impl(some_trait_i32).is_none());
    }
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_type/src/main.nr#L154-L165" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_type/src/main.nr#L154-L165</a></sub></sup>


### as_type

```rust title="as_type" showLineNumbers 
comptime fn as_type(self) -> Type {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/quoted.nr#L21-L23" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/quoted.nr#L21-L23</a></sub></sup>


Interprets this token stream as a resolved type. Panics if the token
stream doesn't parse to a type or if the type isn't a valid type in scope.

```rust title="implements_example" showLineNumbers 
fn function_with_where<T>(_x: T) where T: SomeTrait<i32> {
    comptime
    {
        let t = quote { T }.as_type();
        let some_trait_i32 = quote { SomeTrait<i32> }.as_trait_constraint();
        assert(t.implements(some_trait_i32));

        assert(t.get_trait_impl(some_trait_i32).is_none());
    }
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/compile_success_empty/comptime_type/src/main.nr#L154-L165" target="_blank" rel="noopener noreferrer">Source code: test_programs/compile_success_empty/comptime_type/src/main.nr#L154-L165</a></sub></sup>


### tokens

```rust title="tokens" showLineNumbers 
comptime fn tokens(self) -> [Quoted] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/meta/quoted.nr#L26-L28" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/meta/quoted.nr#L26-L28</a></sub></sup>


Returns a slice of the individual tokens that form this token stream.

## Trait Implementations

```rust
impl Eq for Quoted
impl Hash for Quoted
```
