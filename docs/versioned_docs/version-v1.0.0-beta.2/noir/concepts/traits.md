---
title: Traits
description:
  Traits in Noir can be used to abstract out a common interface for functions across
  several data types.
keywords: [noir programming language, traits, interfaces, generic, protocol]
sidebar_position: 14
---

## Overview

Traits in Noir are a useful abstraction similar to interfaces or protocols in other languages. Each trait defines
the interface of several methods contained within the trait. Types can then implement this trait by providing
implementations for these methods. For example in the program:

```rust
struct Rectangle {
    width: Field,
    height: Field,
}

impl Rectangle {
    fn area(self) -> Field {
        self.width * self.height
    }
}

fn log_area(r: Rectangle) {
    println(r.area());
}
```

We have a function `log_area` to log the area of a `Rectangle`. Now how should we change the program if we want this
function to work on `Triangle`s as well?:

```rust
struct Triangle {
    width: Field,
    height: Field,
}

impl Triangle {
    fn area(self) -> Field {
        self.width * self.height / 2
    }
}
```

Making `log_area` generic over all types `T` would be invalid since not all types have an `area` method. Instead, we can
introduce a new `Area` trait and make `log_area` generic over all types `T` that implement `Area`:

```rust
trait Area {
    fn area(self) -> Field;
}

fn log_area<T>(shape: T) where T: Area {
    println(shape.area());
}
```

We also need to explicitly implement `Area` for `Rectangle` and `Triangle`. We can do that by changing their existing
impls slightly. Note that the parameter types and return type of each of our `area` methods must match those defined
by the `Area` trait.

```rust
impl Area for Rectangle {
    fn area(self) -> Field {
        self.width * self.height
    }
}

impl Area for Triangle {
    fn area(self) -> Field {
        self.width * self.height / 2
    }
}
```

Now we have a working program that is generic over any type of Shape that is used! Others can even use this program
as a library with their own types - such as `Circle` - as long as they also implement `Area` for these types.

## Where Clauses

As seen in `log_area` above, when we want to create a function or method that is generic over any type that implements
a trait, we can add a where clause to the generic function.

```rust
fn log_area<T>(shape: T) where T: Area {
    println(shape.area());
}
```

It is also possible to apply multiple trait constraints on the same variable at once by combining traits with the `+`
operator. Similarly, we can have multiple trait constraints by separating each with a comma:

```rust
fn foo<T, U>(elements: [T], thing: U) where
    T: Default + Add + Eq,
    U: Bar,
{
    let mut sum = T::default();

    for element in elements {
        sum += element;
    }

    if sum == T::default() {
        thing.bar();
    }
}
```

## Generic Implementations

You can add generics to a trait implementation by adding the generic list after the `impl` keyword:

```rust
trait Second {
    fn second(self) -> Field;
}

impl<T> Second for (T, Field) {
    fn second(self) -> Field {
        self.1
    }
}
```

You can also implement a trait for every type this way:

```rust
trait Debug {
    fn debug(self);
}

impl<T> Debug for T {
    fn debug(self) {
        println(self);
    }
}

fn main() {
    1.debug();
}
```

### Generic Trait Implementations With Where Clauses

Where clauses can be placed on trait implementations themselves to restrict generics in a similar way.
For example, while `impl<T> Foo for T` implements the trait `Foo` for every type, `impl<T> Foo for T where T: Bar`
will implement `Foo` only for types that also implement `Bar`. This is often used for implementing generic types.
For example, here is the implementation for array equality:

```rust
impl<T, let N: u32> Eq for [T; let N: u32] where T: Eq {
    // Test if two arrays have the same elements.
    // Because both arrays must have length N, we know their lengths already match.
    fn eq(self, other: Self) -> bool {
        let mut result = true;

        for i in 0 .. self.len() {
            // The T: Eq constraint is needed to call == on the array elements here
            result &= self[i] == other[i];
        }

        result
    }
}
```

Where clauses can also be placed on struct implementations. 
For example, here is a method utilizing a generic type that implements the equality trait.

```rust
struct Foo<T> {
    a: u32,
    b: T,
}

impl<T> Foo<T> where T: Eq {
    fn eq(self, other: Self) -> bool {
        (self.a == other.a) & self.b.eq(other.b)
    }
}
```

## Generic Traits

Traits themselves can also be generic by placing the generic arguments after the trait name. These generics are in
scope of every item within the trait.

```rust
trait Into<T> {
    // Convert `self` to type `T`
    fn into(self) -> T;
}
```

When implementing generic traits the generic arguments of the trait must be specified. This is also true anytime
when referencing a generic trait (e.g. in a `where` clause).

