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

This guide shows you how to use oracles in your Noir program. For the sake of clarity, it assumes that:

- You have read the [explainer on Oracles](../explainers/explainer-oracle.md) and are comfortable with the concept.
- You have a Noir program to add oracles to. You can create one using the [vite-hardhat starter](https://github.com/noir-lang/noir-starter/tree/main/vite-hardhat) as a boilerplate.
- You understand the concept of a JSON-RPC server. Visit the [JSON-RPC website](https://www.jsonrpc.org/) if you need a refresher.
- You are comfortable with server-side JavaScript (e.g. Node.js, managing packages, etc.).

## Rundown

This guide has 3 major steps:

1. How to modify our Noir program to make use of oracle calls as unconstrained functions
2. How to write a JSON RPC Server to resolve these oracle calls with Nargo
3. How to use them in Nargo and how to provide a custom resolver in NoirJS

## Step 1 - Modify your Noir program

An oracle is defined in a Noir program by defining two methods:

- An unconstrained method - This tells the compiler that it is executing an [unconstrained functions](../noir/concepts//unconstrained.md).
- A decorated oracle method - This tells the compiler that this method is an RPC call.

An example of an oracle that returns a `Field` would be:

```rust
#[oracle(getSqrt)]
unconstrained fn sqrt(number: Field) -> Field { }

unconstrained fn get_sqrt(number: Field) -> Field {
    sqrt(number)
}
```

In this example, we're wrapping our oracle function in an unconstrained method, and decorating it with `oracle(getSqrt)`. We can then call the unconstrained function as we would call any other function:

```rust
fn main(input: Field) {
    let sqrt = get_sqrt(input);
}
```

In the next section, we will make this `getSqrt` (defined on the `sqrt` decorator) be a method of the RPC server Noir will use.

:::danger

As explained in the [Oracle Explainer](../explainers/explainer-oracle.md), this `main` function is unsafe unless you constrain its return value. For example:

```rust
fn main(input: Field) {
    let sqrt = get_sqrt(input);
    assert(sqrt.pow_32(2) as u64 == input as u64); // <---- constrain the return of an oracle!
}
```

:::

:::info

Currently, oracles only work with single params or array params. For example:

```rust
#[oracle(getSqrt)]
unconstrained fn sqrt([Field; 2]) -> [Field; 2] { }
```

:::

## Step 2 - Write an RPC server

Brillig will call *one* RPC server. Most likely you will have to write your own, and you can do it in whatever language you prefer. In this guide, we will do it in Javascript.

Let's use the above example of an oracle that consumes an array with two `Field` and returns their square roots:

```rust
#[oracle(getSqrt)]
unconstrained fn sqrt(input: [Field; 2]) -> [Field; 2] { }

unconstrained fn get_sqrt(input: [Field; 2]) -> [Field; 2] {
    sqrt(input)
}

fn main(input: [Field; 2]) {
    let sqrt = get_sqrt(input);
    assert(sqrt[0].pow_32(2) as u64 == input[0] as u64);
    assert(sqrt[1].pow_32(2) as u64 == input[1] as u64);
}

#[test]
fn test() {
    let input = [4, 16];
    main(input);
}
```

:::info

Why square root?

In general, computing square roots is computationally more expensive than multiplications, which takes a toll when speaking about ZK applications. In this case, instead of calculating the square root in Noir, we are using our oracle to offload that computation to be made in plain. In our circuit we can simply multiply the two values.

:::

Now, we should write the correspondent RPC server, starting with the [default JSON-RPC 2.0 boilerplate](https://www.npmjs.com/package/json-rpc-2.0#example):

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
   res.json(jsonRPCResponse);
  } else {
   res.sendStatus(204);
  }
 });
});

app.listen(5555);
```

Now, we will add our `getSqrt` method, as expected by the `#[oracle(getSqrt)]` decorator in our Noir code. It maps through the params array and returns their square roots:

```js
server.addMethod("resolve_foreign_call", async (params) => {
    if (params[0].function !== "getSqrt") {
        throw Error("Unexpected foreign call")
    };
    const values = params[0].inputs[0].map((field) => {
        return `${Math.sqrt(parseInt(field, 16))}`;
    });
    return { values: [values] };
});
```

If you're using Typescript, the following types may be helpful in understanding the expected return value and making sure they're easy to follow:

```js
export type ForeignCallSingle = string;

export type ForeignCallArray = string[];

export type ForeignCallResult = {
  values: (ForeignCallSingle | ForeignCallArray)[];
};
```

:::info Multidimensional Arrays

If the Oracle function is returning an array containing other arrays, such as `[['1','2],['3','4']]`, you need to provide the values in JSON as flattened values. In the previous example, it would be `['1', '2', '3', '4']`. In the Noir program, the Oracle signature can use a nested type, the flattened values will be automatically converted to the nested type.

:::

## Step 3 - Usage with Nargo

Using the [`nargo` CLI tool](../getting_started/installation/index.md), you can use oracles in the `nargo test` and `nargo execute`  commands by passing a value to `--oracle-resolver`. For example:

```bash
nargo test --oracle-resolver http://localhost:5555
```

This tells `nargo` to use your RPC Server URL whenever it finds an oracle decorator.

## Step 4 - Usage with NoirJS

In a JS environment, an RPC server is not strictly necessary, as you may want to resolve your oracles without needing any JSON call at all. NoirJS simply expects that you pass a callback function when you generate proofs, and that callback function can be anything.

For example, if your Noir program expects the host machine to provide CPU pseudo-randomness, you could simply pass it as the `foreignCallHandler`. You don't strictly need to create an RPC server to serve pseudo-randomness, as you may as well get it directly in your app:

```js
const foreignCallHandler = (name, inputs) => crypto.randomBytes(16) // etc

await noir.execute(inputs, foreignCallHandler)
```

As one can see, in NoirJS, the [`foreignCallHandler`](../reference/NoirJS/noir_js/type-aliases/ForeignCallHandler.md) function simply means "a callback function that returns a value of type [`ForeignCallOutput`](../reference/NoirJS/noir_js/type-aliases/ForeignCallOutput.md). It doesn't have to be an RPC call like in the case for Nargo.

:::tip

Does this mean you don't have to write an RPC server like in [Step #2](#step-2---write-an-rpc-server)?

You don't technically have to, but then how would you run `nargo test`? To use both `Nargo` and `NoirJS` in your development flow, you will have to write a JSON RPC server.

:::

In this case, let's make `foreignCallHandler` call the JSON RPC Server we created in [Step #2](#step-2---write-an-rpc-server), by making it a JSON RPC Client.

For example, using the same `getSqrt` program in [Step #1](#step-1---modify-your-noir-program) (comments in the code):

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

// declaring a function that takes the name of the foreign call (getSqrt) and the inputs
const foreignCallHandler = async (name, input) => {
  const inputs = input[0].map((i) => i.toString("hex"))
  // notice that the "inputs" parameter contains *all* the inputs
  // in this case we to make the RPC request with the first parameter "numbers", which would be input[0]
  const oracleReturn = await client.request("resolve_foreign_call", [
    {
      function: name,
      inputs: [inputs]
    },
  ]);
  return [oracleReturn.values[0]];
};

// the rest of your NoirJS code
const input = { input: [4, 16] };
const { witness } = await noir.execute(input, foreignCallHandler);
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