# Functions

## `constructor`

- A special `constructor` function MUST be declared within a contract's scope.
- A constructor doesn't have a name, because its purpose is clear: to initialise state.
- In Aztec terminology, a constructor is always a 'private function' (i.e. it cannot be an `open` function).
- A constructor behaves almost identically to any other function. It's just important for Aztec to be able to identify this function as special: it may only be called once, and will not be deployed as part of the contract.

## secret functions

## `open` functions

## `unconstrained` functions



# Calling functions

## Inlining

## Importing Contracts

### Contract Interface

## Constrained --> Unconstrained

E.g. `get()`

## Oracle calls

## Private --> Private

## Public --> Public

## Private --> Public

## `internal` keyword

## Public --> Private

## Recursive function calls

## L1 --> L2

## L2 --> L1

## Delegatecall

Talk a about the dangers of delegatecall too!