```rust
struct MyStruct {
    array: [Field; 2],
}

impl Into<[Field; 2]> for MyStruct {
    fn into(self) -> [Field; 2] {
        self.array
    }
}

fn as_array<T>(x: T) -> [Field; 2] 
    where T: Into<[Field; 2]>
{
    x.into()
}

fn main() {
    let array = [1, 2];
    let my_struct = MyStruct { array };

    assert_eq(as_array(my_struct), array);
}
```

### Associated Types and Constants

Traits also support associated types and constraints which can be thought of as additional generics that are referred to by name.

Here's an example of a trait with an associated type `Foo` and a constant `Bar`:

```rust
trait MyTrait {
    type Foo;

    let Bar: u32;
}
```

Now when we're implementing `MyTrait` we also have to provide values for `Foo` and `Bar`:

```rust
impl MyTrait for Field {
    type Foo = i32;

    let Bar: u32 = 11;
}
```

Since associated constants can also be used in a type position, its values are limited to only other
expression kinds allowed in numeric generics.

When writing a trait constraint, you can specify all associated types and constants explicitly if
you wish:

```rust
fn foo<T>(x: T) where T: MyTrait<Foo = i32, Bar = 11> {
    ...
}
```

Or you can also elide them since there should only be one `Foo` and `Bar` for a given implementation
of `MyTrait` for a type:

```rust
fn foo<T>(x: T) where T: MyTrait {
    ...
}
```

If you elide associated types, you can still refer to them via the type as trait syntax `<T as MyTrait>`:

```rust
fn foo<T>(x: T) where
    T: MyTrait,
    <T as MyTrait>::Foo: Default + Eq
{
    let foo_value: <T as MyTrait>::Foo = Default::default();
    assert_eq(foo_value, foo_value);
}
```

## Trait Methods With No `self`

A trait can contain any number of methods, each of which have access to the `Self` type which represents each type
that eventually implements the trait. Similarly, the `self` variable is available as well but is not required to be used.
For example, we can define a trait to create a default value for a type. This trait will need to return the `Self` type
but doesn't need to take any parameters:

```rust
trait Default {
    fn default() -> Self;
}
```

Implementing this trait can be done similarly to any other trait:

```rust
impl Default for Field {
    fn default() -> Field {
        0
    }
}

struct MyType {}

impl Default for MyType {
    fn default() -> Field {
        MyType {}
    }
}
```

However, since there is no `self` parameter, we cannot call it via the method call syntax `object.method()`.
Instead, we'll need to refer to the function directly. This can be done either by referring to the
specific impl `MyType::default()` or referring to the trait itself `Default::default()`. In the later
case, type inference determines the impl that is selected.

```rust
let my_struct = MyStruct::default();

let x: Field = Default::default();
let result = x + Default::default();
```

:::warning

```rust
let _ = Default::default();
```

If type inference cannot select which impl to use because of an ambiguous `Self` type, an impl will be
arbitrarily selected. This occurs most often when the result of a trait function call with no parameters
is unused. To avoid this, when calling a trait function with no `self` or `Self` parameters or return type,
always refer to it via the implementation type's namespace - e.g. `MyType::default()`.
This is set to change to an error in future Noir versions.

:::

## Default Method Implementations

A trait can also have default implementations of its methods by giving a body to the desired functions.
Note that this body must be valid for all types that may implement the trait. As a result, the only
valid operations on `self` will be operations valid for any type or other operations on the trait itself.

```rust
trait Numeric {
    fn add(self, other: Self) -> Self;

    // Default implementation of double is (self + self)
    fn double(self) -> Self {
        self.add(self)
    }
}
```

When implementing a trait with default functions, a type may choose to implement only the required functions:

```rust
impl Numeric for Field {
    fn add(self, other: Field) -> Field {
        self + other
    }
}
```

Or it may implement the optional methods as well:

```rust
impl Numeric for u32 {
    fn add(self, other: u32) -> u32 {
        self + other
    }

    fn double(self) -> u32 {
        self * 2
    }
}
```

## Impl Specialization

When implementing traits for a generic type it is possible to implement the trait for only a certain combination
of generics. This can be either as an optimization or because those specific generics are required to implement the trait.

```rust
trait Sub {
    fn sub(self, other: Self) -> Self;
}

struct NonZero<T> {
    value: T,
}

impl Sub for NonZero<Field> {
    fn sub(self, other: Self) -> Self {
        let value = self.value - other.value;
        assert(value != 0);
        NonZero { value }
    }
}
```

## Overlapping Implementations

Overlapping implementations are disallowed by Noir to ensure Noir's decision on which impl to select is never ambiguous.
This means if a trait `Foo` is already implemented
by a type `Bar<T>` for all `T`, then we cannot also have a separate impl for `Bar<Field>` (or any other
type argument). Similarly, if there is an impl for all `T` such as `impl<T> Debug for T`, we cannot create
any more impls to `Debug` for other types since it would be ambiguous which impl to choose for any given
method call.

