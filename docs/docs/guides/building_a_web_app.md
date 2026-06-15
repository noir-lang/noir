---
title: Building a web app with Noir and Barretenberg
description: Learn how to setup a new app that uses Noir to generate and verify zero-knowledge SNARK proofs in a typescript or javascript environment.
keywords: [how to, guide, javascript, typescript, noir, barretenberg, zero-knowledge, proofs, app]
pagination_next: language/data_types/index
---

NoirJS is a Typescript package meant to work both in a browser and a server environment.

In this tutorial, we will combine NoirJS with Aztec's Barretenberg backend to build a simple web app. From here, you should get an idea of how to proceed with your own Noir projects.

You can find the complete app code for this guide [here](https://github.com/noir-lang/tiny-noirjs-app).

## Dependencies

Before we start, we want to make sure we have Node installed. If you don't have it already you can install it [here](https://nodejs.org/en/download), we recommend using [Yarn](https://yarnpkg.com/getting-started/install) as our package manager for this tutorial.

We'll also need version 1.0.0-beta.20 nargo installed, see the Noir [installation guide](../installation.md) for details.

We'll keep this barebones. Doing the bare minimum is not only simple, but also allows you to easily adapt it to almost any frontend framework.

This means we can start with the dependencies even on an empty folder:

```bash
yarn add @noir-lang/noir_js@1.0.0-beta.20 @aztec/bb.js@3.0.0-nightly.20251104 buffer vite vite-plugin-node-polyfills@0.17.0
```

Wait, what are these dependencies?

- `noir_js` is the main Noir package. It will execute our program, and generate the witness that will be sent to the backend.
- `bb.js` is the Typescript interface for Aztec's Barretenberg proving backend. It also uses the `wasm` version in order to run on the browser.

:::info

In this guide, we will install versions pinned to 1.0.0-beta.20. These work with Barretenberg version 3.0.0-nightly.20251104, so we are using that version too. You can also try older or later versions.

:::

## Setting up our Noir program

ZK is a powerful technology: an app can prove that a computation was carried out correctly without revealing some of its inputs, and Noir lets you express this in a single line of code.

:::tip

For syntax highlighting and editor support, check out the [Language Server](../tooling/language_server.md).

:::

A Noir program needs a `main.nr` and a `Nargo.toml` file. `nargo new` scaffolds both for us:

```bash
nargo new circuit
```

This creates a `circuit` directory containing a `Nargo.toml` (already populated with the package name and `type = "bin"`) and a `src/main.nr`.

To make our program interesting, let's give it a real use-case scenario: Bob wants to prove he is older than 18, without disclosing his age. Open `circuit/src/main.nr` and replace its contents with:

#include_code age_check examples/browser/src/main.nr rust

This program accepts a private input called age, and simply proves this number is higher than 18.

This is all that we need to get started with Noir.

## Compiling the circuit

Before we can execute a Noir program, we need to compile it into ACIR, an abstract representation of the circuit.

This can be done by cd-ing into our circuit directory and running the `nargo compile` command.

```bash
cd circuit

nargo compile
```

This will write the compiled circuit into the `target` directory, which we'll then load into our JS later on.

## Setting up our app

A single `html` file and a single `js` file are enough for this app. Let's create them in the project root:

```bash
touch index.html index.js
```

And add something useful to our HTML file:

#include_code index examples/browser/index.html html

This is a minimal UI: some CSS to make two boxes that will display output on the screen.

For the JS, rather than logging everything to the console, we'll add a small helper to display output on the page. Paste this boilerplate into `index.js`:

```js
#include_code show_function examples/browser/index.js raw

document.getElementById('submit').addEventListener('click', async () => {
  try {
    // code will go in here
  }  catch {
    show('logs', 'Something went wrong');
  }
});
```

:::info

At this point in the tutorial, your folder structure should look like this:

```tree
.
├── circuit
│   ├── Nargo.toml
│   ├── src
│   │   └── main.nr
│   └── target
│       └── circuit.json
├── index.html
├── index.js
├── package.json
├── etc...
```

:::

## Some more JS

Next we want to execute our circuit to get the witness, and then feed that witness to Barretenberg. Both packages are straightforward to work with. Let's import them at the top of the file and initialize the WASM modules:

#include_code imports examples/browser/index.js js

And instantiate them inside our try-catch block:

#include_code init examples/browser/index.js js

## Executing and proving

Now for the app itself. We're capturing whatever is in the input when the user presses the submit button. Inside our `try` block, let's grab that input and get its value. Noir will execute it and give us a witness:

#include_code execute examples/browser/index.js js

:::note

For the remainder of the tutorial, everything will be happening inside the `try` block

:::

Now we're ready to prove. Let's feed some inputs to our circuit and calculate the proof:

#include_code prove examples/browser/index.js js

Our program is now complete. To serve it conveniently, we can use a bundler like `vite` by creating a `vite.config.js` file:

```bash
touch vite.config.js
```

Noir needs to load two WASM modules, but Vite doesn't include them by default in the bundle. We need to add the configuration below to `vite.config.js` to make it work.
We also need to target ESNext since `bb.js` uses top-level await, which isn't supported in some browsers.

#include_code config examples/browser/vite.config.js js

This should be enough for vite. We don't even need to install it, just run:

```bash
yarn vite dev
```

If it doesn't open a browser for you, visit `localhost:5173`. You should now see the minimal UI with a single input.

![Noir Webapp UI](@site/static/img/guides/noirjs_webapp/webapp1.png)

Now, our circuit requires a private input `fn main(age: u8)`, and fails if it is less than 18. Let's see if it works. Submit any number above 18 (as long as it fits in 8 bits) and you should get a valid proof. Otherwise the proof won't generate correctly.

The contents of the "proof" box aren't meant to be human-readable; it's the raw proof data.

## Verifying

Generating a proof isn't enough on its own; we also want to verify it. Let's add these lines to verify the proof:

#include_code verify examples/browser/index.js js

You have successfully generated a client-side Noir web app!

## Next steps

At this point, you have a working ZK app that runs in the browser (and on mobile too).

If you want to continue learning by doing, here are some challenges for you:

- Install [nargo](https://noir-lang.org/docs/installation) and write [Noir tests](../tooling/tests)
- Change the circuit to accept a [public input](../language/data_types/#private--public-types) as the cutoff age. It could be different depending on the purpose, for example!
- Using Noir's Rust-like syntax, write a struct `Country` that implements a trait `MinAge` with a method `get_min_age`. Then, make a struct `Person` have a `u8` as its age and a country of type `Country`. You can pass a `person` in JS just like a JSON object `person: { age, country: { min_age: 18 }}`

For more examples, see how NoirJS is used with common frameworks in the [awesome-noir repo](https://github.com/noir-lang/awesome-noir?tab=readme-ov-file#boilerplates).
