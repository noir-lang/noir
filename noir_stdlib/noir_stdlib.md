# Crate std

### Function `println`

<pre><code>pub fn println&lt;T&gt;(input: T)</code></pre>

### Function `print`

<pre><code>pub fn print&lt;T&gt;(input: T)</code></pre>

### Function `verify_proof_with_type`

<pre><code>pub fn verify_proof_with_type&lt;let N: u32, let M: u32, let K: u32&gt;(
    verification_key: [Field; N],
    proof: [Field; M],
    public_inputs: [Field; K],
    key_hash: Field,
    proof_type: u32,
)</code></pre>

Asserts the validity of the provided proof and public inputs against the provided verification key and hash.

The ACVM cannot determine whether the provided proof is valid during execution as this requires knowledge of
the backend against which the program is being proven. However if an invalid proof if submitted, the program may
fail to prove or the backend may generate a proof which will subsequently fail to verify.

# Important Note

If you are not developing your own backend such as [Barretenberg](https://github.com/AztecProtocol/barretenberg)
you probably shouldn't need to interact with this function directly. It's easier and safer to use a verification
library which is published by the developers of the backend which will document or enforce any safety requirements.

If you use this directly, you're liable to introduce underconstrainedness bugs and *your circuit will be insecure*.

# Arguments
- verification_key: The verification key of the circuit to be verified.
- proof: The proof to be verified.
- public_inputs: The public inputs associated with `proof`
- key_hash: The hash of `verification_key` of the form expected by the backend.
- proof_type: An identifier for the proving scheme used to generate the proof to be verified. This allows
              for a single backend to support verifying multiple proving schemes.

# Constraining `key_hash`

The Noir compiler does not by itself constrain that `key_hash` is a valid hash of `verification_key`.
This is because different backends may differ in how they hash their verification keys.
It is then the responsibility of either the noir developer (by explicitly hashing the verification key
in the correct manner) or by the proving system itself internally asserting the correctness of `key_hash`.

### Function `assert_constant`

<pre><code>pub fn assert_constant&lt;T&gt;(x: T)</code></pre>

### Function `static_assert`

<pre><code>pub fn static_assert&lt;T&gt;(predicate: bool, message: T)</code></pre>

### Function `wrapping_add`

<pre><code>pub fn wrapping_add&lt;T&gt;(x: T, y: T) -> T
where
    T: <a href="#AsPrimitive">AsPrimitive</a><Field>,
    Field: <a href="#AsPrimitive">AsPrimitive</a><T>
</code></pre>

### Function `wrapping_sub`

<pre><code>pub fn wrapping_sub&lt;T&gt;(x: T, y: T) -> T
where
    T: <a href="#AsPrimitive">AsPrimitive</a><Field>,
    Field: <a href="#AsPrimitive">AsPrimitive</a><T>
</code></pre>

### Function `wrapping_mul`

<pre><code>pub fn wrapping_mul&lt;T&gt;(x: T, y: T) -> T
where
    T: <a href="#AsPrimitive">AsPrimitive</a><Field>,
    Field: <a href="#AsPrimitive">AsPrimitive</a><T>
</code></pre>

### Function `as_witness`

<pre><code>pub fn as_witness(x: Field)</code></pre>

## Module aes128

### Function `aes128_encrypt`

<pre><code>pub fn aes128_encrypt&lt;let N: u32&gt;(
    input: [u8; N],
    iv: [u8; 16],
    key: [u8; 16],
) -> [u8; N + 16 - N % 16]</code></pre>

## Module append

<a id="Append"></a>

### Trait `Append`

<pre><code>pub trait Append {
}</code></pre>

#### Methods

<pre><code>pub fn empty() -> Self</code></pre>

<pre><code>pub fn append(self: Self, other: Self) -> Self</code></pre>

#### Implementors

<h5><pre><code>impl&lt;T&gt; <a href="#Append">Append</a> for [T]</code></pre></h5>

<h5><pre><code>impl <a href="#Append">Append</a> for Quoted</code></pre></h5>

<h5><pre><code>impl <a href="#Append">Append</a> for CtString</code></pre></h5>

## Module cmp

<a id="Ordering"></a>

### Struct `Ordering`

<pre><code>pub struct Ordering
{ /* private fields */ }
</code></pre>

#### Implementations

<h5><pre><code>impl <a href="#Ordering">Ordering</a></code></pre></h5>

<pre><code>pub fn less() -> <a href="#Ordering">Ordering</a></code></pre>

<pre><code>pub fn equal() -> <a href="#Ordering">Ordering</a></code></pre>

<pre><code>pub fn greater() -> <a href="#Ordering">Ordering</a></code></pre>

#### Trait implementations

<h5><pre><code>impl <a href="#Eq">Eq</a> for <a href="#Ordering">Ordering</a></code></pre></h5>

<a id="Eq"></a>

### Trait `Eq`

<pre><code>pub trait Eq {
}</code></pre>

#### Methods

<pre><code>pub fn eq(self: Self, other: Self) -> bool</code></pre>

#### Implementors

<h5><pre><code>impl <a href="#Eq">Eq</a> for CompoundStruct</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for Field</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for u128</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for u64</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for u32</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for u16</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for u8</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for u1</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for i8</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for i16</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for i32</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for i64</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for ()</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for bool</code></pre></h5>

<h5><pre><code>impl&lt;let N: u32, T&gt; <a href="#Eq">Eq</a> for [T; N]
where
    T: <a href="#Eq">Eq</a>
</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#Eq">Eq</a> for [T]
where
    T: <a href="#Eq">Eq</a>
</code></pre></h5>

<h5><pre><code>impl&lt;let N: u32&gt; <a href="#Eq">Eq</a> for str<N></code></pre></h5>

<h5><pre><code>impl&lt;A, B&gt; <a href="#Eq">Eq</a> for (A, B)
where
    A: <a href="#Eq">Eq</a>,
    B: <a href="#Eq">Eq</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C&gt; <a href="#Eq">Eq</a> for (A, B, C)
where
    A: <a href="#Eq">Eq</a>,
    B: <a href="#Eq">Eq</a>,
    C: <a href="#Eq">Eq</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C, D&gt; <a href="#Eq">Eq</a> for (A, B, C, D)
where
    A: <a href="#Eq">Eq</a>,
    B: <a href="#Eq">Eq</a>,
    C: <a href="#Eq">Eq</a>,
    D: <a href="#Eq">Eq</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C, D, E&gt; <a href="#Eq">Eq</a> for (A, B, C, D, E)
where
    A: <a href="#Eq">Eq</a>,
    B: <a href="#Eq">Eq</a>,
    C: <a href="#Eq">Eq</a>,
    D: <a href="#Eq">Eq</a>,
    E: <a href="#Eq">Eq</a>
</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for <a href="#Ordering">Ordering</a></code></pre></h5>

<h5><pre><code>impl&lt;let MaxLen: u32, T&gt; <a href="#Eq">Eq</a> for <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;
where
    T: <a href="#Eq">Eq</a>
</code></pre></h5>

<h5><pre><code>impl&lt;B, K, let N: u32, V&gt; <a href="#Eq">Eq</a> for <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    V: <a href="#Eq">Eq</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre></h5>

<h5><pre><code>impl&lt;B, K, V&gt; <a href="#Eq">Eq</a> for <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    V: <a href="#Eq">Eq</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for <a href="#EmbeddedCurveScalar">EmbeddedCurveScalar</a></code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for CtString</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for FunctionDefinition</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for Module</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for <a href="#UnaryOp">UnaryOp</a></code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for <a href="#BinaryOp">BinaryOp</a></code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for Quoted</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for TraitConstraint</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for TraitDefinition</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for Type</code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for TypeDefinition</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#Eq">Eq</a> for <a href="#Option">Option</a>&lt;T&gt;
where
    T: <a href="#Eq">Eq</a>
</code></pre></h5>

<a id="Ord"></a>

### Trait `Ord`

<pre><code>pub trait Ord {
}</code></pre>

#### Methods

<pre><code>pub fn cmp(self: Self, other: Self) -> <a href="#Ordering">Ordering</a></code></pre>

#### Implementors

<h5><pre><code>impl <a href="#Ord">Ord</a> for u128</code></pre></h5>

<h5><pre><code>impl <a href="#Ord">Ord</a> for u64</code></pre></h5>

<h5><pre><code>impl <a href="#Ord">Ord</a> for u32</code></pre></h5>

<h5><pre><code>impl <a href="#Ord">Ord</a> for u16</code></pre></h5>

<h5><pre><code>impl <a href="#Ord">Ord</a> for u8</code></pre></h5>

<h5><pre><code>impl <a href="#Ord">Ord</a> for i8</code></pre></h5>

<h5><pre><code>impl <a href="#Ord">Ord</a> for i16</code></pre></h5>

<h5><pre><code>impl <a href="#Ord">Ord</a> for i32</code></pre></h5>

<h5><pre><code>impl <a href="#Ord">Ord</a> for i64</code></pre></h5>

<h5><pre><code>impl <a href="#Ord">Ord</a> for ()</code></pre></h5>

<h5><pre><code>impl <a href="#Ord">Ord</a> for bool</code></pre></h5>

<h5><pre><code>impl&lt;let N: u32, T&gt; <a href="#Ord">Ord</a> for [T; N]
where
    T: <a href="#Ord">Ord</a>
</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#Ord">Ord</a> for [T]
where
    T: <a href="#Ord">Ord</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B&gt; <a href="#Ord">Ord</a> for (A, B)
where
    A: <a href="#Ord">Ord</a>,
    B: <a href="#Ord">Ord</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C&gt; <a href="#Ord">Ord</a> for (A, B, C)
where
    A: <a href="#Ord">Ord</a>,
    B: <a href="#Ord">Ord</a>,
    C: <a href="#Ord">Ord</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C, D&gt; <a href="#Ord">Ord</a> for (A, B, C, D)
where
    A: <a href="#Ord">Ord</a>,
    B: <a href="#Ord">Ord</a>,
    C: <a href="#Ord">Ord</a>,
    D: <a href="#Ord">Ord</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C, D, E&gt; <a href="#Ord">Ord</a> for (A, B, C, D, E)
where
    A: <a href="#Ord">Ord</a>,
    B: <a href="#Ord">Ord</a>,
    C: <a href="#Ord">Ord</a>,
    D: <a href="#Ord">Ord</a>,
    E: <a href="#Ord">Ord</a>
</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#Ord">Ord</a> for <a href="#Option">Option</a>&lt;T&gt;
where
    T: <a href="#Ord">Ord</a>
</code></pre></h5>

### Function `max`

<pre><code>pub fn max&lt;T&gt;(v1: T, v2: T) -> T
where
    T: <a href="#Ord">Ord</a>
</code></pre>

### Function `min`

<pre><code>pub fn min&lt;T&gt;(v1: T, v2: T) -> T
where
    T: <a href="#Ord">Ord</a>
</code></pre>

## Module collections::bounded_vec

<a id="BoundedVec"></a>

### Struct `BoundedVec`

<pre><code>pub struct BoundedVec&lt;T, let MaxLen: u32&gt;
{ /* private fields */ }
</code></pre>
A `BoundedVec<T, MaxLen>` is a growable storage similar to a `Vec<T>` except that it
is bounded with a maximum possible length. Unlike `Vec`, `BoundedVec` is not implemented
via slices and thus is not subject to the same restrictions slices are (notably, nested
slices - and thus nested vectors as well - are disallowed).

Since a BoundedVec is backed by a normal array under the hood, growing the BoundedVec by
pushing an additional element is also more efficient - the length only needs to be increased
by one.

For these reasons `BoundedVec<T, N>` should generally be preferred over `Vec<T>` when there
is a reasonable maximum bound that can be placed on the vector.

Example:

```noir
let mut vector: BoundedVec<Field, 10> = BoundedVec::new();
for i in 0..5 {
    vector.push(i);
}
assert(vector.len() == 5);
assert(vector.max_len() == 10);
```

#### Implementations

<h5><pre><code>impl&lt;let MaxLen: u32, T&gt; <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;</code></pre></h5>

<pre><code>pub fn new() -> <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;</code></pre>

Creates a new, empty vector of length zero.

Since this container is backed by an array internally, it still needs an initial value
to give each element. To resolve this, each element is zeroed internally. This value
is guaranteed to be inaccessible unless `get_unchecked` is used.

Example:

```noir
let empty_vector: BoundedVec<Field, 10> = BoundedVec::new();
assert(empty_vector.len() == 0);
```

Note that whenever calling `new` the maximum length of the vector should always be specified
via a type signature:

```noir
fn good() -> BoundedVec<Field, 10> {
    // Ok! MaxLen is specified with a type annotation
    let v1: BoundedVec<Field, 3> = BoundedVec::new();
    let v2 = BoundedVec::new();

    // Ok! MaxLen is known from the type of `good`'s return value
    v2
}

fn bad() {
    // Error: Type annotation needed
    // The compiler can't infer `MaxLen` from the following code:
    let mut v3 = BoundedVec::new();
    v3.push(5);
}
```

This defaulting of `MaxLen` (and numeric generics in general) to zero may change in future noir versions
but for now make sure to use type annotations when using bounded vectors. Otherwise, you will receive a
constraint failure at runtime when the vec is pushed to.

<pre><code>pub fn get(self: <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;, index: u32) -> T</code></pre>

Retrieves an element from the vector at the given index, starting from zero.

If the given index is equal to or greater than the length of the vector, this
will issue a constraint failure.

Example:

```noir
fn foo<let N: u32>(v: BoundedVec<u32, N>) {
    let first = v.get(0);
    let last = v.get(v.len() - 1);
    assert(first != last);
}
```

<pre><code>pub fn get_unchecked(self: <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;, index: u32) -> T</code></pre>

Retrieves an element from the vector at the given index, starting from zero, without
performing a bounds check.

Since this function does not perform a bounds check on length before accessing the element,
it is unsafe! Use at your own risk!

Example:

```noir
fn sum_of_first_three<let N: u32>(v: BoundedVec<u32, N>) -> u32 {
    // Always ensure the length is larger than the largest
    // index passed to get_unchecked
    assert(v.len() > 2);
    let first = v.get_unchecked(0);
    let second = v.get_unchecked(1);
    let third = v.get_unchecked(2);
    first + second + third
}
```

<pre><code>pub fn set(
    &mut self: &mut <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;,
    index: u32,
    value: T,
)</code></pre>

Writes an element to the vector at the given index, starting from zero.

If the given index is equal to or greater than the length of the vector, this will issue a constraint failure.

Example:

```noir
fn foo<let N: u32>(v: BoundedVec<u32, N>) {
    let first = v.get(0);
    assert(first != 42);
    v.set(0, 42);
    let new_first = v.get(0);
    assert(new_first == 42);
}
```

<pre><code>pub fn set_unchecked(
    &mut self: &mut <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;,
    index: u32,
    value: T,
)</code></pre>

Writes an element to the vector at the given index, starting from zero, without performing a bounds check.

Since this function does not perform a bounds check on length before accessing the element, it is unsafe! Use at your own risk!

Example:

```noir
fn set_unchecked_example() {
    let mut vec: BoundedVec<u32, 5> = BoundedVec::new();
    vec.extend_from_array([1, 2]);

    // Here we're safely writing within the valid range of `vec`
    // `vec` now has the value [42, 2]
    vec.set_unchecked(0, 42);

    // We can then safely read this value back out of `vec`.
    // Notice that we use the checked version of `get` which would prevent reading unsafe values.
    assert_eq(vec.get(0), 42);

    // We've now written past the end of `vec`.
    // As this index is still within the maximum potential length of `v`,
    // it won't cause a constraint failure.
    vec.set_unchecked(2, 42);
    println(vec);

    // This will write past the end of the maximum potential length of `vec`,
    // it will then trigger a constraint failure.
    vec.set_unchecked(5, 42);
    println(vec);
}
```

<pre><code>pub fn push(&mut self: &mut <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;, elem: T)</code></pre>

Pushes an element to the end of the vector. This increases the length
of the vector by one.

Panics if the new length of the vector will be greater than the max length.

Example:

```noir
let mut v: BoundedVec<Field, 2> = BoundedVec::new();

v.push(1);
v.push(2);

// Panics with failed assertion "push out of bounds"
v.push(3);
```

<pre><code>pub fn len(self: <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;) -> u32</code></pre>

Returns the current length of this vector

Example:

```noir
let mut v: BoundedVec<Field, 4> = BoundedVec::new();
assert(v.len() == 0);

v.push(100);
assert(v.len() == 1);

v.push(200);
v.push(300);
v.push(400);
assert(v.len() == 4);

let _ = v.pop();
let _ = v.pop();
assert(v.len() == 2);
```

<pre><code>pub fn max_len(_self: <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;) -> u32</code></pre>

Returns the maximum length of this vector. This is always
equal to the `MaxLen` parameter this vector was initialized with.

Example:

```noir
let mut v: BoundedVec<Field, 5> = BoundedVec::new();

assert(v.max_len() == 5);
v.push(10);
assert(v.max_len() == 5);
```

<pre><code>pub fn storage(self: <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;) -> [T; MaxLen]</code></pre>

Returns the internal array within this vector.

Since arrays in Noir are immutable, mutating the returned storage array will not mutate
the storage held internally by this vector.

Note that uninitialized elements may be zeroed out!

Example:

```noir
let mut v: BoundedVec<Field, 5> = BoundedVec::new();

assert(v.storage() == [0, 0, 0, 0, 0]);

v.push(57);
assert(v.storage() == [57, 0, 0, 0, 0]);
```

<pre><code>pub fn extend_from_array&lt;let Len: u32&gt;(&mut self: &mut <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;, array: [T; Len])</code></pre>

Pushes each element from the given array to this vector.

Panics if pushing each element would cause the length of this vector
to exceed the maximum length.

Example:

```noir
let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
vec.extend_from_array([2, 4]);

assert(vec.len == 2);
assert(vec.get(0) == 2);
assert(vec.get(1) == 4);
```

<pre><code>pub fn extend_from_slice(&mut self: &mut <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;, slice: [T])</code></pre>

Pushes each element from the given slice to this vector.

Panics if pushing each element would cause the length of this vector
to exceed the maximum length.

Example:

```noir
let mut vec: BoundedVec<Field, 3> = BoundedVec::new();
vec.extend_from_slice(&[2, 4]);

assert(vec.len == 2);
assert(vec.get(0) == 2);
assert(vec.get(1) == 4);
```

<pre><code>pub fn extend_from_bounded_vec&lt;let Len: u32&gt;(&mut self: &mut <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;, vec: <a href="#BoundedVec">BoundedVec</a>&lt;T, Len&gt;)</code></pre>

Pushes each element from the other vector to this vector. The length of
the other vector is left unchanged.

Panics if pushing each element would cause the length of this vector
to exceed the maximum length.

```noir
let mut v1: BoundedVec<Field, 5> = BoundedVec::new();
let mut v2: BoundedVec<Field, 7> = BoundedVec::new();

v2.extend_from_array([1, 2, 3]);
v1.extend_from_bounded_vec(v2);

assert(v1.storage() == [1, 2, 3, 0, 0]);
assert(v2.storage() == [1, 2, 3, 0, 0, 0, 0]);
```

<pre><code>pub fn from_array&lt;let Len: u32&gt;(array: [T; Len]) -> <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;</code></pre>

Creates a new vector, populating it with values derived from an array input.
The maximum length of the vector is determined based on the type signature.

Example:

```noir
let bounded_vec: BoundedVec<Field, 10> = BoundedVec::from_array([1, 2, 3])
```

<pre><code>pub fn pop(&mut self: &mut <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;) -> T</code></pre>

Pops the element at the end of the vector. This will decrease the length
of the vector by one.

Panics if the vector is empty.

Example:

```noir
let mut v: BoundedVec<Field, 2> = BoundedVec::new();
v.push(1);
v.push(2);

let two = v.pop();
let one = v.pop();

assert(two == 2);
assert(one == 1);

// error: cannot pop from an empty vector
let _ = v.pop();
```

<pre><code>pub fn any&lt;Env&gt;(self: <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;, predicate: fn[Env](T) -> bool) -> bool</code></pre>

Returns true if the given predicate returns true for any element
in this vector.

Example:

```noir
let mut v: BoundedVec<u32, 3> = BoundedVec::new();
v.extend_from_array([2, 4, 6]);

let all_even = !v.any(|elem: u32| elem % 2 != 0);
assert(all_even);
```

<pre><code>pub fn map&lt;U, Env&gt;(self: <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;, f: fn[Env](T) -> U) -> <a href="#BoundedVec">BoundedVec</a>&lt;U, MaxLen&gt;</code></pre>

Creates a new vector of equal size by calling a closure on each element in this vector.

Example:

```noir
let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
let result = vec.map(|value| value * 2);

let expected = BoundedVec::from_array([2, 4, 6, 8]);
assert_eq(result, expected);
```

<pre><code>pub fn mapi&lt;U, Env&gt;(self: <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;, f: fn[Env](u32, T) -> U) -> <a href="#BoundedVec">BoundedVec</a>&lt;U, MaxLen&gt;</code></pre>

Creates a new vector of equal size by calling a closure on each element
in this vector, along with its index.

Example:

```noir
let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
let result = vec.mapi(|i, value| i + value * 2);

let expected = BoundedVec::from_array([2, 5, 8, 11]);
assert_eq(result, expected);
```

<pre><code>pub fn for_each&lt;Env&gt;(self: <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;, f: fn[Env](T))</code></pre>

Calls a closure on each element in this vector.

Example:

```noir
let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
let mut result = BoundedVec::<u32, 4>::new();
vec.for_each(|value| result.push(value * 2));

let expected = BoundedVec::from_array([2, 4, 6, 8]);
assert_eq(result, expected);
```

<pre><code>pub fn for_eachi&lt;Env&gt;(self: <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;, f: fn[Env](u32, T))</code></pre>

Calls a closure on each element in this vector, along with its index.

Example:

```noir
let vec: BoundedVec<u32, 4> = BoundedVec::from_array([1, 2, 3, 4]);
let mut result = BoundedVec::<u32, 4>::new();
vec.for_eachi(|i, value| result.push(i + value * 2));

let expected = BoundedVec::from_array([2, 5, 8, 11]);
assert_eq(result, expected);
```

<pre><code>pub fn from_parts(array: [T; MaxLen], len: u32) -> <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;</code></pre>

Creates a new BoundedVec from the given array and length.
The given length must be less than or equal to the length of the array.

This function will zero out any elements at or past index `len` of `array`.
This incurs an extra runtime cost of O(MaxLen). If you are sure your array is
zeroed after that index, you can use `from_parts_unchecked` to remove the extra loop.

Example:

```noir
let vec: BoundedVec<u32, 4> = BoundedVec::from_parts([1, 2, 3, 0], 3);
assert_eq(vec.len(), 3);
```

<pre><code>pub fn from_parts_unchecked(array: [T; MaxLen], len: u32) -> <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;</code></pre>

Creates a new BoundedVec from the given array and length.
The given length must be less than or equal to the length of the array.

This function is unsafe because it expects all elements past the `len` index
of `array` to be zeroed, but does not check for this internally. Use `from_parts`
for a safe version of this function which does zero out any indices past the
given length. Invalidating this assumption can notably cause `BoundedVec::eq`
to give incorrect results since it will check even elements past `len`.

Example:

```noir
let vec: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 0], 3);
assert_eq(vec.len(), 3);

// invalid use!
let vec1: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 1], 3);
let vec2: BoundedVec<u32, 4> = BoundedVec::from_parts_unchecked([1, 2, 3, 2], 3);

// both vecs have length 3 so we'd expect them to be equal, but this
// fails because elements past the length are still checked in eq
assert_eq(vec1, vec2); // fails
```

#### Trait implementations

<h5><pre><code>impl&lt;let MaxLen: u32, T&gt; <a href="#Eq">Eq</a> for <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;
where
    T: <a href="#Eq">Eq</a>
</code></pre></h5>

<h5><pre><code>impl&lt;let Len: u32, let MaxLen: u32, T&gt; <a href="#From">From</a>&lt;[T; Len]&gt; for <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;</code></pre></h5>

## Module collections::map

<a id="HashMap"></a>

### Struct `HashMap`

<pre><code>pub struct HashMap&lt;K, V, let N: u32, B&gt;
{ /* private fields */ }
</code></pre>
`HashMap<Key, Value, MaxLen, Hasher>` is used to efficiently store and look up key-value pairs.

`HashMap` is a bounded type which can store anywhere from zero to `MaxLen` total elements.
Note that due to hash collisions, the actual maximum number of elements stored by any particular
hashmap is likely lower than `MaxLen`. This is true even with cryptographic hash functions since
every hash value will be performed modulo `MaxLen`.

Example:

```noir
// Create a mapping from Fields to u32s with a maximum length of 12
// using a poseidon2 hasher
use std::hash::poseidon2::Poseidon2Hasher;
let mut map: HashMap<Field, u32, 12, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();

map.insert(1, 2);
map.insert(3, 4);

let two = map.get(1).unwrap();
```

#### Implementations

<h5><pre><code>impl&lt;B, K, let N: u32, V&gt; <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;</code></pre></h5>

<pre><code>pub fn with_hasher(_build_hasher: B) -> <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;
where
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

Creates a hashmap with an existing `BuildHasher`. This can be used to ensure multiple
hashmaps are created with the same hasher instance.

Example:

```noir
let my_hasher: BuildHasherDefault<Poseidon2Hasher> = Default::default();
let hashmap: HashMap<u8, u32, 10, BuildHasherDefault<Poseidon2Hasher>> = HashMap::with_hasher(my_hasher);
assert(hashmap.is_empty());
```

<pre><code>pub fn clear(&mut self: &mut <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;)</code></pre>

Clears the hashmap, removing all key-value pairs from it.

Example:

```noir
assert(!map.is_empty());
map.clear();
assert(map.is_empty());
```

<pre><code>pub fn contains_key(self: <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;, key: K) -> bool
where
    K: <a href="#Hash">Hash</a>,
    K: <a href="#Eq">Eq</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

Returns `true` if the hashmap contains the given key. Unlike `get`, this will not also return
the value associated with the key.

Example:

```noir
if map.contains_key(7) {
    let value = map.get(7);
    assert(value.is_some());
} else {
    println("No value for key 7!");
}
```

<pre><code>pub fn is_empty(self: <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;) -> bool</code></pre>

Returns `true` if the length of the hash map is empty.

Example:

```noir
assert(map.is_empty());

map.insert(1, 2);
assert(!map.is_empty());

map.remove(1);
assert(map.is_empty());
```

<pre><code>pub fn entries(self: <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;) -> <a href="#BoundedVec">BoundedVec</a>&lt;(K, V), N&gt;</code></pre>

Returns a vector of each key-value pair present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

```noir
let entries = map.entries();

// The length of a hashmap may not be compile-time known, so we
// need to loop over its capacity instead
for i in 0..map.capacity() {
    if i < entries.len() {
        let (key, value) = entries.get(i);
        println(f"{key} -> {value}");
    }
}
```

<pre><code>pub fn keys(self: <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;) -> <a href="#BoundedVec">BoundedVec</a>&lt;K, N&gt;</code></pre>

Returns a vector of each key present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

```noir
let keys = map.keys();

for i in 0..keys.max_len() {
    if i < keys.len() {
        let key = keys.get_unchecked(i);
        let value = map.get(key).unwrap_unchecked();
        println(f"{key} -> {value}");
    }
}
```

<pre><code>pub fn values(self: <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;) -> <a href="#BoundedVec">BoundedVec</a>&lt;V, N&gt;</code></pre>

Returns a vector of each value present in the hashmap.

The length of the returned vector is always equal to the length of the hashmap.

Example:

```noir
let values = map.values();

for i in 0..values.max_len() {
    if i < values.len() {
        let value = values.get_unchecked(i);
        println(f"Found value {value}");
    }
}
```

<pre><code>pub fn iter_mut(&mut self: &mut <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;, f: fn(K, V) -> (K, V))
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

Iterates through each key-value pair of the HashMap, setting each key-value pair to the
result returned from the given function.

Note that since keys can be mutated, the HashMap needs to be rebuilt as it is iterated
through. If this is not desired, use `iter_values_mut` if only values need to be mutated,
or `entries` if neither keys nor values need to be mutated.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

```noir
// Add 1 to each key in the map, and double the value associated with that key.
map.iter_mut(|k, v| (k + 1, v * 2));
```

<pre><code>pub fn iter_keys_mut(&mut self: &mut <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;, f: fn(K) -> K)
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

Iterates through the HashMap, mutating each key to the result returned from
the given function.

Note that since keys can be mutated, the HashMap needs to be rebuilt as it is iterated
through. If only iteration is desired and the keys are not intended to be mutated,
prefer using `entries` instead.

The iteration order is left unspecified. As a result, if two keys are mutated to become
equal, which of the two values that will be present for the key in the resulting map is also unspecified.

Example:

```noir
// Double each key, leaving the value associated with that key untouched
map.iter_keys_mut(|k| k * 2);
```

<pre><code>pub fn iter_values_mut(&mut self: &mut <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;, f: fn(V) -> V)</code></pre>

Iterates through the HashMap, applying the given function to each value and mutating the
value to equal the result. This function is more efficient than `iter_mut` and `iter_keys_mut`
because the keys are untouched and the underlying hashmap thus does not need to be reordered.

Example:

```noir
// Halve each value
map.iter_values_mut(|v| v / 2);
```

<pre><code>pub fn retain(&mut self: &mut <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;, f: fn(K, V) -> bool)</code></pre>

Retains only the key-value pairs for which the given function returns true.
Any key-value pairs for which the function returns false will be removed from the map.

Example:

```noir
map.retain(|k, v| (k != 0) & (v != 0));
```

<pre><code>pub fn len(self: <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;) -> u32</code></pre>

Returns the current length of this hash map.

Example:

```noir
// This is equivalent to checking map.is_empty()
assert(map.len() == 0);

map.insert(1, 2);
map.insert(3, 4);
map.insert(5, 6);
assert(map.len() == 3);

// 3 was already present as a key in the hash map, so the length is unchanged
map.insert(3, 7);
assert(map.len() == 3);

map.remove(1);
assert(map.len() == 2);
```

<pre><code>pub fn capacity(_self: <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;) -> u32</code></pre>

Returns the maximum capacity of this hashmap. This is always equal to the capacity
specified in the hashmap's type.

Unlike hashmaps in general purpose programming languages, hashmaps in Noir have a
static capacity that does not increase as the map grows larger. Thus, this capacity
is also the maximum possible element count that can be inserted into the hashmap.
Due to hash collisions (modulo the hashmap length), it is likely the actual maximum
element count will be lower than the full capacity.

Example:

```noir
let empty_map: HashMap<Field, Field, 42, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
assert(empty_map.len() == 0);
assert(empty_map.capacity() == 42);
```

<pre><code>pub fn get(self: <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;, key: K) -> <a href="#Option">Option</a>&lt;V&gt;
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

Retrieves a value from the hashmap, returning `Option::none()` if it was not found.

Example:

```noir
fn get_example(map: HashMap<Field, Field, 5, BuildHasherDefault<Poseidon2Hasher>>) {
    let x = map.get(12);

    if x.is_some() {
        assert(x.unwrap() == 42);
    }
}
```

<pre><code>pub fn insert(
    &mut self: &mut <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;,
    key: K,
    value: V,
)
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

Inserts a new key-value pair into the map. If the key was already in the map, its
previous value will be overridden with the newly provided one.

Example:

```noir
let mut map: HashMap<Field, Field, 5, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
map.insert(12, 42);
assert(map.len() == 1);
```

<pre><code>pub fn remove(&mut self: &mut <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;, key: K)
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

Removes the given key-value pair from the map. If the key was not already present
in the map, this does nothing.

Example:

```noir
let mut map: HashMap<Field, Field, 5, BuildHasherDefault<Poseidon2Hasher>> = HashMap::default();
map.insert(12, 42);
assert(!map.is_empty());

map.remove(12);
assert(map.is_empty());

// If a key was not present in the map, remove does nothing
map.remove(12);
assert(map.is_empty());
```

#### Trait implementations

<h5><pre><code>impl&lt;B, K, let N: u32, V&gt; <a href="#Eq">Eq</a> for <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    V: <a href="#Eq">Eq</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre></h5>

<h5><pre><code>impl&lt;B, K, let N: u32, V&gt; <a href="#Default">Default</a> for <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;
where
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>,
    B: <a href="#Default">Default</a>
</code></pre></h5>

## Module collections::umap

<a id="UHashMap"></a>

### Struct `UHashMap`

<pre><code>pub struct UHashMap&lt;K, V, B&gt;
{ /* private fields */ }
</code></pre>

#### Implementations

<h5><pre><code>impl&lt;B, K, V&gt; <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;</code></pre></h5>

<pre><code>pub fn with_hasher(_build_hasher: B) -> <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;
where
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

<pre><code>pub fn with_hasher_and_capacity(_build_hasher: B, capacity: u32) -> <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;
where
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

<pre><code>pub fn clear(&mut self: &mut <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;)</code></pre>

<pre><code>pub fn contains_key(self: <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;, key: K) -> bool
where
    K: <a href="#Hash">Hash</a>,
    K: <a href="#Eq">Eq</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

<pre><code>pub fn is_empty(self: <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;) -> bool</code></pre>

<pre><code>pub fn entries(self: <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;) -> [(K, V)]</code></pre>

<pre><code>pub fn keys(self: <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;) -> [K]</code></pre>

<pre><code>pub fn values(self: <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;) -> [V]</code></pre>

<pre><code>pub unconstrained fn iter_mut(&mut self: &mut <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;, f: fn(K, V) -> (K, V))
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

<pre><code>pub unconstrained fn iter_keys_mut(&mut self: &mut <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;, f: fn(K) -> K)
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

<pre><code>pub fn iter_values_mut(&mut self: &mut <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;, f: fn(V) -> V)</code></pre>

<pre><code>pub fn retain(&mut self: &mut <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;, f: fn(K, V) -> bool)</code></pre>

<pre><code>pub fn len(self: <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;) -> u32</code></pre>

<pre><code>pub fn capacity(self: <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;) -> u32</code></pre>

<pre><code>pub unconstrained fn get(self: <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;, key: K) -> <a href="#Option">Option</a>&lt;V&gt;
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

<pre><code>pub unconstrained fn insert(
    &mut self: &mut <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;,
    key: K,
    value: V,
)
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

<pre><code>pub unconstrained fn remove(&mut self: &mut <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;, key: K)
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre>

#### Trait implementations

<h5><pre><code>impl&lt;B, K, V&gt; <a href="#Eq">Eq</a> for <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;
where
    K: <a href="#Eq">Eq</a>,
    K: <a href="#Hash">Hash</a>,
    V: <a href="#Eq">Eq</a>,
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>
</code></pre></h5>

<h5><pre><code>impl&lt;B, K, V&gt; <a href="#Default">Default</a> for <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;
where
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>,
    B: <a href="#Default">Default</a>
</code></pre></h5>

## Module collections::vec

<a id="Vec"></a>

### Struct `Vec`

<pre><code>pub struct Vec&lt;T&gt;
{ /* private fields */ }
</code></pre>

#### Implementations

<h5><pre><code>impl&lt;T&gt; <a href="#Vec">Vec</a>&lt;T&gt;</code></pre></h5>

<pre><code>pub fn new() -> <a href="#Vec">Vec</a>&lt;T&gt;</code></pre>

<pre><code>pub fn from_slice(slice: [T]) -> <a href="#Vec">Vec</a>&lt;T&gt;</code></pre>

<pre><code>pub fn get(self: <a href="#Vec">Vec</a>&lt;T&gt;, index: u32) -> T</code></pre>

Get an element from the vector at the given index.
Panics if the given index
points beyond the end of the vector.

<pre><code>pub fn set(
    &mut self: &mut <a href="#Vec">Vec</a>&lt;T&gt;,
    index: u32,
    value: T,
)</code></pre>

Write an element to the vector at the given index.
Panics if the given index points beyond the end of the vector (`self.len()`).

<pre><code>pub fn push(&mut self: &mut <a href="#Vec">Vec</a>&lt;T&gt;, elem: T)</code></pre>

Push a new element to the end of the vector, returning a
new vector with a length one greater than the
original unmodified vector.

<pre><code>pub fn pop(&mut self: &mut <a href="#Vec">Vec</a>&lt;T&gt;) -> T</code></pre>

Pop an element from the end of the given vector, returning
a new vector with a length of one less than the given vector,
as well as the popped element.
Panics if the given vector's length is zero.

<pre><code>pub fn insert(
    &mut self: &mut <a href="#Vec">Vec</a>&lt;T&gt;,
    index: u32,
    elem: T,
)</code></pre>

Insert an element at a specified index, shifting all elements
after it to the right

<pre><code>pub fn remove(&mut self: &mut <a href="#Vec">Vec</a>&lt;T&gt;, index: u32) -> T</code></pre>

Remove an element at a specified index, shifting all elements
after it to the left, returning the removed element

<pre><code>pub fn len(self: <a href="#Vec">Vec</a>&lt;T&gt;) -> u32</code></pre>

Returns the number of elements in the vector

## Module compat

### Function `is_bn254`

<pre><code>pub fn is_bn254() -> bool</code></pre>

## Module convert

<a id="From"></a>

### Trait `From`

<pre><code>pub trait From&lt;T&gt; {
}</code></pre>

#### Methods

<pre><code>pub fn from(input: T) -> Self</code></pre>

#### Implementors

<h5><pre><code>impl&lt;let N: u32&gt; <a href="#From">From</a>&lt;str<N>&gt; for [u8; N]</code></pre></h5>

<h5><pre><code>impl&lt;let Len: u32, let MaxLen: u32, T&gt; <a href="#From">From</a>&lt;[T; Len]&gt; for <a href="#BoundedVec">BoundedVec</a>&lt;T, MaxLen&gt;</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#From">From</a>&lt;T&gt; for T</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u8&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u8&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u16&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u8&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u16&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u32&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u8&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u16&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u32&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u64&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u8&gt; for Field</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u16&gt; for Field</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u32&gt; for Field</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u64&gt; for Field</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;u128&gt; for Field</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;i8&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;i8&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;i16&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;i8&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;i16&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;i32&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;bool&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;bool&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;bool&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;bool&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;bool&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;bool&gt; for i8</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;bool&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;bool&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;bool&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#From">From</a>&lt;bool&gt; for Field</code></pre></h5>

<h5><pre><code>impl&lt;let N: u32&gt; <a href="#From">From</a>&lt;[u8; N]&gt; for str<N></code></pre></h5>

<a id="Into"></a>

### Trait `Into`

<pre><code>pub trait Into&lt;T&gt; {
}</code></pre>

#### Methods

<pre><code>pub fn into(self: Self) -> T</code></pre>

#### Implementors

<h5><pre><code>impl&lt;T, U&gt; <a href="#Into">Into</a>&lt;T&gt; for U
where
    T: <a href="#From">From</a><U>
</code></pre></h5>

<a id="AsPrimitive"></a>

### Trait `AsPrimitive`

<pre><code>pub trait AsPrimitive&lt;T&gt; {
}</code></pre>

A generic interface for casting between primitive types,
equivalent of using the `as` keyword between values.

# Example

```
let x: Field = 1234567890;
let y: u8 = x as u8;
let z: u8 = x.as_();
assert_eq(y, z);
```

#### Methods

<pre><code>pub fn as_(self: Self) -> T</code></pre>

The equivalent of doing `self as T`.

#### Implementors

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u8&gt; for bool</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i8&gt; for bool</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u32&gt; for bool</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i32&gt; for bool</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;bool&gt; for bool</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u128&gt; for bool</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u16&gt; for bool</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i16&gt; for bool</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u64&gt; for bool</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i64&gt; for bool</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u32&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i32&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;bool&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u128&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u16&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i16&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u64&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i64&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u8&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i8&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u32&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i32&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;bool&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u128&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u16&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i16&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u64&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i64&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u8&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i8&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u32&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i32&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;bool&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u128&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u16&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i16&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u64&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i64&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u8&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i8&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u32&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i32&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;bool&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u128&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u16&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i16&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u64&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i64&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u8&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i8&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u32&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i32&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;bool&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u128&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u16&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i16&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u64&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i64&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u8&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i8&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;bool&gt; for i8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u128&gt; for i8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u16&gt; for i8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i16&gt; for i8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u64&gt; for i8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i64&gt; for i8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u8&gt; for i8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i8&gt; for i8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u32&gt; for i8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i32&gt; for i8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;bool&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u128&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u16&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i16&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u64&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i64&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u8&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i8&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u32&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i32&gt; for i16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;bool&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u128&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u16&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i16&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u64&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i64&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u8&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i8&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u32&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i32&gt; for i32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;bool&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u128&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u16&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i16&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u64&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i64&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u8&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i8&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u32&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;i32&gt; for i64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;Field&gt; for bool</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;Field&gt; for u8</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;Field&gt; for u16</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;Field&gt; for u32</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;Field&gt; for u64</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;Field&gt; for u128</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u64&gt; for Field</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u32&gt; for Field</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u16&gt; for Field</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u8&gt; for Field</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;bool&gt; for Field</code></pre></h5>

<h5><pre><code>impl <a href="#AsPrimitive">AsPrimitive</a>&lt;u128&gt; for Field</code></pre></h5>

## Module default

<a id="Default"></a>

### Trait `Default`

<pre><code>pub trait Default {
}</code></pre>

#### Methods

<pre><code>pub fn default() -> Self</code></pre>

#### Implementors

<h5><pre><code>impl&lt;K, V&gt; <a href="#Default">Default</a> for Slot&lt;K, V&gt;</code></pre></h5>

<h5><pre><code>impl&lt;B, K, let N: u32, V&gt; <a href="#Default">Default</a> for <a href="#HashMap">HashMap</a>&lt;K, V, N, B&gt;
where
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>,
    B: <a href="#Default">Default</a>
</code></pre></h5>

<h5><pre><code>impl&lt;K, V&gt; <a href="#Default">Default</a> for Slot&lt;K, V&gt;</code></pre></h5>

<h5><pre><code>impl&lt;B, K, V&gt; <a href="#Default">Default</a> for <a href="#UHashMap">UHashMap</a>&lt;K, V, B&gt;
where
    B: <a href="#BuildHasher">BuildHasher</a><H = <B as BuildHasher>::H>,
    B: <a href="#Default">Default</a>
</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for Field</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for u1</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for u8</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for u16</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for u32</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for u64</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for u128</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for i8</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for i16</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for i32</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for i64</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for ()</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for bool</code></pre></h5>

<h5><pre><code>impl&lt;let N: u32, T&gt; <a href="#Default">Default</a> for [T; N]
where
    T: <a href="#Default">Default</a>
</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#Default">Default</a> for [T]</code></pre></h5>

<h5><pre><code>impl&lt;A, B&gt; <a href="#Default">Default</a> for (A, B)
where
    A: <a href="#Default">Default</a>,
    B: <a href="#Default">Default</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C&gt; <a href="#Default">Default</a> for (A, B, C)
where
    A: <a href="#Default">Default</a>,
    B: <a href="#Default">Default</a>,
    C: <a href="#Default">Default</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C, D&gt; <a href="#Default">Default</a> for (A, B, C, D)
where
    A: <a href="#Default">Default</a>,
    B: <a href="#Default">Default</a>,
    C: <a href="#Default">Default</a>,
    D: <a href="#Default">Default</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C, D, E&gt; <a href="#Default">Default</a> for (A, B, C, D, E)
where
    A: <a href="#Default">Default</a>,
    B: <a href="#Default">Default</a>,
    C: <a href="#Default">Default</a>,
    D: <a href="#Default">Default</a>,
    E: <a href="#Default">Default</a>
</code></pre></h5>

<h5><pre><code>impl&lt;H&gt; <a href="#Default">Default</a> for <a href="#BuildHasherDefault">BuildHasherDefault</a>&lt;H&gt;
where
    H: <a href="#Hasher">Hasher</a>,
    H: <a href="#Default">Default</a>
</code></pre></h5>

<h5><pre><code>impl <a href="#Default">Default</a> for Poseidon2Hasher</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#Default">Default</a> for <a href="#Option">Option</a>&lt;T&gt;</code></pre></h5>

## Module ecdsa_secp256k1

### Function `verify_signature`

<pre><code>pub fn verify_signature(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8; 32],
) -> bool</code></pre>

