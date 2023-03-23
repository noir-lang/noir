# Archiver
To run:
1. Run `anvil`,
2. in the aztec3-l1-contracts repo check out my branch `janb/archiver-test-data`,
3. deploy the contracts and generate initial activity with: `forge script --fork-url "http://127.0.0.1:8545/" --ffi GenerateActivityTest --sig "testGenerateActivity()" --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 --broadcast`
4. in this repository run `yarn start:dev` (Note: this repo is currently messy and eslint will not allow it to be built with `yarn start`)
