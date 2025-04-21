## Project Structure

This folder is organized into several key components:

- `acir/` - Definition and implementation of ACIR
- `acvm/` - Implementation of ACVM that executes ACIR
- `brillig/` - Definition and implementation of unconstrained Brillig opcodes
- `brillig_vm/` - Implementation of Brillig VM that executes Brillig

Click into each folder to learn more from their READMEs.

# How to add a new crate to the workspace

- Create the new crate with the current version of the other crates.
- In root `Cargo.toml`, add the new crate to the workspace members list.
- If you want to import it from multiple packages, you can add it as a dependency in the root `Cargo.toml`.
- In `release-please-config.json`:
  - Add a package entry
  - Add the crate name to the `linked-versions` plugin list
  - If you added the new crate as a dependency in the root `Cargo.toml`, add it to the extra-files of the root package.
- In `.release-please-manifest.json`, add the new crate with the same version of the others.
- In [publish.yml](.github/workflows/publish.yml), add the new crate to the `publish` job after its dependencies.