Verifies a ECDSA signature over the secp256k1 curve.
- inputs:
    - x coordinate of public key as 32 bytes
    - y coordinate of public key as 32 bytes
    - the signature, as a 64 bytes array
      The signature internally will be represented as `(r, s)`,
      where `r` and `s` are fixed-sized big endian scalar values.
      As the `secp256k1` has a 256-bit modulus, we have a 64 byte signature
      while `r` and `s` will both be 32 bytes.
      We expect `s` to be normalized. This means given the curve's order,
      `s` should be less than or equal to `order / 2`.
      This is done to prevent malleability.
      For more context regarding malleability you can reference BIP 0062.
    - the hash of the message, as a vector of bytes
- output: false for failure and true for success

### Function `_verify_signature`

<pre><code>pub fn _verify_signature(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8; 32],
    predicate: bool,
) -> bool</code></pre>

## Module ecdsa_secp256r1

### Function `verify_signature`

<pre><code>pub fn verify_signature(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8; 32],
) -> bool</code></pre>

### Function `_verify_signature`

<pre><code>pub fn _verify_signature(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8; 32],
    predicate: bool,
) -> bool</code></pre>

## Module embedded_curve_ops

<a id="EmbeddedCurvePoint"></a>

### Struct `EmbeddedCurvePoint`

