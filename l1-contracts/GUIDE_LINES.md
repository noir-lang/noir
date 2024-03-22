# Smart Contract Guideline
In the following, there will be guidelines for the process of writing readable, maintainable and secure contract code. We try to cover the entire process: 
- before writing the code, 
- while writing, 
- when optimizing
- when reviewing. 

In general the language of choice will be Solidity for our L1 contracts, so most of the specific style rules will be having that in mind, but the process for writing it can be adapted slightly for vyper, and will have an extension when Aztec.nr contracts are more mature. 

![](https://media.tenor.com/ry_sCXk6wH0AAAAC/pirates-caribbean-code.gif)

## Outline
When you plan to build a system, or write a contract to extend an existing one. This should be the go-to approach.
1. Write a specification. 
    - Define how the system is planned to work, what trade-offs are made and why. Do this without writing an implementation! The spec should have a "TL;DR" or similar so people can quickly skim to see what you want to do and why.
2. Write a reference implementation of the spec, focus on readability
    - Solidity contracts especially can be a pain in the neck to read if an implementation is optimized heavily.
3. Write extensive test for individual functions, integrations and scenarios
4. Add fuzz-testing when the bases are covered
5. Review reference implementation
6. Fix any issues that have been revealed by the review 
7. Create an optimized implementation

# 1. Writing a spec
Writing a spec can feel wasteful because it "slows" development down, but be really really practical when:
- Other people have to figure out what your code is supposed to do;
- Checking that the underpinning ideas of the implementation is sound.

For the spec, having a description of the data structures to very helpful to understand the system - don't think about storage layout and gas-costs for the initial design.

Secondly, we want a rather specific (but not to the level of implementation) description of the actions (functions) that can be executed on the "model". For any action, we ideally want to specify:
- **Name**: A `name` and inputs for the action, e.g., `transfer(from, to, amount)`
- **Preconditions**: A set of preconditions that must be satisfied for the action to execute successfully, e.g., `balance[from] >= amount`.
- **Effects** A set of state updates, e.g., `balance[from] -= amount  && balance[to] += amount`.

The spec should outline the dependencies of the contract (other protocols and tokens) and how they work. In the case of tokens for accounting purposes this includes listing the ways in which the balance can be altered (e.g., transfer, burn, mint, rebase, freeze, liquidate, etc). This is an important step, as it makes it a lot easier to:
1. Assess the risk of the extension/protocol.
2. Makes it more clear what types of actions the accounting needs to support, and if any are missing.

## 2. Writing a reference implementation
A reference implementation is a practical instantiation of the spec that should strive to maximize readability. 
It should make it very clear what the purpose is, and how it achieves its goal, meaning that no heavy optimizations should be made. The reference is simply to be used as a good way to understand the code, and to build tests for. To make assessing the coverage of the code easier, ternary operators should be avoid, e.g., instead of `a > b ? a : b` please use explicit `if-else` branching.
Having a reference implementation, that can later be optimized, should also make it easier for auditors and external parties to give a first look at the protocol, e.g., if there is a flaw in the business logic, it should be possible to catch at this point.

## 3. Writing tests
Test to cover all the user intended flows should be written, as well as tests for failure cases and less intended flows. 

To make the test useful for reviewers, a small comment describing it in easily understandable language should  be added.

When the tests have been written, coverage should be ~100%. If not, put focus on the parts that are missing, is there some possible behavior that we have not thought about or what is the reasoning behind the missing coverage. 

### 3.1 Happy Paths
A happy path is a series of actions that we expect users to do, or be part of, e.g., users using the protocol as expected.

### 3.2 Unhappy Paths
These tests are to handle the cases where someone deliberately are trying to mess with the accounting or extract funds.

### 3.3 Fail cases
This is tests that should have us end with a revert, any revert should have a tests where they will be hit.

## 4. Writing fuzz-tests
For actions with user-controlled input amounts or where inner values depend on timing, property based testing (fuzzing) should be used to change time (between actions) and the input values. Remember to limit values to something that is possible, e.g., if the total supply of a token is fixed, values beyond the supply should not be accepted.

## 5. Review reference implementation
For every issue found, create a `github` issue. These should be grouped in one of the following groups `informational, low, medium, high`. Issues should be separate, but if the same problem have been spotted multiple places, e.g., off-by one for both deposit and withdraw, it can be grouped into one `issue`.

When reviewing, consult the references to have checklists to go through. 

Also, is the contract planned to be used behind a proxy or directly? If behind a proxy, is there are deployment script that will initialize it, and does the upgrade mechanism work or is something missing.

## 6. Handle issues
For each of the `github` issue that have been created by the review, either a PR should be made that addresses the issue or it should be acknowledged. 

## 7. Create optimized implementation
Before writing the optimized implementation, the tests should be extended to the point where no test is lacking. Also, all tests should pass for both sets.

### 7.1 Good places to start
- Variable packing
- Cached variables (if read from storage multiple times, cache the value and use it)
- Unchecked math for increment in values where it is known that it will never overflow. 

### 7.2 Gas-comparison for reference / optimized
For each of the changes that are made, please add an estimate on the gas savings. If gas is saved and it doesn't significantly increase the code-size or heavily degrades the readability 

## Reviews
There is a couple of different angles when reviewing smart contracts:
- **Correctness**: Checking correctness of the implementation, e.g., does the implementation matches the specification, and is the specification sound?
- **Attack vectors**, e.g., how can an attacker gain access or influence the system in an unintended way.
- **Gas Optimizations**

## Good References
- [solcurity](https://github.com/Rari-Capital/solcurity)
- [ToB building secure contracts](https://github.com/crytic/building-secure-contracts/blob/master/development-guidelines/workflow.md)
- [Nascent security toolkit](https://github.com/nascentxyz/simple-security-toolkit)

## Tools
- [Slither](https://github.com/crytic/slither)
- [Echidna](https://github.com/crytic/echidna)
- [Manticore](https://github.com/trailofbits/manticore)
- [Foundry](https://github.com/foundry-rs/foundry)
- [Pyrometer](https://github.com/nascentxyz/pyrometer)

## Solidity Style Guide
We generally strive to adhere to the [soliditylang guidelines](https://docs.soliditylang.org/en/latest/style-guide.html) with a few extension outlined below:

- Any imports should be fully specified, e.g., `import { IERC20 } from "./IERC20.sol"` over `import "./IERC20.sol"`.
- When using `uint` always be explicit with the size, e.g., `uint256` or `uint128` or whatever
- Beyond the ordering of functions, we use error codes, events, struct, storage, modifiers and then the functions. Functions again sorted as by the soliditylang guidelines.
- For `if` statements, we should *always* add brackets `{}` even if it may be on a single line.
- When overriding, always specify what is overridden, e.g., `override(IRollupProcessor)`
- `_` prefix on internal/private function names (unless if in library)
- Don't prefix with `_` on internal/private storage variables
- `_` prefix on function arguments
- Push people to use Custom Errors instead of requires.
- `constant` and `immutable` values should be written in `CAPITAL_CASE`

### Natspec

Natspec should be written for all functions (`internal` mainly for clarity). Use the `@notice` tag for general explanation of the function that a user should be able to understand and `@dev` for more developer specific information. 

```solidity
  /**
   * @notice Inserts a new message into the Inbox
   * @dev Emits `MessageSent` with data for easy access by the sequencer
   * @param _recipient - The recipient of the message
   * @param _content - The content of the message (application specific)
   * @param _secretHash - The secret hash of the message (make it possible to hide when a specific message is consumed on L2)
   * @return Hash of the sent message.
   */
  function sendL2Message(
    DataStructures.L2Actor memory _recipient,
    bytes32 _content,
    bytes32 _secretHash
  ) external override(IInbox) returns (bytes32) {
```

### Solhint configuration

Many of these guidelines are enforced by solhint, with additions in https://github.com/LHerskind/solhint. 

```json
{
  "extends": "solhint:recommended",
  "rules": {
    "compiler-version": ["error", ">=0.8.4"],
    "no-inline-assembly": "off",
    "func-visibility": ["error", { "ignoreConstructors": true }],
    "no-empty-blocks": "off",
    "no-unused-vars": ["error"],
    "state-visibility": ["error"],
    "not-rely-on-time": "off",
    "const-name-snakecase": ["error", { "treatImmutableVarAsConstant": true }],
    "var-name-mixedcase": ["error", { "treatImmutableVarAsConstant": true }],
    "custom-error-name-camelcase": ["error", { "allowPrefix": true }],
    "private-func-leading-underscore": ["error"],
    "private-vars-no-leading-underscore": ["error"],
    "func-param-name-leading-underscore": ["error"],
    "func-param-name-mixedcase": ["error"],
    "custom-error-over-require": ["error"],
    "strict-override": ["error"],
    "strict-import": ["error"],
    "ordering": ["error"],
    "comprehensive-interface": ["error"]
  }
}
```