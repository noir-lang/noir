## Project Structure

This folder is organized into several key components:

- `acir/` - Definition and implementation of ACIR
- `acvm/` - Implementation of ACVM that executes ACIR
- `brillig/` - Definition and implementation of unconstrained Brillig opcodes
- `brillig_vm/` - Implementation of Brillig VM that executes Brillig

Click into each folder to learn more from their READMEs.

## Development

### Adding a New Crate

To add a new crate to the workspace:

1. Create the new crate with the current version of other crates
2. In root `Cargo.toml`, add the new crate to the workspace members list
3. If you want to import it in other noir-lang packages, add it as a dependency in the root `Cargo.toml`
4. Update `release-please-config.json`:
   - Add a package entry
   - Add the crate name to the `linked-versions` plugin list
   - If added as a dependency in the root `Cargo.toml`, add it to the extra-files of the root package
5. Update `.release-please-manifest.json` with the new crate at the same versioning number as others
6. Update [publish.yml](../.github/workflows/publish-acvm.yml) to include the new crate in the `publish` job