<pre><code>pub struct EmbeddedCurvePoint {
    pub x: Field,
    pub y: Field,
    pub is_infinite: bool,
}
</code></pre>
A point on the embedded elliptic curve
By definition, the base field of the embedded curve is the scalar field of the proof system curve, i.e the Noir Field.
x and y denotes the Weierstrass coordinates of the point, if is_infinite is false.

#### Fields

##### x: Field

##### y: Field

##### is_infinite: bool

#### Implementations

<h5><pre><code>impl <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre></h5>

<pre><code>pub fn double(self: <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a>) -> <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre>

Elliptic curve point doubling operation
returns the doubled point of a point P, i.e P+P

<pre><code>pub fn point_at_infinity() -> <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre>

Returns the null element of the curve; 'the point at infinity'

<pre><code>pub fn generator() -> <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre>

Returns the curve's generator point.

#### Trait implementations

<h5><pre><code>impl Add for <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre></h5>

<h5><pre><code>impl Sub for <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre></h5>

<h5><pre><code>impl Neg for <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre></h5>

<h5><pre><code>impl <a href="#Eq">Eq</a> for <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre></h5>

<a id="EmbeddedCurveScalar"></a>

### Struct `EmbeddedCurveScalar`

<pre><code>pub struct EmbeddedCurveScalar {
    pub lo: Field,
    pub hi: Field,
}
</code></pre>
Scalar for the embedded curve represented as low and high limbs
By definition, the scalar field of the embedded curve is base field of the proving system curve.
It may not fit into a Field element, so it is represented with two Field elements; its low and high limbs.

