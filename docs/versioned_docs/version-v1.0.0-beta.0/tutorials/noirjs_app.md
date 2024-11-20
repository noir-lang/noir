---
title: Building a web app with NoirJS
description: Learn how to setup a new app that uses Noir to generate and verify zero-knowledge SNARK proofs in a typescript or javascript environment.
keywords: [how to, guide, javascript, typescript, noir, barretenberg, zero-knowledge, proofs, app]
sidebar_position: 0
pagination_next: noir/concepts/data_types/index
---

NoirJS is a set of packages meant to work both in a browser and a server environment. In this tutorial, we will build a simple web app using them. From here, you should get an idea on how to proceed with your own Noir projects!

You can find the complete app code for this guide [here](https://github.com/noir-lang/tiny-noirjs-app).

## Setup

:::note

Feel free to use whatever versions, just keep in mind that Nargo and the NoirJS packages are meant to be in sync. For example, Nargo 0.31.x matches `noir_js@0.31.x`, etc.

In this guide, we will be pinned to 0.31.0.

:::

Before we start, we want to make sure we have Node, Nargo and the Barretenberg proving system (`bb`) installed.

We start by opening a terminal and executing `node --version`. If we don't get an output like `v20.10.0`, that means node is not installed. Let's do that by following the handy [nvm guide](https://github.com/nvm-sh/nvm?tab=readme-ov-file#install--update-script).

As for `Nargo`, we can follow the [Nargo guide](../getting_started/quick_start.md) to install it. If you're lazy, just paste this on a terminal and run `noirup`:

```sh
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
```

Follow the instructions on [this page](https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg/cpp/src/barretenberg/bb#installation) to install `bb`.
Version 0.41.0 is compatible with `nargo` version 0.31.0, which you can install with `bbup -v 0.41.0` once `bbup` is installed.

Easy enough. Onwards!

## Our project

ZK is a powerful technology. An app that doesn't reveal one of the inputs to _anyone_ is almost unbelievable, yet Noir makes it as easy as a single line of code.

In fact, it's so simple that it comes nicely packaged in `nargo`. Let's do that!

### Nargo

Run:

```bash
nargo new circuit
```

And... That's about it. Your program is ready to be compiled and run.

To compile, let's `cd` into the `circuit` folder to enter our project, and call:

```bash
nargo compile
```

This compiles our circuit into `json` format and add it to a new `target` folder.

:::info

At this point in the tutorial, your folder structure should look like this:

```tree
.
└── circuit <---- our working directory
    ├── Nargo.toml
    ├── src
    │   └── main.nr
    └── target
        └── circuit.json
```

:::

### Node and Vite

If you want to explore Nargo, feel free to go on a side-quest now and follow the steps in the
[getting started](../getting_started/quick_start.md) guide. However, we want our app to run on the browser, so we need Vite.

Vite is a powerful tool to generate static websites. While it provides all kinds of features, let's just go barebones with some good old vanilla JS.

To do this this, go back to the previous folder (`cd ..`) and create a new vite project by running `npm create vite` and choosing "Vanilla" and "Javascript".

A wild `vite-project` directory should now appear in your root folder! Let's not waste any time and dive right in:

```bash
cd vite-project
```

### Setting Up Vite and Configuring the Project

Before we proceed with any coding, let's get our environment tailored for Noir. We'll start by laying down the foundations with a `vite.config.js` file. This little piece of configuration is our secret sauce for making sure everything meshes well with the NoirJS libraries and other special setups we might need, like handling WebAssembly modules. Here’s how you get that going:

#### Creating the vite.config.js

In your freshly minted `vite-project` folder, create a new file named `vite.config.js` and open it in your code editor. Paste the following to set the stage:

```javascript
import { defineConfig } from 'vite';
import copy from 'rollup-plugin-copy';
import fs from 'fs';
import path from 'path';

const wasmContentTypePlugin = {
  name: 'wasm-content-type-plugin',
  configureServer(server) {
    server.middlewares.use(async (req, res, next) => {
      if (req.url.endsWith('.wasm')) {
        res.setHeader('Content-Type', 'application/wasm');
        const newPath = req.url.replace('deps', 'dist');
        const targetPath = path.join(__dirname, newPath);
        const wasmContent = fs.readFileSync(targetPath);
        return res.end(wasmContent);
      }
      next();
    });
  },
};

export default defineConfig(({ command }) => {
  if (command === 'serve') {
    return {
      build: {
        target: 'esnext',
        rollupOptions: {
          external: ['@aztec/bb.js']
        }
      },
      optimizeDeps: {
        esbuildOptions: {
          target: 'esnext'
        }
      },
      plugins: [
        copy({
          targets: [{ src: 'node_modules/**/*.wasm', dest: 'node_modules/.vite/dist' }],
          copySync: true,
          hook: 'buildStart',
        }),
        command === 'serve' ? wasmContentTypePlugin : [],
      ],
    };
  }

  return {};
});
```

#### Install Dependencies

Now that our stage is set, install the necessary NoirJS packages along with our other dependencies:

```bash
npm install && npm install @noir-lang/backend_barretenberg@0.31.0 @noir-lang/noir_js@0.31.0
npm install rollup-plugin-copy --save-dev
```

:::info

At this point in the tutorial, your folder structure should look like this:

```tree
.
└── circuit
    └── ...etc...
└── vite-project <---- our working directory
    └── ...etc...
```

:::

#### Some cleanup

`npx create vite` is amazing but it creates a bunch of files we don't really need for our simple example. Actually, let's just delete everything except for `vite.config.js`, `index.html`, `main.js` and `package.json`. I feel lighter already.

![my heart is ready for you, noir.js](@site/static/img/memes/titanic.jpeg)

## HTML

Our app won't run like this, of course. We need some working HTML, at least. Let's open our broken-hearted `index.html` and replace everything with this code snippet:

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
  <script type="module" src="/main.js"></script>
  <h1>Noir app</h1>
  <div class="input-area">
    <input id="guessInput" type="number" placeholder="Enter your guess" />
    <button id="submitGuess">Submit Guess</button>
  </div>
  <div class="outer">
    <div id="logs" class="inner"><h2>Logs</h2></div>
    <div id="results" class="inner"><h2>Proof</h2></div>
  </div>
</body>
</html>
```

It _could_ be a beautiful UI... Depending on which universe you live in.

## Some good old vanilla Javascript

Our love for Noir needs undivided attention, so let's just open `main.js` and delete everything (this is where the romantic scenery becomes a bit creepy).

Start by pasting in this boilerplate code:

```js
function display(container, msg) {
  const c = document.getElementById(container);
  const p = document.createElement('p');
  p.textContent = msg;
  c.appendChild(p);
}

document.getElementById('submitGuess').addEventListener('click', async () => {
  try {
    // here's where love happens
  } catch (err) {
    display('logs', 'Oh 💔 Wrong guess');
  }
});
```

The display function doesn't do much. We're simply manipulating our website to see stuff happening. For example, if the proof fails, it will simply log a broken heart 😢

:::info

At this point in the tutorial, your folder structure should look like this:

```tree
.
└── circuit
    └── ...same as above
└── vite-project
    ├── vite.config.js
    ├── main.js
    ├── package.json
    └── index.html
```

You'll see other files and folders showing up (like `package-lock.json`, `node_modules`) but you shouldn't have to care about those.

:::

## Some NoirJS

We're starting with the good stuff now. If you've compiled the circuit as described above, you should have a `json` file we want to import at the very top of our `main.js` file:

```ts
import circuit from '../circuit/target/circuit.json';
```

[Noir is backend-agnostic](../index.mdx#whats-new-about-noir). We write Noir, but we also need a proving backend. That's why we need to import and instantiate the two dependencies we installed above: `BarretenbergBackend` and `Noir`. Let's import them right below:

```js
import { BarretenbergBackend, BarretenbergVerifier as Verifier } from '@noir-lang/backend_barretenberg';
import { Noir } from '@noir-lang/noir_js';
```

And instantiate them inside our try-catch block:

```ts
// try {
const backend = new BarretenbergBackend(circuit);
const noir = new Noir(circuit);
// }
```

:::note

For the remainder of the tutorial, everything will be happening inside the `try` block

:::

## Our app

Now for the app itself. We're capturing whatever is in the input when people press the submit button. Just add this:

```js
const x = parseInt(document.getElementById('guessInput').value);
const input = { x, y: 2 };
```

Now we're ready to prove stuff! Let's feed some inputs to our circuit and calculate the proof:

```js
await setup(); // let's squeeze our wasm inits here

display('logs', 'Generating proof... ⌛');
const { witness } = await noir.execute(input);
const proof = await backend.generateProof(witness);
display('logs', 'Generating proof... ✅');
display('results', proof.proof);
```

You're probably eager to see stuff happening, so go and run your app now!

From your terminal, run `npm run dev`. If it doesn't open a browser for you, just visit `localhost:5173`. You should now see the worst UI ever, with an ugly input.

![Getting Started 0](@site/static/img/noir_getting_started_1.png)

Now, our circuit says `fn main(x: Field, y: pub Field)`. This means only the `y` value is public, and it's hardcoded above: `input = { x, y: 2 }`. In other words, you won't need to send your secret`x` to the verifier!

By inputting any number other than 2 in the input box and clicking "submit", you should get a valid proof. Otherwise the proof won't even generate correctly. By the way, if you're human, you shouldn't be able to understand anything on the "proof" box. That's OK. We like you, human ❤️.

## Verifying

Time to celebrate, yes! But we shouldn't trust machines so blindly. Let's add these lines to see our proof being verified:

```js
display('logs', 'Verifying proof... ⌛');
const isValid = await backend.verifyProof(proof);

// or to cache and use the verification key:
// const verificationKey = await backend.getVerificationKey();
// const verifier = new Verifier();
// const isValid = await verifier.verifyProof(proof, verificationKey);

if (isValid) display('logs', 'Verifying proof... ✅');
```

You have successfully generated a client-side Noir web app!

![coded app without math knowledge](@site/static/img/memes/flextape.jpeg)

## Further Reading

You can see how noirjs is used in a full stack Next.js hardhat application in the [noir-starter repo here](https://github.com/noir-lang/noir-starter/tree/main/vite-hardhat). The example shows how to calculate a proof in the browser and verify it with a deployed Solidity verifier contract from noirjs.

You should also check out the more advanced examples in the [noir-examples repo](https://github.com/noir-lang/noir-examples), where you'll find reference usage for some cool apps.

## UltraHonk Backend

Barretenberg has recently exposed a new UltraHonk backend. We can use UltraHonk in NoirJS after version 0.33.0. Everything will be the same as the tutorial above, except that the class we need to import will change:

```js
import { UltraHonkBackend, UltraHonkVerifier as Verifier } from '@noir-lang/backend_barretenberg';
```

The backend will then be instantiated as such:

```js
const backend = new UltraHonkBackend(circuit);
```

Then all the commands to prove and verify your circuit will be same.

The only feature currently unsupported with UltraHonk are [recursive proofs](../explainers/explainer-recursion.md).
