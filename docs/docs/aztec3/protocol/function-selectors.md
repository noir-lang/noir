---
title: Function Selectors
---

:::caution
We are building Aztec 3 as transparently as we can. The documents published here are merely an entry point to understanding. These documents are largely complete, but unpolished.

If you would like to help us build Aztec 3, consider reviewing our [GitHub](https://github.com/AztecProtocol) to contribute code and joining our [forum](https://discourse.aztec.network/) to participate in discussions.
:::

## Why are function selectors necessary in the Function Tree?

Full question: **"Why does a function leaf need to include the function selector versus just the vkHash and isPrivate?"**

### Answer

Imagine you’re writing contract A and it imports a contract B which adheres to an ERC20 interface (for example). You don’t care about the body of the contract you import; in fact you might want any contract with that interface to be callable from contract A and each such contract might have a slightly different implementation, so the VKHash of all those potential contracts would be different. So you just care about the correctness of the function signature being imported. The function signature is enough info for Noir++ to generate constraints which represent a ‘call’ to any implementation of contract B.

We include the function signature in the contract tree, so that the kernel circuit can make sure a function with this signature was used under these circumstances.

Uniswap is an example of contract A. It allows any two erc20 contracts to swap, without caring about their implementation bodies.

We use the first 4 bytes of the hash of the function signature, just to copy ethereum, really. Although the hash will be something snark-friendly instead of keccak. I suppose the 4 byte truncation (vs the full 32bytes of the hash) reduces communication bandwidth when interacting with L1? Maybe it’s unnecessary. We don’t go to fewer than 4 bytes because then we’d be highly likely to have collisions between different functions of the same contract (it’s actually a general case of the birthday problem).

### Leaning into the Uniswap example

Contract A can import some interface like ERC20, and enable calls to the "`ERC20.transfer`" function of any other contract (B or C...) that implements that interface. B and C might have different implementations of `ERC20.transfer`, so their VKHashes will be different. All that matters to A is that the function being called has the proper signature. This function signature (`transfer(address,uint256)`) is enough information for Noir++ to generate constraints which represent a 'call' to any implementation of `ERC20.transfer`.

Uniswap is an example of contract A. For `swap`, it allows calls to any ERC20 contract's transfer function. The function's signature just needs to match "`transfer(address,uint256)`".

Here are links to the UniswapV2Pair's relevant code:

- [A `SELECTOR` constant containing the ERC20 `transfer` function's signature](https://github.com/Uniswap/v2-core/blob/master/contracts/UniswapV2Pair.sol#L16)
- [`swap` function](https://github.com/Uniswap/v2-core/blob/master/contracts/UniswapV2Pair.sol#L159)
- [`swap` function calls a helper (`_safeTransfer`) to transfer whichever ERC20 token is being transferred out of the liquidity pool to the swapper](https://github.com/Uniswap/v2-core/blob/master/contracts/UniswapV2Pair.sol#L170)
- [`_safeTransfer` makes a `call` to the ERC20 token contract using the `SELECTOR` function signature](https://github.com/Uniswap/v2-core/blob/master/contracts/UniswapV2Pair.sol#L44)

[Here](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/IERC20.sol#L41) is the `transfer` function as defined in the ERC20 interface.

And finally, [here](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/token/ERC20/ERC20.sol#L113) is an example implementation of the ERC20 `transfer` function.

## Participate

Keep up with the latest discussion and join the conversation in the [Aztec forum](https://discourse.aztec.network).