#### Fields

##### lo: Field

##### hi: Field

#### Implementations

<h5><pre><code>impl <a href="#EmbeddedCurveScalar">EmbeddedCurveScalar</a></code></pre></h5>

<pre><code>pub fn new(lo: Field, hi: Field) -> <a href="#EmbeddedCurveScalar">EmbeddedCurveScalar</a></code></pre>

<pre><code>pub fn from_field(scalar: Field) -> <a href="#EmbeddedCurveScalar">EmbeddedCurveScalar</a></code></pre>

#### Trait implementations

<h5><pre><code>impl <a href="#Eq">Eq</a> for <a href="#EmbeddedCurveScalar">EmbeddedCurveScalar</a></code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for <a href="#EmbeddedCurveScalar">EmbeddedCurveScalar</a></code></pre></h5>

### Function `multi_scalar_mul`

<pre><code>pub fn multi_scalar_mul&lt;let N: u32&gt;(points: [<a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a>; N], scalars: [<a href="#EmbeddedCurveScalar">EmbeddedCurveScalar</a>; N]) -> <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre>

### Function `fixed_base_scalar_mul`

<pre><code>pub fn fixed_base_scalar_mul(scalar: <a href="#EmbeddedCurveScalar">EmbeddedCurveScalar</a>) -> <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre>

