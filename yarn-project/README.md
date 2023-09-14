# Aztec Typescript Packages

All the Typescript packages that make up [Aztec](https://docs.aztec.network/aztec3/overview).

## Development

All scripts are run in the `yarn-project` workspace root:

- Run `yarn` to install dependencies
- Run `yarn build:dev` to compile all packages on file changes
- Run `yarn prepare` to update `tsconfig.json` and `build_manifest.json` references
- Run `yarn format` to prettify all files
- Run `yarn formatting` to check prettier and eslint rules on each package (slow)

## Tests

To run tests for a specific package, just run `yarn test` in its folder. Note that `end-to-end` tests require `anvil` to be running, which is installed as part of the Foundry toolchain.

## Useful extensions

Consider installing the Prettier and ESLint extensions if using VSCode. Configure Prettier to format the code on save, and ensure that ESLint errors are shown in your IDE.

## Typescript config

- `yarn-project/tsconfig.json`: Base tsconfig file, extended by all packages. Used directly by vscode and eslint, where it functions to include the whole project without listing project references.
- `yarn-project/[package]/tsconfig.json`: Each package has its own file that specifies its project reference dependencies. This allows them to be built independently.

## Package.json inheritance

To simplify the management of all package.json files, we have a custom script that injects the contents of `package.common.json` into all packages that reference it via the `inherits` custom field. To run the script, just run `yarn prepare`. To override any of the fields from `package.common.json`, declare a `package.local.json` local to the package and add it to the `inherits` field.

## Adding a new package

To add a new package, make sure to add it to the `build_manifest.json`, to the `workspaces` entry in the root `package.json`, and to the `.circleci/config`. Then, copy the structure from another existing package, including:

- `.eslintrc.cjs`
- `Dockerfile`
- `package.json`
- `README.md`
- `tsconfig.json`

You may also need to modify the [Dockerfile](yarn-project/yarn-project-base/Dockerfile) to copy your new `package.json` into the container to get CI to pass.