```rust
trait Trait {}

// Previous impl defined here
impl<A, B> Trait for (A, B) {}

// error: Impl for type `(Field, Field)` overlaps with existing impl
impl Trait for (Field, Field) {}
```

## Trait Coherence

Another restriction on trait implementations is coherence. This restriction ensures other crates cannot create
impls that may overlap with other impls, even if several unrelated crates are used as dependencies in the same
program.

The coherence restriction is: to implement a trait, either the trait itself or the object type must be declared
in the crate the impl is in.

In practice this often comes up when using types provided by libraries. If a library provides a type `Foo` that does
not implement a trait in the standard library such as `Default`, you may not `impl Default for Foo` in your own crate.
While restrictive, this prevents later issues or silent changes in the program if the `Foo` library later added its
own impl for `Default`. If you are a user of the `Foo` library in this scenario and need a trait not implemented by the
library your choices are to either submit a patch to the library or use the newtype pattern.

### The Newtype Pattern

The newtype pattern gets around the coherence restriction by creating a new wrapper type around the library type
that we cannot create `impl`s for. Since the new wrapper type is defined in our current crate, we can create
impls for any trait we need on it.

```rust
struct Wrapper {
    foo: some_library::Foo,
}

impl Default for Wrapper {
    fn default() -> Wrapper {
        Wrapper {
            foo: some_library::Foo::new(),
        }
    }
}
```

Since we have an impl for our own type, the behavior of this code will not change even if `some_library` is updated
to provide its own `impl Default for Foo`. The downside of this pattern is that it requires extra wrapping and
unwrapping of values when converting to and from the `Wrapper` and `Foo` types.

### Trait Inheritance

Sometimes, you might need one trait to use another trait’s functionality (like "inheritance" in some other languages). In this case, you can specify this relationship by listing any child traits after the parent trait's name and a colon. Now, whenever the parent trait is implemented it will require the child traits to be implemented as well. A parent trait is also called a "super trait."

```rust
trait Person {
    fn name(self) -> String;
}

// Person is a supertrait of Student.
// Implementing Student requires you to also impl Person.
trait Student: Person {
    fn university(self) -> String;
}

trait Programmer {
    fn fav_language(self) -> String;
}

// CompSciStudent (computer science student) is a subtrait of both Programmer 
// and Student. Implementing CompSciStudent requires you to impl both supertraits.
trait CompSciStudent: Programmer + Student {
    fn git_username(self) -> String;
}
```

### Trait Aliases

Similar to the proposed Rust feature for [trait aliases](https://github.com/rust-lang/rust/blob/4d215e2426d52ca8d1af166d5f6b5e172afbff67/src/doc/unstable-book/src/language-features/trait-alias.md),
Noir supports aliasing one or more traits and using those aliases wherever
traits would normally be used.

```rust
trait Foo {
    fn foo(self) -> Self;
}

trait Bar {
    fn bar(self) -> Self;
}

// Equivalent to:
// trait Baz: Foo + Bar {}
//
// impl<T> Baz for T where T: Foo + Bar {}
trait Baz = Foo + Bar;

// We can use `Baz` to refer to `Foo + Bar`
fn baz<T>(x: T) -> T where T: Baz {
    x.foo().bar()
}
```

#### Generic Trait Aliases

Trait aliases can also be generic by placing the generic arguments after the
trait name. These generics are in scope of every item within the trait alias.

```rust
trait Foo {
    fn foo(self) -> Self;
}

trait Bar<T> {
    fn bar(self) -> T;
}

// Equivalent to:
// trait Baz<T>: Foo + Bar<T> {}
//
// impl<T, U> Baz<T> for U where U: Foo + Bar<T> {}
trait Baz<T> = Foo + Bar<T>;
```

#### Trait Alias Where Clauses

Trait aliases support where clauses to add trait constraints to any of their
generic arguments, e.g. ensuring `T: Baz` for a trait alias `Qux<T>`.

```rust
trait Foo {
    fn foo(self) -> Self;
}

trait Bar<T> {
    fn bar(self) -> T;
}

trait Baz {
    fn baz(self) -> bool;
}

// Equivalent to:
// trait Qux<T>: Foo + Bar<T> where T: Baz {}
//
// impl<T, U> Qux<T> for U where
//     U: Foo + Bar<T>,
//     T: Baz,
// {}
trait Qux<T> = Foo + Bar<T> where T: Baz;
```

Note that while trait aliases support where clauses,
the equivalent traits can fail due to [#6467](https://github.com/noir-lang/noir/issues/6467)

### Visibility

By default, like functions, traits and trait aliases are private to the module
they exist in. You can use `pub` to make the trait public or `pub(crate)` to make
it public to just its crate:

```rust
// This trait is now public
pub trait Trait {}

// This trait alias is now public
pub trait Baz = Foo + Bar;
```

Trait methods have the same visibility as the trait they are in.