### Function `embedded_curve_add`

<pre><code>pub fn embedded_curve_add(point1: <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a>, point2: <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a>) -> <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre>

This function only assumes that the points are on the curve
It handles corner cases around the infinity point causing some overhead compared to embedded_curve_add_not_nul and embedded_curve_add_unsafe

### Function `embedded_curve_add_not_nul`

<pre><code>pub fn embedded_curve_add_not_nul(point1: <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a>, point2: <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a>) -> <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre>

This function assumes that:
The points are on the curve, and
The points don't share an x-coordinate, and
Neither point is the infinity point.
If it is used with correct input, the function ensures the correct non-zero result is returned.
Except for points on the curve, the other assumptions are checked by the function. It will cause assertion failure if they are not respected.

### Function `embedded_curve_add_unsafe`

<pre><code>pub fn embedded_curve_add_unsafe(point1: <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a>, point2: <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a>) -> <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre>

Unsafe ec addition
If the inputs are the same, it will perform a doubling, but only if point1 and point2 are the same variable.
If they have the same value but are different variables, the result will be incorrect because in this case
it assumes (but does not check) that the points' x-coordinates are not equal.
It also assumes neither point is the infinity point.

## Module field

### Function `modulus_num_bits`

<pre><code>pub fn modulus_num_bits() -> u64</code></pre>

