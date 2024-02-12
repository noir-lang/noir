---
title: Slow Updates Tree
---

## Struct `SlowMap`

### Overview

The `SlowMap` struct is used to interact with a slow updates tree deployed via the SlowTree smart contract.

### Fields

| Name    | Type    | Description                          |
| ------- | ------- | ------------------------------------ |
| address | `Field` | The address of the SlowTree contract |

## Functions

### at

Returns an instance of `SlowMap` at the specified address.

**Parameters**

| Name      | Type           | Description                 |
| --------- | -------------- | --------------------------- |
| `address` | `AztecAddress` | The address of the SlowTree |

**Return**

| Name | Type      | Description            |
| ---- | --------- | ---------------------- |
| -    | `SlowMap` | The `SlowMap` instance |

**Example**

#include_code slowmap_at noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

### initialize

Initializes the `SlowMap`.

**Parameters**

| Name      | Type            | Description           |
| --------- | --------------- | --------------------- |
| `context` | `PublicContext` | The execution context |

**Return**

| Name | Type | Description |
| ---- | ---- | ----------- |
| -    | -    | -           |

**Example**

#include_code slowmap_initialize noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

### read_at_pub

Reads a value at a specified index from a public function.

**Parameters**

| Name      | Type            | Description           |
| --------- | --------------- | --------------------- |
| `context` | `PublicContext` | The execution context |
| `index`   | `Field`         | The index to read at  |

**Return**

| Name     | Type    | Description          |
| -------- | ------- | -------------------- |
| `result` | `Field` | The value at `index` |

**Example**

#include_code read_at_pub noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

### read_at

Reads a value at a specified index from a private function.

**Parameters**

| Name      | Type             | Description           |
| --------- | ---------------- | --------------------- |
| `context` | `PrivateContext` | The execution context |
| `index`   | `Field`          | The index to read at  |

**Return**

| Name     | Type    | Description          |
| -------- | ------- | -------------------- |
| `result` | `Field` | The value at `index` |

**Example**

#include_code slowmap_read_at noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

### update_at_private

Updates a value at a specified index from a private function. Does not return anything.

**Parameters**

| Name        | Type             | Description           |
| ----------- | ---------------- | --------------------- |
| `context`   | `PrivateContext` | The execution context |
| `index`     | `Field`          | The index to update   |
| `new_value` | `Field`          | The new value         |

**Example**

#include_code get_and_update_private noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust

## Updating from public

This is not a method in the interface as it can be done using regular Aztec.nr public storage update syntax.

**Example**

#include_code write_slow_update_public noir-projects/noir-contracts/contracts/token_blacklist_contract/src/main.nr rust
