---
title: Building a web app with Noir and Barretenberg
description: Learn how to setup a new app that uses Noir to generate and verify zero-knowledge SNARK proofs in a typescript or javascript environment.
keywords: [how to, guide, javascript, typescript, noir, barretenberg, zero-knowledge, proofs, app]
sidebar_position: 0
pagination_next: noir/concepts/data_types/index
---

NoirJS is a Typescript package meant to work both in a browser and a server environment.

In this tutorial, we will combine NoirJS with Aztec's Barretenberg backend to build a simple web app. From here, you should get an idea on how to proceed with your own Noir projects!

You can find the complete app code for this guide [here](https://github.com/noir-lang/tiny-noirjs-app).

## Dependencies

Before we start, we want to make sure we have Node installed. If you don't have it already you can install it [here](https://nodejs.org/en/download), we recommend using [Yarn](https://yarnpkg.com/getting-started/install) as our package manager for this tutorial.

We'll also need version 1.0.0-beta.2 nargo installed, see the Noir [installation guide](../getting_started/noir_installation.md) for details.

Let's go barebones. Doing the bare minimum is not only simple, but also allows you to easily adapt it to almost any frontend framework.

Barebones means we can immediately start with the dependencies even on an empty folder 😈:

```bash
yarn add @noir-lang/noir_js@1.0.0-beta.2 @aztec/bb.js@0.72.1
```

Wait, what are these dependencies?

- `noir_js` is the main Noir package. It will execute our program, and generate the witness that will be sent to the backend.
- `bb.js` is the Typescript interface for Aztec's Barretenberg proving backend. It also uses the `wasm` version in order to run on the browser.

:::info

In this guide, we will install versions pinned to 1.0.0-beta.2. These work with Barretenberg version 0.72.1, so we are using that one version too. Feel free to try with older or later versions, though!

:::

## Setting up our Noir program

ZK is a powerful technology. An app that reveals computational correctness but doesn't reveal some of its inputs is almost unbelievable, yet Noir makes it as easy as a single line of code.

:::tip

It's not just you. We also enjoy syntax highlighting. [Check out the Language Server](../tooling/language_server.md)

:::

All you need is a `main.nr` and a `Nargo.toml` file. You can follow the [noirup](../getting_started/noir_installation.md) installation and just run `noirup -v 1.0.0-beta.2`, or just create them by hand:

```bash
mkdir -p circuit/src
touch circuit/src/main.nr circuit/Nargo.toml
```

To make our program interesting, let's give it a real use-case scenario: Bob wants to prove he is older than 18, without disclosing his age. Open `main.nr` and write:

```rust
fn main(age: u8) {
  assert(age >= 18);
}
```

This program accepts a private input called age, and simply proves this number is higher than 18. But to run this code, we need to give the compiler a `Nargo.toml` with at least a name and a type:

```toml
[package]
name = "circuit"
type = "bin"
```

This is all that we need to get started with Noir.

![my heart is ready for you, noir.js](@site/static/img/memes/titanic.jpeg)

## Compile compile compile

Finally we're up for something cool. But before we can execute a Noir program, we need to compile it into ACIR: an abstract representation.

This can be done by cd-ing into our circuit directory and running the `nargo compile` command.

```bash
cd circuits

nargo compile
```

This will write the compiled circuit into the `target` directory, which we'll then load into our JS later on.

## Setting up our app

Remember when apps only had one `html` and one `js` file? Well, that's enough for Noir webapps. Let's create them:

```bash
touch index.html index.js
```

And add something useful to our HTML file:

```html
<!DOCTYPE html>
<head>
  <style>
    .outer {
        display: flex;
        justify-content: space-between;
        width: 100%;
    }
    .inner {
        width: 45%;
        border: 1px solid black;
        padding: 10px;
        word-wrap: break-word;
    }
  </style>
</head>
<body>
  <script type="module" src="/index.js"></script>
  <h1>Noir app</h1>
  <div class="input-area">
    <input id="age" type="number" placeholder="Enter age" />
    <button id="submit">Submit Age</button>
  </div>
  <div class="outer">
    <div id="logs" class="inner"><h2>Logs</h2></div>
    <div id="results" class="inner"><h2>Proof</h2></div>
  </div>
</body>
</html>
```

It _could_ be a beautiful UI... Depending on which universe you live in. In any case, we're using some scary CSS to make two boxes that will show cool things on the screen.

As for the JS, real madmen could just `console.log` everything, but let's say we want to see things happening (the true initial purpose of JS... right?). Here's some boilerplate for that. Just paste it in `index.js`:

```js
const show = (id, content) => {
 const container = document.getElementById(id);
 container.appendChild(document.createTextNode(content));
 container.appendChild(document.createElement("br"));
};

document.getElementById("submit").addEventListener("click", async () => {
 try {
  // noir goes here
 } catch {
  show("logs", "Oh 💔");
 }
});

```

:::info

At this point in the tutorial, your folder structure should look like this:

```tree
.
└── circuit
    └── src
           └── main.nr
        Nargo.toml
    index.js
    package.json
    index.html
    ...etc
```

:::

## Some more JS

We're starting with the good stuff now. We want to execute our circuit to get the witness, and then feed that witness to Barretenberg. Luckily, both packages are quite easy to work with. Let's import them at the top of the file:

```js
import { UltraHonkBackend } from '@aztec/bb.js';
import { Noir } from '@noir-lang/noir_js';
import circuit from "./circuit/target/circuit.json";
```

And instantiate them inside our try-catch block:

```ts
// try {
const noir = new Noir(circuit);
const backend = new UltraHonkBackend(circuit.bytecode);
// }
```

## Executing and proving

Now for the app itself. We're capturing whatever is in the input when people press the submit button. Inside our `try` block, let's just grab that input and get its value. Noir will gladly execute it, and give us a witness:

```js
const age = document.getElementById("age").value;
show("logs", "Generating witness... ⏳");
const { witness } = await noir.execute({ age });
show("logs", "Generated witness... ✅");

```

:::note

For the remainder of the tutorial, everything will be happening inside the `try` block

:::

Now we're ready to prove stuff! Let's feed some inputs to our circuit and calculate the proof:

```js
show("logs", "Generating proof... ⏳");
const proof = await backend.generateProof(witness);
show("logs", "Generated proof... ✅");
show("results", proof.proof);
```

Our program is technically **done** . You're probably eager to see stuff happening! To serve this in a convenient way, we can use a bundler like `vite` by creating a `vite.config.js` file:

```bash
touch vite.config.js
```

Noir needs to load two WASM modules, but Vite doesn't include them by default in the bundle. We need to add the configuration below to `vite.config.js` to make it work.
We also need to target ESNext since `bb.js` uses top-level await, which isn't supported in some browsers.

```js
export default {
  optimizeDeps: {
    esbuildOptions: { target: "esnext" },
    exclude: ['@noir-lang/noirc_abi', '@noir-lang/acvm_js']
  }
};
```

This should be enough for vite. We don't even need to install it, just run:

```bash
yarn dlx vite
```

If it doesn't open a browser for you, just visit `localhost:5173`. You should now see the worst UI ever, with an ugly input.

![Noir Webapp UI](@site/static/img/tutorials/noirjs_webapp/webapp1.png)

Now, our circuit requires a private input `fn main(age: u8)`, and fails if it is less than 18. Let's see if it works. Submit any number above 18 (as long as it fits in 8 bits) and you should get a valid proof. Otherwise the proof won't even generate correctly.

By the way, if you're human, you shouldn't be able to understand anything on the "proof" box. That's OK. We like you, human ❤️.

## Verifying

Time to celebrate, yes! But we shouldn't trust machines so blindly. Let's add these lines to see our proof being verified:

```js
show('logs', 'Verifying proof... ⌛');
const isValid = await backend.verifyProof(proof);
show("logs", `Proof is ${isValid ? "valid" : "invalid"}... ✅`);
```

You have successfully generated a client-side Noir web app!

![coded app without math knowledge](@site/static/img/memes/flextape.jpeg)

## Next steps

At this point, you have a working ZK app that works on the browser. Actually, it works on a mobile phone too!

If you want to continue learning by doing, here are some challenges for you:

- Install [nargo](https://noir-lang.org/docs/getting_started/noir_installation) and write [Noir tests](../tooling/testing)
- Change the circuit to accept a [public input](../noir/concepts/data_types/#private--public-types) as the cutoff age. It could be different depending on the purpose, for example!
- Enjoy Noir's Rust-like syntax and write a struct `Country` that implements a trait `MinAge` with a method `get_min_age`. Then, make a struct `Person` have an `u8` as its age and a country of type `Country`. You can pass a `person` in JS just like a JSON object `person: { age, country: { min_age: 18 }}`

The world is your stage, just have fun with ZK! You can see how noirjs is used in some common frameworks in the [awesome-noir repo](https://github.com/noir-lang/awesome-noir?tab=readme-ov-file#boilerplates).
