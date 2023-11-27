---
title: Dependencies
description:
  Learn how to specify and manage dependencies in Nargo, allowing you to upload packages to GitHub
  and use them easily in your project.
keywords: [Nargo, dependencies, GitHub, package management, versioning]
---

Nargo allows you to upload packages to GitHub and use them as dependencies.

## Specifying a dependency

Specifying a dependency requires a tag to a specific commit and the git url to the url containing
the package.

Currently, there are no requirements on the tag contents. If requirements are added, it would follow
semver 2.0 guidelines.

> Note: Without a `tag` , there would be no versioning and dependencies would change each time you
> compile your project.

For example, to add the [ecrecover-noir library](https://github.com/colinnielsen/ecrecover-noir) to your project, add it to `Nargo.toml`:

```toml
# Nargo.toml

[dependencies]
ecrecover = {tag = "v0.8.0", git = "https://github.com/colinnielsen/ecrecover-noir"}
```

If the module is in a subdirectory, you can define a subdirectory in your git repository, for example:

```toml
# Nargo.toml

[dependencies]
easy_private_token_contract = {tag ="v0.1.0-alpha62", git = "https://github.com/AztecProtocol/aztec-packages", directory = "yarn-project/noir-contracts/src/contracts/easy_private_token_contract"}
```

## Specifying a local dependency

You can also specify dependencies that are local to your machine.

For example, this file structure has a library and binary crate

```tree
├── binary_crate
│   ├── Nargo.toml
│   └── src
│       └── main.nr
└── liba
    ├── Nargo.toml
    └── src
        └── lib.nr
```

Inside of the binary crate, you can specify:

```toml
# Nargo.toml

[dependencies]
libA = { path = "../liba" }
```

## Importing dependencies

You can import a dependency to a Noir file using the following syntax. For example, to import the
ecrecover-noir library and local liba referenced above:

```rust
use dep::ecrecover;
use dep::libA;
```

You can also import only the specific parts of dependency that you want to use, like so:

```rust
use dep::std::hash::sha256;
use dep::std::scalar_mul::fixed_base;
```

Lastly, as demonstrated in the
[elliptic curve example](../standard_library/cryptographic_primitives/ec_primitives#examples), you
can import multiple items in the same line by enclosing them in curly braces:

```rust
use dep::std::ec::tecurve::affine::{Curve, Point};
```

We don't have a way to consume libraries from inside a [workspace](./workspaces) as external dependencies right now.

Inside a workspace, these are consumed as `{ path = "../to_lib" }` dependencies in Nargo.toml.

## Dependencies of Dependencies

Note that when you import a dependency, you also get access to all of the dependencies of that package.

For example, the [phy_vector](https://github.com/resurgencelabs/phy_vector) library imports an [fraction](https://github.com/resurgencelabs/fraction) library. If you're importing the phy_vector library, then you can access the functions in fractions library like so:

```rust
use dep::phy_vector;

fn main(x : Field, y : pub Field) {
  //...
  let f = phy_vector::fraction::toFraction(true, 2, 1);
  //...
}
```

## Available Libraries

Noir does not currently have an official package manager. You can find a list of available Noir libraries in the [awesome-noir repo here](https://github.com/noir-lang/awesome-noir#libraries).

Some libraries that are available today include:

- [Standard Library](https://github.com/noir-lang/noir/tree/master/noir_stdlib) - the Noir Standard Library
- [Ethereum Storage Proof Verification](https://github.com/aragonzkresearch/noir-trie-proofs) - a library that contains the primitives necessary for RLP decoding (in the form of look-up table construction) and Ethereum state and storage proof verification (or verification of any trie proof involving 32-byte long keys)
- [BigInt](https://github.com/shuklaayush/noir-bigint) - a library that provides a custom BigUint56 data type, allowing for computations on large unsigned integers
- [ECrecover](https://github.com/colinnielsen/ecrecover-noir/tree/main) - a library to verify an ECDSA signature and return the source Ethereum address
- [Sparse Merkle Tree Verifier](https://github.com/vocdoni/smtverifier-noir/tree/main) - a library for verification of sparse Merkle trees
- [Signed Int](https://github.com/resurgencelabs/signed_int) - a library for accessing a custom Signed Integer data type, allowing access to negative numbers on Noir
- [Fraction](https://github.com/resurgencelabs/fraction) - a library for accessing fractional number data type in Noir, allowing results that aren't whole numbers
