---
title: Subsidy Module
---

Aztec Connect has a [subsidy module](https://etherscan.io/address/0xABc30E831B5Cc173A9Ed5941714A7845c909e7fA) that strives to improve consistency of bridge executions, e.g., how often they are executed. This is done by allowing anyone to setup a subsidy which is paying some amount to the sequencer when executing the bridge. It is essentially a way to make it profitable for the sequencer to include the bridge, even if the users that are queued don't cover the gas entirely themselves. 

The module is primary expected to be used by projects/teams that are added to Aztec who want to give their users a more consistent experience. To give you an example, Yearn subsidises deposits into the Eth and Dai bridges, reducing the time users have to wait until they have their yvEth and yvDai.

The module itself is an immutable contract that have no admin keys, allowing anyone to subsidise the bridges. Prior to the module, teams wanting to subsidise bridges would have to send some Eth to the Aztec team and we would manually setup the subsidy within our sequencer software. This was impractical since it required our cooperation.

The idea is relatively simple, the bridge has an upper bound on the gas it consumes. You never want to subsidise more than this amount per "run", as you are then wasting funds, e.g., upper bound of subsidy = upper bound of gas consumed by the bridge. At the same time, you have an idea about how often the bridge should be executed at a minimum, so you let the subsidy grow over this time period. Written as a formula `bridge_gas_cost / time_between_executions = gas_per_time_unit`. In the case of Yearn, they wanted daily execution, so they needed to cover `200K / 24 hours = 140 gas per minute` (200k gas is how much the Yearn bridge call costs - this value is bridge specific). After 24 hours, the subsidy reaches its maximum amount. In such a scenario even one user fee would be enough to make it profitable for the sequencer to include the bridge call in a rollup block. This is because the fee paid by user + the subsidy is bigger than the actual gas cost of the bridge call. When the subsidy is claimed, the amount goes back to 0, and the cycle repeats. 

On the module, the subsidy is accruing to a specific `bridge` and `criteria`. Criteria is an integer which can be used by the bridge to differentiate between different flows. This valuable because different execution flow might have a different gas cost and because it's common that people want to subsidize only a specific flow (for example Yearn wanted to subsidize only deposits to their vaults and not exits). The `criteria` value is being computed by the bridge itself and it is registered by the bridge in the Subsidy contract. This is usually done in the bridge's constructor. Because the subsidy is accruing to a criteria, multiple flows that all have the same criteria will be "fighting" over it. Something you need to consider when building the bridge. 


---
# What to keep in mind while building
When building a bridge, the bridge needs to have a way of calling the `setGasUsageAndMinGasPerMinute()` function of the subsidy module. This function is used to specify the bounds and criteria that should be able to receive a subsidy from the module. We are performing this "registration" from the bridge itself as it is a way of getting rid of any trusted party, no admin keys. The `_minGasPerMinute` that is passed is primarily used to ensure that someone cannot just come in to subsidise a tiny tiny amount over time, effectively giving no subsidy.

Thereafter, you need to decide how your bridge is to compute the criteria values. If your bridge is limited in the actions it can take, e.g., the Yearn bridge only deposits/withdraws into/from yearn, it might be useful to have simple criteria that are shared between different flows, as it is then straightforward to subsidise multiple of them at once (in this case Yearn subsidised all the deposits, regardless of asset). If the bridge is more general purpose (such as ERC4626) it is beneficial to allow subsidising based on the combination of input and output assets. This can be done by computing a hash of input/output token addresses and converting that value to `uint256` and should be implemented in the `computeCriteria()` function.

With setup and criteria managed, you just need to make sure that the `convert()` function claims the subsidy through the `claimSubsidy()` function, which takes the criteria and the rollup beneficiary (address passed by sequencer). Without this function, no subsidy is given. 

# How do I subsidise?
When you have found a bridge to subsidise, you need to figure out what flows you want to subsidise (the criteria). This criteria will depend on the bridge and the best way to figure out the meaning of different criteria is to simply check the bridge's code. 
For bridges that follow the base implementation, there will be a `computeCriteria()` function taking assets and auxdata as input that should compute the criteria for a given interaction. *Note*: multiple flows can have the same criteria.


To give you an example our [Yearn bridge](https://github.com/AztecProtocol/aztec-connect-bridges/blob/master/src/bridges/yearn/YearnBridge.sol) uses criteria to distinguish between deposit and withdrawal flows while our [Uniswap bridge](https://github.com/AztecProtocol/aztec-connect-bridges/blob/master/src/bridges/uniswap/UniswapBridge.sol)  a specific combination of input/output assets.

To fund the subsidy using etherscan as an interface do the following:

1. Go to the contract's `subsidize(...)` function on [Etherscan](https://etherscan.io/address/0xabc30e831b5cc173a9ed5941714a7845c909e7fa#writeContract#F5)
2. Connect your wallet.
3. Fill in the following details:

  - **payableAmount** – This is the amount of ETH you want to commit to the subsidy (in Wei). We recommend initially setting this value relatively low (somewhere between 20-40% of the total grant size) because it’s possible you would want to change the Subsidy parameters below, and it is not possible to do that before current subsidy runs out. The minimum acceptable value is set to 0.1 ETH.
  - **\_bridge** – This is the address of the bridge you are subsidizing.
  - **\_criteria** – This is the parameter specifying which flow you want to subsidize.
  - **\_gasPerMinute** – This is the parameter for declaring how much gas per minute you are willing to subsidize. This value defines the rate at which the subsidy is released. The minimum acceptable value is bridge specific. We usually set **\_gasPerMinute** value such that a call is fully subsidized once per day. Fully subsidized would mean that the user’s base fee would get covered in full, and users would only have to pay priority fee tips

4. Send the transaction by clicking “Write”.

> Note: This is a standard contract interaction so feel free to fund the subsidy any other way.
> If you are using a multi-sig, you can instead propose this transaction in the UI of the multi-sig wallet, with the same parameters.