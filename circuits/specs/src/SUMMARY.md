# Summary

[Introduction](intro.md)

# Contracts

- [Overview](./architecture/contracts/contracts.md)
- [Trees](./architecture/contracts/trees.md)
- [Function Types](./architecture/contracts/function-types.md)
- [States & Storage](./architecture/contracts/states-and-storage.md)
- [Contract Deployment](./architecture/contracts/deployment.md)
- [Transactions (Calls)](./architecture/contracts/transactions.md)
- [L1 Calls](architecture/contracts/l1-calls.md)

# Fees

- [Fees](./architecture/fees/fees.md)

# App-specific Circuits

- [Public Input ABIs](./architecture/app-circuits/public-input-abis.md)

# Kernel Circuits

- [Intro](./architecture/kernel-circuits/kernel-circuits.md)
- [Private Kernel Circuit](./architecture/kernel-circuits/private-kernel.md)
- [Public Kernel Circuit](./architecture/kernel-circuits/public-kernel.md)
- [Contract Deployment Kernel Circuit](./architecture/kernel-circuits/contract-deployment-kernel.md)

# Rollup Circuits

- [Intro](./architecture/rollup-circuits/rollup-circuits.md)
- [Base Rollup Circuit](./architecture/rollup-circuits/base-rollup.md)
- [Merge Rollup Circuit](./architecture/rollup-circuits/merge-rollup.md)
- [L1 Results Copier Circuit](./architecture/rollup-circuits/l1-results-copier.md)
- [Standard Plonk Converter Circuit](./architecture/rollup-circuits/standard-plonk-converter.md)

# Examples

- [ERC20 Shielding](./examples/erc20/erc20-shielding.md)
  - [Deployment](./examples/erc20/deployment.md)
  - [Deposit](./examples/erc20/deposit.md)
  - [Transfer](./examples/erc20/transfer.md)
  - [Withdraw](./examples/erc20/withdraw.md)
  - [Appendix](./examples/erc20/appendix/appendix.md)
    - [RollupProcessor Contract](./examples/erc20/appendix/rollup-processor.md)
    - [Portal Contract](./examples/erc20/appendix/portal-contract.md)

# Noir Stuff
- [Intro](./noir-stuff/noir-stuff.md)
- [Extremes](./noir-stuff/extremes/extremes.md)
  - [incrementation of private state _not_ owned by caller](./noir-stuff/extremes/incr-private-not-owned.md)
  - [incrementation of private state owned by the caller](./noir-stuff/extremes/incr-private-owned.md)
  - [decrementation of private state owned by the caller](./noir-stuff/extremes/decr-private-owned.md)
  - [editing a public state](./noir-stuff/extremes/edit-public.md)
  - [reading a public state](./noir-stuff/extremes/read-public.md)
  - [calling a private function of a different contract](./noir-stuff/extremes/call-private.md)
  - [calling a public function of a different contract](./noir-stuff/extremes/call-public.md)
  - [calling a public function from a private function of the same contract](./noir-stuff/extremes/call-public-same.md)
  - [calling an L1 function](./noir-stuff/extremes/call-l1.md)
  - [emitting an 'event'](./noir-stuff/extremes/emit-event.md)
  - [executing a function as a callback, following an L1 result](./noir-stuff/extremes/callback.md)
  - [deploying a contract from a private/public function](./noir-stuff/extremes/deploy-contract.md)
  - [new contract constructors]()
  - ['globally' available variables](./noir-stuff/extremes/global-vars.md)
  - [Aztec Connect claim example](./noir-stuff/extremes/claim-simple.md)

# Open Questions

- [Questions](./QUESTIONS.md)