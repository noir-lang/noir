# Concepts [OLD BY MIKE, TO SPLIT]

<!-- Note to help authors: there's this big hackmd, which is out of date, but contains lots of detail. It might help with a first draft: https://hackmd.io/3aoMU-2GT3mCl4Vl4nTBkg?both -->


Here, we outline all of the Noir Contract language concepts and keywords.

## `contract`

```rust
contract MyContract {

    constructor() {}
    
    fn my_function_1(x: Field, y: Field) -> Field {
        x + y
    }
    
    fn my_function_2(x: Field, y: Field) -> Field {
        x * y
    }
}
```

- A dev can declare a `contract`; a scope which encapsulates a collection of functions and state variables.
- Functions within this scope are said to belong to that contract.
- No external functions (i.e. functions which cannot be inlined) are allowed outside a contract scope. All external functions belong to a contract.
- Contracts are named using PascalCase.
- There is no `main()` function within a contract scope (as opposed to a 'regular noir' program). This is because more than one function may be called and proven (as opposed to inlined by the compiler). That is to say, a developer might want to be able to generate a proof for more than one function in a contract's scope.

## Noir Contract stdlib

On top of 'regular Noir's' stdlib, we provide a stdlib for writing Noir Contracts. The Noir Contract stdlib contains structs and abstractions which remove the need to understand the low-level Aztec protocol.

### State Variables

#### `PublicState<T>`

#### `Map<T>`

#### Private State

##### UTXO trees

##### Notes

##### Custom Notes

##### `UTXO<NoteType>`

##### `UTXOSet<NoteType>`



## Functions

### Public vs Private

Naming things is hard.

The words 'public' and 'private' are perfect for describing Aztec's features -- the ability to hide and/or reveal state variables and function execution.

But those words are also overloaded:
- In smart contract languages, `public` and `private` can be used to describe how a function may be called.
- In many other languages, `public` and `private` describe the accessibility of class methods and members.
- In 'regular Noir', `pub` is used to declare that a parameter or return variable is a 'public input' to the circuit. 

So, whilst 'public' and 'private' are used liberally when describing features of the Aztec network (and indeed in the Aztec codebase), we've avoided using those words anywhere in Noir Contract syntax.

Instead, we seem to use `open` instead of `public` (why not!) and `secret` for private stuff. Maybe we should use `open` and `closed`. Seems better.

### `constructor`

- A special `constructor` function MUST be declared within a contract's scope.
- A constructor doesn't have a name, because its purpose is clear: to initialise state.
- In Aztec terminology, a constructor is always a 'private function' (i.e. it cannot be an `open` function).
- A constructor behaves almost identically to any other function. It's just important for Aztec to be able to identify this function as special: it may only be called once, and will not be deployed as part of the contract.

### secret functions

### `open` functions

### `unconstrained` functions



## Calling functions

### Inlining

### Importing Contracts

### Constrained --> Unconstrained

E.g. `get()`

### Oracle calls

### Private --> Private

### Public --> Public

### Private --> Public

### `internal` keyword

### Public --> Private

### Recursive function calls

### L1 --> L2

### L2 --> L1

### Delegatecall

Talk a about the dangers of delegatecall too!



## Events

### Constraining events

### Unencrypted Events

### Encrypted Events

### Costs

Explain L1 cost to emit an event.



## Access Control

### msg_sender

### slow updates tree (TBC!!!)







## Limitations

### Num reads and writes

### Num function calls

### Num logs

### Num key pair validations

### No gas or fees yet


## Future Improvements

See the Noir Dogfooding retro doc.