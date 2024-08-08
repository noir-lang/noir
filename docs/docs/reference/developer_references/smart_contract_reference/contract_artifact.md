---
title: "Contract Artifact Reference"
tags: [contracts]
---

After compiling a contract you'll get a Contract Artifact file, that contains the data needed to interact with a specific contract, including its name, functions that can be executed, and the interface and code of those functions. Since private functions are not published in the Aztec network, you'll need this artifact file to be able to call private functions of contracts.

The artifact file can be used with `aztec.js` to instantiate contract objects and interact with them.

## Contract Artifact Structure

The structure of a contract artifact is as follows:
```json
{
  "name": "CardGame",
  "functions": [
    {
      "name": "constructor",
      "functionType": "private",
      "isInternal": false,
      "parameters": [],
      "returnTypes": [],
      "bytecode": "...",
      "verificationKey": "..."
    },
    {
      "name": "on_card_played",
      "functionType": "public",
      "isInternal": true,
      "parameters": [
        {
          "name": "game",
          "type": {
            "kind": "integer",
            "sign": "unsigned",
            "width": 32
          },
          "visibility": "private"
        },
        {
          "name": "player",
          "type": {
            "kind": "field"
          },
          "visibility": "private"
        },
        {
          "name": "card_as_field",
          "type": {
            "kind": "field"
          },
          "visibility": "private"
        }
      ],
      "returnTypes": [
        ...
      ],
      "bytecode": "...",
      "verificationKey": "..."
    },
   ...
  ]
}

```

### `name`
It is a simple string that matches the name that the contract developer used for this contract in noir. It's used for logs and errors.

### `functions`
A contract is a collection of several functions that can be called. Each function has the following properties:

#### `function.name`
A simple string that matches the name that the contract developer used for this function in noir. For logging and debugging purposes.

#### `function.functionType`
The function type can have one of the following values:

- Private: The function is ran and proved locally by the clients, and its bytecode not published to the network.
- Public: The function is ran and proved by the sequencer, and its bytecode is published to the network.
- Unconstrained: The function is ran locally by the clients to generate digested information useful for the user. It's not meant to be transacted against.

#### `function.isInternal`
The is internal property is a boolean that indicates whether the function is internal to the contract and cannot be called from outside.

#### `function.parameters`
Each function can have multiple parameters that are arguments to execute the function. Parameters have a name, and type (like integers, strings, or complex types like arrays and structures).

#### `function.returnTypes`
The return types property defines the types of values that the function returns after execution.

#### `function.bytecode`
The bytecode is a string representing the compiled ACIR of the function, ready for execution on the network.

#### `function.verificationKey`
The verification key is an optional property that contains the verification key of the function. This key is used to verify the proof of the function execution.

### `debug` (Optional)
Although not significant for non-developer users, it is worth mentioning that there is a debug section in the contract artifact which helps contract developers to debug and test their contracts. This section mainly contains debug symbols and file maps that link back to the original source code.

## Understanding Parameter and Return Types
To make the most of the functions, it's essential to understand the types of parameters and return values. Here are some common types you might encounter:

 - `field`: A basic type representing a field element in the finite field of the curve used in the Aztec protocol.
 - `boolean`: A simple true/false value.
 - `integer`: Represents whole numbers. It has attributes defining its sign (positive or negative) and width (the number of bits representing the integer).
 - `array`: Represents a collection of elements, all of the same type. It has attributes defining its length and the type of elements it holds.
 - `string`: Represents a sequence of characters with a specified length.
 - `struct`: A complex type representing a structure with various fields, each having a specific type and name.

