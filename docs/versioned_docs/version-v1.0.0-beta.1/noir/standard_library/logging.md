---
title: Logging
description:
  Learn how to use the println statement for debugging in Noir with this tutorial. Understand the
  basics of logging in Noir and how to implement it in your code.
keywords:
  [
    noir logging,
    println statement,
    print statement,
    debugging in noir,
    noir std library,
    logging tutorial,
    basic logging in noir,
    noir logging implementation,
    noir debugging techniques,
    rust,
  ]
---

The standard library provides two familiar statements you can use: `println` and `print`. Despite being a limited implementation of rust's `println!` and `print!` macros, these constructs can be useful for debugging.

You can print the output of both statements in your Noir code by using the `nargo execute` command or the `--show-output` flag when using `nargo test` (provided there are print statements in your tests).

It is recommended to use `nargo execute` if you want to debug failing constraints with `println` or `print` statements. This is due to every input in a test being a constant rather than a witness, so we issue an error during compilation while we only print during execution (which comes after compilation). Neither `println`, nor `print` are callable for failed constraints caught at compile time.

Both `print` and `println` are generic functions which can work on integers, fields, strings, and even structs or expressions. Note however, that slices are currently unsupported. For example:

```rust
struct Person {
    age: Field,
    height: Field,
}

fn main(age: Field, height: Field) {
    let person = Person {
        age: age,
        height: height,
    };
    println(person);
    println(age + height);
    println("Hello world!");
}
```

You can print different types in the same statement (including strings) with a type called `fmtstr`. It can be specified in the same way as a normal string, just prepended with an "f" character:

```rust
  let fmt_str = f"i: {i}, j: {j}";
  println(fmt_str);

  let s = myStruct { y: x, x: y };
  println(s);

  println(f"i: {i}, s: {s}");

  println(x);
  println([x, y]);

  let foo = fooStruct { my_struct: s, foo: 15 };
  println(f"s: {s}, foo: {foo}");

  println(15);       // prints 0x0f, implicit Field
  println(-1 as u8); // prints 255
  println(-1 as i8); // prints -1
```

Examples shown above are interchangeable between the two `print` statements:

```rust
let person = Person { age : age, height : height };

println(person);
print(person);

println("Hello world!"); // Prints with a newline at the end of the input
print("Hello world!");   // Prints the input and keeps cursor on the same line
```
