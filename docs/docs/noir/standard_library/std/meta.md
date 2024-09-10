# Module `std::meta`

## unquote

```noir
fn unquote(code: Quoted) -> Quoted
```

Calling unquote as a macro (via `unquote!(arg)`) will unquote
its argument. Since this is the effect `!` already does, `unquote`
itself does not need to do anything besides return its argument.

## type_of

```noir
fn type_of<T>(x: T) -> Type
```

Returns the type of any value

## derive

```noir
fn derive(s: StructDefinition, traits: [TraitDefinition]) -> Quoted
```

## derive_via

```noir
fn derive_via(t: TraitDefinition, f: DeriveFunction)
```

## make_trait_impl

```noir
fn make_trait_impl<Env1, Env2>(s: StructDefinition, trait_name: Quoted, function_signature: Quoted, for_each_field: fn[Env1](Quoted) -> Quoted, join_fields_with: Quoted, body: fn[Env2](Quoted) -> Quoted) -> Quoted
```

`make_impl` is a helper function to make a simple impl, usually while deriving a trait.
This impl has a couple assumptions:
1. The impl only has one function, with the signature `function_signature`
2. The trait itself does not have any generics.

While these assumptions are met, `make_impl` will create an impl from a StructDefinition,
automatically filling in the required generics from the struct, along with the where clause.
The function body is created by mapping each field with `for_each_field` and joining the
results with `join_fields_with`. The result of this is passed to the `body` function for
any final processing - e.g. wrapping each field in a `StructConstructor { .. }` expression.

See `derive_eq` and `derive_default` for example usage.

