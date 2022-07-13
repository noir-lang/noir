# Asset Valuation

All dollar valuations seen in zk.money are derived with use of oracles provided by [Chainlink](https://chain.link/).

| Asset | Chainlink Oracle                                                                                     |
| ----- | ---------------------------------------------------------------------------------------------------- |
| ETH   | [ethereum/mainnet/crypto-usd/eth-usd](https://data.chain.link/ethereum/mainnet/crypto-usd/eth-usd)   |
| DAI   | [ethereum/mainnet/stablecoins/dai-usd](https://data.chain.link/ethereum/mainnet/stablecoins/dai-usd) |

## wstETH

Chainlink provides no direct oracle for wstETH, but does provide an [oracle for stETH](https://data.chain.link/ethereum/mainnet/crypto-usd/steth-usd). We use this stETH oracle in conjunction with the `getStETHByWstETH` method on the [Lido contract](https://etherscan.io/address/0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0#readContract) to calculate a wstETH dollar valuation.

## Element fixed yield positions

Over time we interpolate between the asset quantitiy locked in your position and its promised output. The interpolated quanity is then given a dollar valuation with use of Chainlink.