### Function `modulus_be_bits`

<pre><code>pub fn modulus_be_bits() -> [u1]</code></pre>

### Function `modulus_le_bits`

<pre><code>pub fn modulus_le_bits() -> [u1]</code></pre>

### Function `modulus_be_bytes`

<pre><code>pub fn modulus_be_bytes() -> [u8]</code></pre>

### Function `modulus_le_bytes`

<pre><code>pub fn modulus_le_bytes() -> [u8]</code></pre>

### Function `bytes32_to_field`

<pre><code>pub fn bytes32_to_field(bytes32: [u8; 32]) -> Field</code></pre>

## Module field::bn254

### Function `decompose`

<pre><code>pub fn decompose(x: Field) -> (Field, Field)</code></pre>

Decompose a single field into two 16 byte fields.

### Function `assert_gt`

<pre><code>pub fn assert_gt(a: Field, b: Field)</code></pre>

### Function `assert_lt`

<pre><code>pub fn assert_lt(a: Field, b: Field)</code></pre>

### Function `gt`

<pre><code>pub fn gt(a: Field, b: Field) -> bool</code></pre>

### Function `lt`

<pre><code>pub fn lt(a: Field, b: Field) -> bool</code></pre>

## Module hash

<a id="BuildHasherDefault"></a>

### Struct `BuildHasherDefault`

<pre><code>pub struct BuildHasherDefault&lt;H&gt; {}
</code></pre>

#### Trait implementations

<h5><pre><code>impl&lt;H&gt; <a href="#BuildHasher">BuildHasher</a> for <a href="#BuildHasherDefault">BuildHasherDefault</a>&lt;H&gt;
where
    H: <a href="#Hasher">Hasher</a>,
    H: <a href="#Default">Default</a>
</code></pre></h5>

<h5><pre><code>impl&lt;H&gt; <a href="#Default">Default</a> for <a href="#BuildHasherDefault">BuildHasherDefault</a>&lt;H&gt;
where
    H: <a href="#Hasher">Hasher</a>,
    H: <a href="#Default">Default</a>
</code></pre></h5>

<a id="Hash"></a>

### Trait `Hash`

<pre><code>pub trait Hash {
}</code></pre>

#### Methods

<pre><code>pub fn hash&lt;H&gt;(self: Self, state: &mut H)
where
    H: <a href="#Hasher">Hasher</a>
</code></pre>

#### Implementors

<h5><pre><code>impl <a href="#Hash">Hash</a> for <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for <a href="#EmbeddedCurveScalar">EmbeddedCurveScalar</a></code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for Field</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for u1</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for u8</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for u16</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for u32</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for u64</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for u128</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for i8</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for i16</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for i32</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for i64</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for bool</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for ()</code></pre></h5>

<h5><pre><code>impl&lt;let N: u32, T&gt; <a href="#Hash">Hash</a> for [T; N]
where
    T: <a href="#Hash">Hash</a>
</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#Hash">Hash</a> for [T]
where
    T: <a href="#Hash">Hash</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B&gt; <a href="#Hash">Hash</a> for (A, B)
where
    A: <a href="#Hash">Hash</a>,
    B: <a href="#Hash">Hash</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C&gt; <a href="#Hash">Hash</a> for (A, B, C)
where
    A: <a href="#Hash">Hash</a>,
    B: <a href="#Hash">Hash</a>,
    C: <a href="#Hash">Hash</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C, D&gt; <a href="#Hash">Hash</a> for (A, B, C, D)
where
    A: <a href="#Hash">Hash</a>,
    B: <a href="#Hash">Hash</a>,
    C: <a href="#Hash">Hash</a>,
    D: <a href="#Hash">Hash</a>
</code></pre></h5>

<h5><pre><code>impl&lt;A, B, C, D, E&gt; <a href="#Hash">Hash</a> for (A, B, C, D, E)
where
    A: <a href="#Hash">Hash</a>,
    B: <a href="#Hash">Hash</a>,
    C: <a href="#Hash">Hash</a>,
    D: <a href="#Hash">Hash</a>,
    E: <a href="#Hash">Hash</a>
</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for CtString</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for FunctionDefinition</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for Module</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for <a href="#UnaryOp">UnaryOp</a></code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for <a href="#BinaryOp">BinaryOp</a></code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for Quoted</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for TraitConstraint</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for TraitDefinition</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for Type</code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for TypeDefinition</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#Hash">Hash</a> for <a href="#Option">Option</a>&lt;T&gt;
where
    T: <a href="#Hash">Hash</a>
</code></pre></h5>

<a id="Hasher"></a>

### Trait `Hasher`

<pre><code>pub trait Hasher {
}</code></pre>

#### Methods

<pre><code>pub fn finish(self: Self) -> Field</code></pre>

<pre><code>pub fn write(&mut self: &mut Self, input: Field)</code></pre>

#### Implementors

<h5><pre><code>impl <a href="#Hasher">Hasher</a> for Poseidon2Hasher</code></pre></h5>

<a id="BuildHasher"></a>

### Trait `BuildHasher`

<pre><code>pub trait BuildHasher {
}</code></pre>

#### Methods

<pre><code>pub fn build_hasher(self: Self) -> H</code></pre>

#### Implementors

