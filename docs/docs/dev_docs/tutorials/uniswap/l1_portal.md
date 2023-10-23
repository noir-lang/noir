---
title: Uniswap Portal on L1
---

In this step we will set up our Solidity portal contract.

In `l1-tokens` create a new file called `UniswapPortal.sol`

```sh
cd l1-contracts/contracts && touch UniswapPortal.sol
```

and paste this inside:

```solidity
pragma solidity ^0.8.20;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {IRegistry} from "@aztec/l1-contracts/src/core/interfaces/messagebridge/IRegistry.sol";
import {DataStructures} from "@aztec/l1-contracts/src/core/libraries/DataStructures.sol";
import {Hash} from "@aztec/l1-contracts/src/core/libraries/Hash.sol";

#include_code setup l1-contracts/test/portals/UniswapPortal.sol raw
```

In this set up we defined the `initialize()` function and a struct (`LocalSwapVars`) to manage assets being swapped.

Like we saw in the [TokenPortal](../token_portal/depositing_to_aztec.md), we initialize this portal with the registry contract address (to fetch the appropriate inbox and outbox) and the portal’s sister contract address on L2.

In the next step we will set up the appropriate L2 Uniswap contract!
