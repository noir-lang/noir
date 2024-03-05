---
title: What a contract looks like
---

## Example Aztec.nr Contract

In keeping with the origins of blockchain, here's an example of a simple private token contract. Everyone's balances are private.

#include_code easy_private_token_contract /noir-projects/noir-contracts/contracts/easy_private_token_contract/src/main.nr rust

The prelude consists of more commonly imported aztec types that are needed for development. Here is what the prelude includes:

#include_code prelude /noir-projects/aztec-nr/aztec/src/prelude.nr rust

:::info Disclaimer
Please note that any example contract set out herein is provided solely for informational purposes only and does not constitute any inducement to use or deploy. Any implementation of any such contract with an interface or any other infrastructure should be used in accordance with applicable laws and regulations.
:::