<h5><pre><code>impl&lt;H&gt; <a href="#BuildHasher">BuildHasher</a> for <a href="#BuildHasherDefault">BuildHasherDefault</a>&lt;H&gt;
where
    H: <a href="#Hasher">Hasher</a>,
    H: <a href="#Default">Default</a>
</code></pre></h5>

### Function `sha256_compression`

<pre><code>pub fn sha256_compression(input: [u32; 16], state: [u32; 8]) -> [u32; 8]</code></pre>

### Function `keccakf1600`

<pre><code>pub fn keccakf1600(input: [u64; 25]) -> [u64; 25]</code></pre>

### Function `blake2s`

<pre><code>pub fn blake2s&lt;let N: u32&gt;(input: [u8; N]) -> [u8; 32]</code></pre>

### Function `blake3`

<pre><code>pub fn blake3&lt;let N: u32&gt;(input: [u8; N]) -> [u8; 32]</code></pre>

### Function `pedersen_commitment`

<pre><code>pub fn pedersen_commitment&lt;let N: u32&gt;(input: [Field; N]) -> <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre>

### Function `pedersen_commitment_with_separator`

<pre><code>pub fn pedersen_commitment_with_separator&lt;let N: u32&gt;(input: [Field; N], separator: u32) -> <a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a></code></pre>

### Function `pedersen_hash`

<pre><code>pub fn pedersen_hash&lt;let N: u32&gt;(input: [Field; N]) -> Field</code></pre>

### Function `pedersen_hash_with_separator`

<pre><code>pub fn pedersen_hash_with_separator&lt;let N: u32&gt;(input: [Field; N], separator: u32) -> Field</code></pre>

### Function `derive_generators`

<pre><code>pub fn derive_generators&lt;let N: u32, let M: u32&gt;(domain_separator_bytes: [u8; M], starting_index: u32) -> [<a href="#EmbeddedCurvePoint">EmbeddedCurvePoint</a>; N]</code></pre>

### Function `poseidon2_permutation`

<pre><code>pub fn poseidon2_permutation&lt;let N: u32&gt;(input: [Field; N], state_len: u32) -> [Field; N]</code></pre>

## Module hash::keccak

### Function `keccakf1600`

<pre><code>pub fn keccakf1600(input: [u64; 25]) -> [u64; 25]</code></pre>

## Module hint

### Function `black_box`

<pre><code>pub fn black_box&lt;T&gt;(value: T) -> T</code></pre>

An identity function that *hints* to the compiler to be maximally pessimistic about what `black_box` could do.

This can be used to block the SSA optimization passes being applied to a value, which should help to prevent
test programs from being optimized down to nothing and have them resemble runtime code more closely.

## Module mem

### Function `zeroed`

<pre><code>pub fn zeroed&lt;T&gt;() -> T</code></pre>

For any type, return an instance of that type by initializing
all of its fields to 0. This is considered to be unsafe since there
is no guarantee that all zeroes is a valid bit pattern for every type.

### Function `checked_transmute`

<pre><code>pub fn checked_transmute&lt;T, U&gt;(value: T) -> U</code></pre>

Transmutes a value of type T to a value of type U.

Both types are asserted to be equal during compilation but after type checking.
If not, a compilation error is issued.

This function is useful for types using arithmetic generics in cases
which the compiler otherwise cannot prove equal during type checking.
You can use this to obtain a value of the correct type while still asserting
that it is equal to the previous.

### Function `array_refcount`

<pre><code>pub fn array_refcount&lt;T, let N: u32&gt;(array: [T; N]) -> u32</code></pre>

Returns the internal reference count of an array value in unconstrained code.

Arrays only have reference count in unconstrained code - using this anywhere
else will return zero.

### Function `slice_refcount`

<pre><code>pub fn slice_refcount&lt;T&gt;(slice: [T]) -> u32</code></pre>

Returns the internal reference count of a slice value in unconstrained code.

Slices only have reference count in unconstrained code - using this anywhere
else will return zero.

## Module meta

<a id="DeriveFunction"></a>

### Type alias `DeriveFunction`

<pre><code>pub type DeriveFunction = fn(TypeDefinition) -> Quoted;</code></pre>

### Function `unquote`

<pre><code>pub fn unquote(code: Quoted) -> Quoted</code></pre>

Calling unquote as a macro (via `unquote!(arg)`) will unquote
its argument. Since this is the effect `!` already does, `unquote`
itself does not need to do anything besides return its argument.

### Function `type_of`

<pre><code>pub fn type_of&lt;T&gt;(x: T) -> Type</code></pre>

Returns the type of any value

### Function `derive`

<pre><code>pub fn derive(s: TypeDefinition, traits: [TraitDefinition]) -> Quoted</code></pre>

### Function `derive_via`

<pre><code>pub fn derive_via(t: TraitDefinition, f: <a href="#DeriveFunction">DeriveFunction</a>)</code></pre>

### Function `make_trait_impl`

<pre><code>pub fn make_trait_impl&lt;Env1, Env2&gt;(
    s: TypeDefinition,
    trait_name: Quoted,
    function_signature: Quoted,
    for_each_field: fn[Env1](Quoted) -> Quoted,
    join_fields_with: Quoted,
    body: fn[Env2](Quoted) -> Quoted,
) -> Quoted</code></pre>

`make_impl` is a helper function to make a simple impl, usually while deriving a trait.
This impl has a couple assumptions:
1. The impl only has one function, with the signature `function_signature`
2. The trait itself does not have any generics.

While these assumptions are met, `make_impl` will create an impl from a TypeDefinition,
automatically filling in the required generics from the type, along with the where clause.
The function body is created by mapping each field with `for_each_field` and joining the
results with `join_fields_with`. The result of this is passed to the `body` function for
any final processing - e.g. wrapping each field in a `StructConstructor { .. }` expression.

See `derive_eq` and `derive_default` for example usage.

## Module meta::ctstring

<a id="AsCtString"></a>

### Trait `AsCtString`

<pre><code>pub trait AsCtString {
}</code></pre>

#### Methods

<pre><code>pub fn as_ctstring(self: Self) -> CtString</code></pre>

#### Implementors

<h5><pre><code>impl&lt;let N: u32&gt; <a href="#AsCtString">AsCtString</a> for str<N></code></pre></h5>

<h5><pre><code>impl&lt;let N: u32, T&gt; <a href="#AsCtString">AsCtString</a> for fmtstr<N, T></code></pre></h5>

## Module meta::expr

Contains methods on the built-in `Expr` type for quoted, syntactically valid expressions.

## Module meta::op

<a id="UnaryOp"></a>

### Struct `UnaryOp`

<pre><code>pub struct UnaryOp
{ /* private fields */ }
</code></pre>

#### Implementations

<h5><pre><code>impl <a href="#UnaryOp">UnaryOp</a></code></pre></h5>

