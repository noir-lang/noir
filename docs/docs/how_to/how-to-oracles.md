---
title: How to use Oracles
description: Learn how to use oracles in your Noir program with examples in both Nargo and NoirJS. This guide also covers writing a JSON RPC server and providing custom foreign call handlers for NoirJS.
keywords:
  - Noir Programming
  - Oracles
  - Nargo
  - NoirJS
  - JSON RPC Server
  - Foreign Call Handlers
sidebar_position: 1
---

This guide shows you how to use oracles in your Noir program. It is interface-agnostic, as we will give examples in both Nargo and NoirJS.

For the sake of clarity, it is assumed that:

- You have read the [explainer on Oracles](../explainers/explainer-oracle.md) and are comfortable with the concept
- You already have your own NoirJS and/or Nargo app. You can use the [Vite Hardhat](https://github.com/noir-lang/noir-starter/tree/main/vite-hardhat) as a boilerplate (for example, by using its [devcontainer in codespaces](../how_to/using-devcontainers.md))
- You understand the concept of a JSON-RPC server. Visit the [JSON-RPC website](https://www.jsonrpc.org/) if you need a refresher.
- You are comfortable with server-side javascript. For the sake of brevity, will skip any details on installing Node, packages, etc.

If you are looking for an end-to-end guide with a code example you can clone and play around, follow the [Oracle Tutorial](../tutorials/oracles.md) instead.

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

### Nargo CLI

Using the [`nargo` CLI tool](../getting_started/installation/index.md), you can use oracles in the `nargo test`, `nargo execute` and `nargo prove` commands, for example:

```bash
nargo test --oracle-resolver http://localhost:5555
```

This tells `nargo` to use your RPC Server URL whenever it finds an oracle decorator

### NoirJS - Is it all RPCs?

In a JS environment, you don't necessarily need an RPC server, as you're already working with a programming language that allows you to get values however you want. NoirJS simply expects that you pass a callback function when you generate proofs, and that callback function can be anything.

For example, if your circuit expects the host machine to provide CPU pseudo-randomness, you could simply pass it as the `foreignCallHandler`:

```js
const foreignCallHandler = (name, inputs) => crypto.randomBytes(16) // etc

await noir.generateFinalProof(inputs, foreignCallHandler)
```

In NoirJS, the [`foreignCallHandler`](../reference/NoirJS/noir_js/type-aliases/ForeignCallHandler.md) function means "a callback function that returns a value of type [`ForeignCallOutput`]("../reference/NoirJS/noir_js/type-aliases/ForeignCallOutput.md)".

:::note

Does this mean you don't have to write an RPC server like in [Step #2](#step-2---write-an-rpc-server)? You don't technically have to, but then how would you run `nargo test` or `nargo prove`?

To use both `Nargo` and `NoirJS` in your development flow, you will most certainly have to write a JSON RPC server anyway.

:::

In this case, let's make `foreignCallHandler` call the same JSON RPC Server, by making it a JSON RPC Client. For example, using the same `getSquared` circuit in [Step #1](#step-1---modify-your-circuit):

```js
import { JSONRPCClient } from "json-rpc-2.0";

// declaring the JSONRPCClient
const client = new JSONRPCClient((jsonRPCRequest) => {
// hitting the same JSON RPC Server we coded above
 return fetch("http://localhost:5555", {
  method: "POST",
  headers: {
   "content-type": "application/json",
  },
  body: JSON.stringify(jsonRPCRequest),
 }).then((response) => {
  if (response.status === 200) {
   return response
    .json()
    .then((jsonRPCResponse) => client.receive(jsonRPCResponse));
  } else if (jsonRPCRequest.id !== undefined) {
   return Promise.reject(new Error(response.statusText));
  }
 });
});

// declaring a function that takes the name of the foreign call (getSquared) and the inputs
const foreignCallHandler = async (name, input) => {
    // notice that the "inputs" parameter contains *all* the inputs
    // in this case we to make the RPC request with the first parameter "numbers", which would be input[0]
    const oracleReturn = await client.request(name, [
        { Array: input[0].map((i) => ({ inner: i })) },
    ]);
    return [oracleReturn.values[0].Array.map((x) => x.inner)];
};

// the rest of your NoirJS code
const input = { numbers: [1, 2], target: [1, 4] };
const { witness } = await noir.execute(numbers, foreignCallHandler);
```

:::note

If you're in a NoirJS environment running your RPC server together with a frontend app, you'll probably hit a familiar problem in full-stack development: requests being blocked by CORS policy. In development, you can simply install and use the `cors` npm package to get around the problem:

```bash
yarn add cors # npm i cors
```

and use it as a middleware right after starting your express server:

```js
import cors from "cors";

const app = express();
app.use(cors())
```

:::

## Conclusion

Hopefully by the end of this guide, you should be able to:

- Write your own logic around Oracles and how to write a JSON RPC server to make them work with your Nargo commands.
- Provide custom foreign call handlers for NoirJS.

If you are looking for a more end-to-end guide with a code example you can clone and play around, follow the [Oracle Tutorial](../tutorials/oracles.md)
