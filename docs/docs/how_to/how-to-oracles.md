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

This guide shows you how to use oracles in your Noir program. For the sake of clarity, it is assumed that:

- You have read the [explainer on Oracles](../explainers/explainer-oracle.md) and are comfortable with the concept
- You already have your own NoirJS and/or Nargo app. If you don't, you can use the [vite-hardhat starter](https://github.com/noir-lang/noir-starter/tree/main/vite-hardhat) as a boilerplate (for example, by using its [devcontainer in codespaces](./using-devcontainers.mdx))
- You understand the concept of a JSON-RPC server. Visit the [JSON-RPC website](https://www.jsonrpc.org/) if you need a refresher.
- You are comfortable with server-side javascript. Will skip any details on installing Node, packages, etc, so as to keep the guide short and straight to the point.

If you are looking for an end-to-end guide example with a repository you can clone and play around, follow the [Oracle Tutorial](../tutorials/oracles.md) instead.

## Rundown

This guide has 3 major steps:

1. How to modify our circuit to make use of oracle calls as unconstrained functions.
2. How to write a JSON RPC Server to resolve these oracle calls with Nargo
3. How to use them in Nargo and how to provide a custom resolver in NoirJS

## Step 1 - Modify your circuit

An oracle is defined in a circuit by defining two methods:

- An unconstrained method - This tells the Noir compiler that it is executing a Brillig block.
- A decorated oracle method - This tells the compiler that this method is an RPC call.

An example of an oracle that returns a `Field` would be:

```rust
#[oracle(getSquared)]
unconstrained fn squared(number: Field) -> Field { }

unconstrained fn get_squared(number: Field) -> Field {
    squared(number)
}
```

In this example, we're wrapping our oracle function in a unconstrained method, and decorating it with `oracle(getSquared)`. We can then call the unconstrained function as we would call any other function:

```rust
fn main(input: Field) {
    let squared = get_squared(input);
}
```

In the next section, we will make this `getSquared` be a method of the RPC server Noir will use.

:::danger

As explained in the [Oracle Explainer](../explainers/explainer-oracle.md), this `main` function is unsafe unless you constrain its return value. For example:

```rust
fn main(input: Field, target: Field) {
    let squared = get_squared(input);
    assert(squared as u64 == target as u64); // <---- constrain the return of an oracle!
}
```

:::

:::info

Currently, oracles only work with single params or array params. For example:

```rust
#[oracle(getSquared)]
unconstrained fn squared([Field; 2]) -> [Field; 2] { }
```

:::

## Step 2 - Write an RPC server

Brillig will call *one* RPC server. Most likely you will have to write your own, and you can do it in whatever language you prefer. In this guide, we will do it in Javascript.

Let's use the above example of an oracle that consumes an array with two `Field` and squares them:

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

Now, we will add our `getSquared` method, as expected by the `#[oracle(getSquared)]` decorator in our Noir code. It maps through the params array and squares the values:

```js
server.addMethod("getSquared", async (params) => {
  const values = params[0].Array.map(({ inner }) => {
    return { inner: `${inner * inner}` };
  });
  return { values: [{ Array: values }] };
});
```

:::tip

Brillig expects an object with an array of values. Each value is an object declaring to be `Single` or `Array` and returning a `inner` property *as a string*. For example:

```json
{ "values": [{ "Array": [{ "inner": "1" }, { "inner": "2"}]}]}
{ "values": [{ "Single": { "inner": "1" }}]}
{ "values": [{ "Single": { "inner": "1" }}, { "Array": [{ "inner": "1", { "inner": "2" }}]}]}
```

If you're using Typescript, the following types may be helpful in understanding the expected return value and making sure they're easy to follow:

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

:::

## Step 3 - Usage with Nargo

Using the [`nargo` CLI tool](../getting_started/installation/index.md), you can use oracles in the `nargo test`, `nargo execute` and `nargo prove` commands by passing a value to `--oracle-resolver`. For example:

```bash
nargo test --oracle-resolver http://localhost:5555
```

This tells `nargo` to use your RPC Server URL whenever it finds an oracle decorator.

## Step 4 - Usage with NoirJS

In a JS environment, an RPC server is not strictly necessary, as you may want to resolve your oracles without needing any JSON call at all. NoirJS simply expects that you pass a callback function when you generate proofs, and that callback function can be anything.

For example, if your circuit expects the host machine to provide CPU pseudo-randomness, you could simply pass it as the `foreignCallHandler`. You don't strictly need to create an RPC server to serve pseudo-randomness, as you may as well get it directly in your app:

```js
const foreignCallHandler = (name, inputs) => crypto.randomBytes(16) // etc

await noir.generateFinalProof(inputs, foreignCallHandler)
```

As one can see, in NoirJS, the [`foreignCallHandler`](../reference/NoirJS/noir_js/type-aliases/ForeignCallHandler.md) function simply means "a callback function that returns a value of type [`ForeignCallOutput`](../reference/NoirJS/noir_js/type-aliases/ForeignCallOutput.md). It doesn't have to be an RPC call like in the case for Nargo.

:::tip

Does this mean you don't have to write an RPC server like in [Step #2](#step-2---write-an-rpc-server)?

You don't technically have to, but then how would you run `nargo test` or `nargo prove`? To use both `Nargo` and `NoirJS` in your development flow, you will most certainly have to write a JSON RPC server anyway.

:::

In this case, let's make `foreignCallHandler` call the JSON RPC Server we created in [Step #2](#step-2---write-an-rpc-server), by making it a JSON RPC Client. For example, using the same `getSquared` circuit in [Step #1](#step-1---modify-your-circuit) (comments in the code):

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

:::tip

If you're in a NoirJS environment running your RPC server together with a frontend app, you'll probably hit a familiar problem in full-stack development: requests being blocked by [CORS](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) policy. For development only, you can simply install and use the [`cors` npm package](https://www.npmjs.com/package/cors) to get around the problem:

```bash
yarn add cors
```

and use it as a middleware:

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