<pre><code>pub fn is_minus(self: <a href="#UnaryOp">UnaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_not(self: <a href="#UnaryOp">UnaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_mutable_reference(self: <a href="#UnaryOp">UnaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_dereference(self: <a href="#UnaryOp">UnaryOp</a>) -> bool</code></pre>

<pre><code>pub fn quoted(self: <a href="#UnaryOp">UnaryOp</a>) -> Quoted</code></pre>

#### Trait implementations

<h5><pre><code>impl <a href="#Eq">Eq</a> for <a href="#UnaryOp">UnaryOp</a></code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for <a href="#UnaryOp">UnaryOp</a></code></pre></h5>

<a id="BinaryOp"></a>

### Struct `BinaryOp`

<pre><code>pub struct BinaryOp
{ /* private fields */ }
</code></pre>

#### Implementations

<h5><pre><code>impl <a href="#BinaryOp">BinaryOp</a></code></pre></h5>

<pre><code>pub fn is_add(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_subtract(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_multiply(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_divide(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_equal(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_not_equal(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_less_than(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_less_than_or_equal(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_greater_than(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_greater_than_or_equal(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_and(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_or(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_xor(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_shift_right(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_shift_left(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn is_modulo(self: <a href="#BinaryOp">BinaryOp</a>) -> bool</code></pre>

<pre><code>pub fn quoted(self: <a href="#BinaryOp">BinaryOp</a>) -> Quoted</code></pre>

#### Trait implementations

<h5><pre><code>impl <a href="#Eq">Eq</a> for <a href="#BinaryOp">BinaryOp</a></code></pre></h5>

<h5><pre><code>impl <a href="#Hash">Hash</a> for <a href="#BinaryOp">BinaryOp</a></code></pre></h5>

## Module meta::typ

Contains methods on the built-in `Type` type used for representing a type in the source program.

### Function `fresh_type_variable`

<pre><code>pub fn fresh_type_variable() -> Type</code></pre>

Creates and returns an unbound type variable. This is a special kind of type internal
to type checking which will type check with any other type. When it is type checked
against another type it will also be set to that type. For example, if `a` is a type
variable and we have the type equality `(a, i32) = (u8, i32)`, the compiler will set
`a` equal to `u8`.

Unbound type variables will often be rendered as `_` while printing them. Bound type
variables will appear as the type they are bound to.

This can be used in conjunction with functions which internally perform type checks
such as `Type::implements` or `Type::get_trait_impl` to potentially grab some of the types used.

Note that calling `Type::implements` or `Type::get_trait_impl` on a type variable will always
fail.

Example:

```noir
trait Serialize<let N: u32> {}

impl Serialize<1> for Field {}

impl<T, let N: u32, let M: u32> Serialize<N * M> for [T; N]
    where T: Serialize<M> {}

impl<T, U, let N: u32, let M: u32> Serialize<N + M> for (T, U)
    where T: Serialize<N>, U: Serialize<M> {}

fn fresh_variable_example() {
    let typevar1 = std::meta::typ::fresh_type_variable();
    let constraint = quote { Serialize<$typevar1> }.as_trait_constraint();
    let field_type = quote { Field }.as_type();

    // Search for a trait impl (binding typevar1 to 1 when the impl is found):
    assert(field_type.implements(constraint));

    // typevar1 should be bound to the "1" generic now:
    assert_eq(typevar1.as_constant().unwrap(), 1);

    // If we want to do the same with a different type, we need to
    // create a new type variable now that `typevar1` is bound
    let typevar2 = std::meta::typ::fresh_type_variable();
    let constraint = quote { Serialize<$typevar2> }.as_trait_constraint();
    let array_type = quote { [(Field, Field); 5] }.as_type();
    assert(array_type.implements(constraint));

    // Now typevar2 should be bound to the serialized pair size 2 times the array length 5
    assert_eq(typevar2.as_constant().unwrap(), 10);
}
```

## Module meta::typed_expr

Contains methods on the built-in `TypedExpr` type for resolved and type-checked expressions.

## Module meta::unresolved_type

Contains methods on the built-in `UnresolvedType` type for the syntax of types.

## Module option

<a id="Option"></a>

### Struct `Option`

<pre><code>pub struct Option&lt;T&gt;
{ /* private fields */ }
</code></pre>

#### Implementations

<h5><pre><code>impl&lt;T&gt; <a href="#Option">Option</a>&lt;T&gt;</code></pre></h5>

<pre><code>pub fn none() -> <a href="#Option">Option</a>&lt;T&gt;</code></pre>

Constructs a None value

<pre><code>pub fn some(_value: T) -> <a href="#Option">Option</a>&lt;T&gt;</code></pre>

Constructs a Some wrapper around the given value

<pre><code>pub fn is_none(self: <a href="#Option">Option</a>&lt;T&gt;) -> bool</code></pre>

True if this Option is None

<pre><code>pub fn is_some(self: <a href="#Option">Option</a>&lt;T&gt;) -> bool</code></pre>

True if this Option is Some

<pre><code>pub fn unwrap(self: <a href="#Option">Option</a>&lt;T&gt;) -> T</code></pre>

Asserts `self.is_some()` and returns the wrapped value.

<pre><code>pub fn unwrap_unchecked(self: <a href="#Option">Option</a>&lt;T&gt;) -> T</code></pre>

Returns the inner value without asserting `self.is_some()`
Note that if `self` is `None`, there is no guarantee what value will be returned,
only that it will be of type `T`.

<pre><code>pub fn unwrap_or(self: <a href="#Option">Option</a>&lt;T&gt;, default: T) -> T</code></pre>

Returns the wrapped value if `self.is_some()`. Otherwise, returns the given default value.

<pre><code>pub fn unwrap_or_else&lt;Env&gt;(self: <a href="#Option">Option</a>&lt;T&gt;, default: fn[Env]() -> T) -> T</code></pre>

Returns the wrapped value if `self.is_some()`. Otherwise, calls the given function to return
a default value.

<pre><code>pub fn expect&lt;let N: u32, MessageTypes&gt;(self: <a href="#Option">Option</a>&lt;T&gt;, message: fmtstr<N, MessageTypes>) -> T</code></pre>

Asserts `self.is_some()` with a provided custom message and returns the contained `Some` value

<pre><code>pub fn map&lt;U, Env&gt;(self: <a href="#Option">Option</a>&lt;T&gt;, f: fn[Env](T) -> U) -> <a href="#Option">Option</a>&lt;U&gt;</code></pre>

If self is `Some(x)`, this returns `Some(f(x))`. Otherwise, this returns `None`.

<pre><code>pub fn map_or&lt;U, Env&gt;(
    self: <a href="#Option">Option</a>&lt;T&gt;,
    default: U,
    f: fn[Env](T) -> U,
) -> U</code></pre>

If self is `Some(x)`, this returns `f(x)`. Otherwise, this returns the given default value.

<pre><code>pub fn map_or_else&lt;U, Env1, Env2&gt;(
    self: <a href="#Option">Option</a>&lt;T&gt;,
    default: fn[Env1]() -> U,
    f: fn[Env2](T) -> U,
) -> U</code></pre>

If self is `Some(x)`, this returns `f(x)`. Otherwise, this returns `default()`.

<pre><code>pub fn and(self: <a href="#Option">Option</a>&lt;T&gt;, other: <a href="#Option">Option</a>&lt;T&gt;) -> <a href="#Option">Option</a>&lt;T&gt;</code></pre>

Returns None if self is None. Otherwise, this returns `other`.

<pre><code>pub fn and_then&lt;U, Env&gt;(self: <a href="#Option">Option</a>&lt;T&gt;, f: fn[Env](T) -> <a href="#Option">Option</a>&lt;U&gt;) -> <a href="#Option">Option</a>&lt;U&gt;</code></pre>

If self is None, this returns None. Otherwise, this calls the given function
with the Some value contained within self, and returns the result of that call.

In some languages this function is called `flat_map` or `bind`.

<pre><code>pub fn or(self: <a href="#Option">Option</a>&lt;T&gt;, other: <a href="#Option">Option</a>&lt;T&gt;) -> <a href="#Option">Option</a>&lt;T&gt;</code></pre>

If self is Some, return self. Otherwise, return `other`.

<pre><code>pub fn or_else&lt;Env&gt;(self: <a href="#Option">Option</a>&lt;T&gt;, default: fn[Env]() -> <a href="#Option">Option</a>&lt;T&gt;) -> <a href="#Option">Option</a>&lt;T&gt;</code></pre>

If self is Some, return self. Otherwise, return `default()`.

<pre><code>pub fn xor(self: <a href="#Option">Option</a>&lt;T&gt;, other: <a href="#Option">Option</a>&lt;T&gt;) -> <a href="#Option">Option</a>&lt;T&gt;</code></pre>

<pre><code>pub fn filter&lt;Env&gt;(self: <a href="#Option">Option</a>&lt;T&gt;, predicate: fn[Env](T) -> bool) -> <a href="#Option">Option</a>&lt;T&gt;</code></pre>

Returns `Some(x)` if self is `Some(x)` and `predicate(x)` is true.
Otherwise, this returns `None`

<pre><code>pub fn flatten(option: <a href="#Option">Option</a>&lt;<a href="#Option">Option</a>&lt;T&gt;&gt;) -> <a href="#Option">Option</a>&lt;T&gt;</code></pre>

Flattens an Option<Option<T>> into a Option<T>.
This returns None if the outer Option is None. Otherwise, this returns the inner Option.

#### Trait implementations

<h5><pre><code>impl&lt;T&gt; <a href="#Default">Default</a> for <a href="#Option">Option</a>&lt;T&gt;</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#Eq">Eq</a> for <a href="#Option">Option</a>&lt;T&gt;
where
    T: <a href="#Eq">Eq</a>
</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#Hash">Hash</a> for <a href="#Option">Option</a>&lt;T&gt;
where
    T: <a href="#Hash">Hash</a>
</code></pre></h5>

<h5><pre><code>impl&lt;T&gt; <a href="#Ord">Ord</a> for <a href="#Option">Option</a>&lt;T&gt;
where
    T: <a href="#Ord">Ord</a>
</code></pre></h5>

## Module panic

### Function `panic`

<pre><code>pub fn panic&lt;T, U, let N: u32&gt;(message: fmtstr<N, T>) -> U</code></pre>

## Module runtime

### Function `is_unconstrained`

<pre><code>pub fn is_unconstrained() -> bool</code></pre>

## Module test

<a id="OracleMock"></a>

### Struct `OracleMock`

<pre><code>pub struct OracleMock
{ /* private fields */ }
</code></pre>

#### Implementations

<h5><pre><code>impl <a href="#OracleMock">OracleMock</a></code></pre></h5>

<pre><code>pub unconstrained fn mock&lt;let N: u32&gt;(name: str<N>) -> <a href="#OracleMock">OracleMock</a></code></pre>

<pre><code>pub unconstrained fn with_params&lt;P&gt;(self: <a href="#OracleMock">OracleMock</a>, params: P) -> <a href="#OracleMock">OracleMock</a></code></pre>

<pre><code>pub unconstrained fn get_last_params&lt;P&gt;(self: <a href="#OracleMock">OracleMock</a>) -> P</code></pre>

<pre><code>pub unconstrained fn returns&lt;R&gt;(self: <a href="#OracleMock">OracleMock</a>, returns: R) -> <a href="#OracleMock">OracleMock</a></code></pre>

<pre><code>pub unconstrained fn times(self: <a href="#OracleMock">OracleMock</a>, times: u64) -> <a href="#OracleMock">OracleMock</a></code></pre>

<pre><code>pub unconstrained fn clear(self: <a href="#OracleMock">OracleMock</a>)</code></pre>

<pre><code>pub unconstrained fn times_called(self: <a href="#OracleMock">OracleMock</a>) -> Field</code></pre>


