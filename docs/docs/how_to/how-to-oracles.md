---
title: How to use oracles
description: 
keywords:
  [
  ]
sidebar_position: 1
---

This guide shows you how to use oracles in your Noir program. It is interface-agnostic, as we will give examples in both Nargo and NoirJS.

For the sake of clarity, it is assumed that:

- You have read the [explainer on Oracles](../explainers/explainer-oracle.md) and are comfortable with the concept
- You already have your own NoirJS and/or Nargo app. You can use the [Vite Hardhat](https://github.com/noir-lang/noir-starter/tree/main/vite-hardhat) as a boilerplate (for example, by using its [devcontainer in codespaces](../how_to/using-devcontainers.md))
- You understand the concept of a JSON-RPC server. Visit the [JSON-RPC website](https://www.jsonrpc.org/) if you need a refresher.
- You are comfortable with server-side javascript. For the sake of brevity, will skip any details on installing Node, packages, etc.

## Step 1 - Modify your circuit

An oracle is defined in a circuit by two things:

- An unconstrained method - This tells the Noir compiler that it is executing a Brillig block so it won't constrain it
- A decorated oracle method - This tells the compiler that this method is an RPC call.

An example of an oracle that returns a `Field` would be:

```rust
#[oracle(getSquared)]
unconstrained fn squared() -> Field { }

unconstrained fn get_squared() -> Field {
    squared()
}
```

In this example, we're wrapping our oracle function in a unconstrained method, and decorating it with `oracle(getSquared)`. `getSquared` is the actual RPC method being called by Brillig.

You can then call the unconstrained function as you would call any other function:

```rust
fn main(input: Field) {
    let squared = get_squared(input);
}
```

:::danger

As explained in the [Oracle Explainer](../explainers/explainer-oracle.md), this `main` function is unsafe unless you constrain its return value. For example:

```rust
fn main(input: Field, target: Field) {
    let squared = get_squared(input);
    assert(squared as u64 == target as u64); // <---- constrain the return of an oracle!
}
```

:::

You can work with arrays, and pass parameters as long as they're of fixed size. Currently, you can only work with single params or array params.

```rust
#[oracle(getSquared)]
unconstrained fn squared([Field; 2]) -> [Field; 2] { }
```

## Step 2 - Write an RPC server

Brillig will call *one* RPC server. Most likely you will have to write your own, and you can do it in whatever language you prefer. In this guide, we will do it in Javascript.

Let's use the above example of an oracle that consumes an array with two Fields and squares them:

```rust
#[oracle(getSquared)]
unconstrained fn squared(input: [Field; 2]) -> [Field; 2] { }

unconstrained fn get_squared(input: [Field; 2]) -> [Field; 2] {
    squared(input)
}

fn main(input: [Field; 2], target: [Field; 2]) {
    let squared = get_squared(input);
    assert(squared == target);
}
```

And write the correspondent RPC server, starting with the [default JSON-RPC 2.0 boilerplate](https://www.npmjs.com/package/json-rpc-2.0#example):

```js
import { JSONRPCServer } from "json-rpc-2.0";
import express from "express";
import bodyParser from "body-parser";

const app = express();
app.use(bodyParser.json());

const server = new JSONRPCServer();
app.post("/", (req, res) => {
 const jsonRPCRequest = req.body;
 server.receive(jsonRPCRequest).then((jsonRPCResponse) => {
  if (jsonRPCResponse) {
   console.log(jsonRPCResponse);
   res.json(jsonRPCResponse);
  } else {
   res.sendStatus(204);
  }
 });
});

app.listen(5555);
```

Now, add our `getSquared` method, as expected by the `#[oracle(getSquared)]` decorator in our Noir code. It simply maps through the params array and squares the values:

```js
server.addMethod("getSquared", async (params) => {
    const values = params[0].Array.map((x) => {
        return { inner: `${x.inner * x.inner}` };
    });
    return { values: [{ Array: values }] };
});
```

While the syntax can be improved here, this is due to how the oracle expects types. In short, it expects an object with an array of values. Each value is an object declaring to be `Single` or `Array` and returning a `inner` property *as a string*. For example:

```json
{ "values": [{ "Array": [{ "inner": "1" }, { "inner": "2"}]}]}
{ "values": [{ "Single": { "inner": "1" }}]}
{ "values": [{ "Single": { "inner": "1" }}, { "Array": [{ "inner": "1", { "inner": "2" }}]}]}
```

If you're using Typescript, the following types may be helpful in understanding the expected return value and making sure it is easy to follow:

```js
interface Value {
  inner: string,
}

interface SingleForeignCallParam {
  Single: Value,
}

interface ArrayForeignCallParam {
  Array: Value[],
}

type ForeignCallParam = SingleForeignCallParam | ArrayForeignCallParam;

interface ForeignCallResult {
  values: ForeignCallParam[],
}
```

## Step 3 - Running tests and proving

Using the [`nargo` CLI tool](../getting_started/installation/index.md), you can use oracles in the `nargo test`, `nargo execute` and `nargo prove` commands.

We will use tests, so we can see the oracle working on tests both failing and passing.
