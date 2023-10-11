---
title: Noir JS
description: Learn how to use noir js to use Noir in a Typescript or Javascript environment
keywords: [Noir project, javascript, typescript, node.js, browser, react]
---

Noir JS are a set of typescript libraries that make it easy to use Noir on your dapp, webapp, node.js server, website, etc.

It is composed of two major elements:

- Noir
- Backend proving system

<!-- TODO add "and noir_wasm" to the end once it's ready -->

Your only concern should be to write Noir. Noir.js will work out-of-the box and abstract all the components, such as the ACVM and others.

## Barretenberg

Since Noir is backend agnostic, you can instantiate `noir_js` with supported backends through their own `js` interface.

Aztec Labs maintains the `barretenberg` backend. You can use it to instantiate your `Noir` class.
