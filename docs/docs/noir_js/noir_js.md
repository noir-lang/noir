---
title: NoirJS
description: Interact with Noir in Typescript or Javascript
keywords: [Noir project, javascript, typescript, node.js, browser, react]
---

NoirJS is a TypeScript library that make it easy to use Noir on your dapp, webapp, Node.js server, website, etc.

A typical workflow would be composed of two major elements:

- NoirJS
- Proving backend of choice's JavaScript package

<!-- TODO add "and noir_wasm" to the end once it's ready -->

To install NoirJS, install Node.js if you have not already and run this in your JavaScript project:

```bash
npm i @noir-lang/noir_js
```

## Proving backend

Since Noir is backend agnostic, you can instantiate NoirJS without any backend (i.e. to execute a function). But for proving, you would have to instantiate NoirJS with any of the supported backends through their own `js` interface.

### Barretenberg

Aztec Labs maintains the `barretenberg` proving backend, which you can instantiate and make use of alongside NoirJS. It is also the default proving backend installed and used with Nargo, the Noir CLI tool.

To install its JavaScript library, run this in your project:

```bash
npm i @noir-lang/backend_barretenberg
```

For more details on how to instantiate and use the libraries, refer to the [Full Noir App Guide](./getting_started/01_tiny_noir_app.md) and [Reference](./reference/01_noirjs.md) sections.
