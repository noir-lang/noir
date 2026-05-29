---
title: Overview
description:
  A map of every Noir reference surface — language, standard library, CLI,
  JavaScript/TypeScript, ACIR, and compiler internals — and which one is
  authoritative for each kind of question.
keywords:
  [
    Noir,
    reference,
    documentation,
    Nargo,
    NoirJS,
    noir_wasm,
    ACIR,
    standard library,
    compiler,
  ]
sidebar_position: 0
---

# References

Noir's reference material is spread across several surfaces, each generated and
maintained differently: narrative docs on this site, generated CLI docs, TypeDoc
output for the JavaScript packages, generated standard library API docs, the
ACIR reference, and the compiler's source-level docs.

This page is the single map. Use it to find which reference is **authoritative**
for a given kind of question, then follow the link to that surface. The tables
below group the references by what you are trying to do.

## Writing Noir

| Reference | Use it for | Authoritative source |
| --- | --- | --- |
| [Language](../language/data_types/index.md) | Syntax and semantics of the Noir language: types, functions, generics, traits, comptime, oracles, and more. | Narrative docs in the **Language** section of this site. |
| [Standard Library (guides)](../libraries/standard_library/cryptographic_primitives/index.md) | How to use stdlib features (hashing, signatures, containers, metaprogramming) with examples. | Narrative docs in the **Libraries → Standard Library** section of this site. |
| [Standard Library (generated API)](https://noir-lang.github.io/noir/stdlib/index.html) | The complete, exhaustive `std` API: every module, function, and signature. | Generated from the stdlib source by `nargo doc` and published to GitHub Pages. |

The narrative standard library docs explain the commonly used pieces with
context; the generated API is the exhaustive, always-in-sync listing. When the
two disagree, the generated API reflects the current source.

## Tooling and CLI

| Reference | Use it for | Authoritative source |
| --- | --- | --- |
| [Nargo CLI](./nargo_commands.md) | Every `nargo` command and flag (`compile`, `execute`, `test`, `prove`-adjacent tooling, etc.). | Generated from the `nargo` CLI definitions at docs build time. |
| [Noir Codegen for TypeScript](../tooling/noir_codegen.md) | Generating type-safe TypeScript bindings from compiled Noir programs. | Narrative docs in the **Tooling** section of this site. |
| [Language Server (LSP)](../tooling/language_server.md) | Editor integration and supported LSP features. | Narrative docs in the **Tooling** section of this site. |
| [Debugger](../tooling/debugger/index.mdx) | Stepping through Noir execution in VS Code or the REPL. | Narrative docs in the **Tooling** section of this site. |

## JavaScript and TypeScript integration

| Reference | Use it for | Authoritative source |
| --- | --- | --- |
| [NoirJS — `noir_js`](./NoirJS/noir_js/index.md) | Executing programs and generating witnesses from JavaScript (`Noir`, foreign-call handlers, black-box helpers). | TypeDoc, generated from `tooling/noir_js`. |
| [Noir WASM — `noir_wasm`](./NoirJS/noir_wasm/index.md) | Compiling Noir programs from JavaScript/WASM (`compile`, `compile_contract`, file manager). | TypeDoc, generated from `compiler/wasm`. |

## Low-level and compiler internals

| Reference | Use it for | Authoritative source |
| --- | --- | --- |
| [ACIR reference](https://noir-lang.github.io/noir/docs/acir/circuit/index.html) | The Abstract Circuit Intermediate Representation that the compiler emits and proving backends consume. | `rustdoc` for the `acir` crate, published to GitHub Pages. |
| [Compiler crates (rustdoc)](https://noir-lang.github.io/noir/docs/) | Internal APIs of the compiler crates (`noirc_frontend`, `noirc_evaluator`, `nargo`, and more). | `rustdoc` for the workspace, published to GitHub Pages. |
| [Design notes](https://github.com/noir-lang/noir/tree/master/design) | The rationale behind specific language, compiler, and tooling decisions (e.g. comptime, oracles). | The `design/` directory in the `noir-lang/noir` repository. |

The design notes record *why* things work the way they do and are aimed at
contributors; they are not user-facing language documentation.

## Beyond this repository

| Reference | Use it for |
| --- | --- |
| [Ecosystem libraries](https://github.com/noir-lang/awesome-noir?tab=readme-ov-file#libraries) | Community-maintained Noir libraries. |
| [Proving backend guides](https://barretenberg.aztec.network/docs) | Generating Solidity verifiers, recursive aggregation, and running in the browser with Barretenberg. |
