# Building dapps

Please use the [TUTORIAL-TEMPLATE](./TUTORIAL_TEMPLATE.md) for standalone guides / tutorials.

Explain how to write a dapp using [`aztec.js`](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/aztec.js). Maybe that readme is enough?

- aztec.js
  - Docs outlining every typescript type.
    - Q: can we use the tsdocs, or do we need something with a human touch (i.e. with careful explanations sandwiching the auto-generated interface data?)
  - Docs outlining every external function:
    - Q: can we use the tsdocs, or do we need something with a human touch (i.e. with careful explanations sandwiching the auto-generated interface data?)
    - web3.js is nice docs.
    - ethers.js is nice docs.
    - Purpose of the function
    - Parameters
    - Return values
    - Example of the function being used in a wider context?
  - Walk-throughs / examples of using aztec.js for different use-cases
    - Hopefully we can pull large code snippets directly from the e2e tests for this.
    - We could even add way more comments to the e2e test files directly, and then use the include_code (see root README.md) to pull code snippets into the docs.
    - Use the e2e tests as inspiration.
      - Instantiate a contract
      - Deploy a contract
      - How to generate a nice typescript interface for an Aztec.nr contract's functions (we have a little `.ts` program in `noir-contracts` to generate a types file at the moment... how would a user do this?)
      - Simulate functions (simulate the result, without sending to the 'network')
      - Execute functions (send them to the 'network')
      - Tx hashes and tx receipts
      - How to query state
      - How to query whether a tx has been 'mined'
      - How to subscribe to logs
      - How to filter for historical data in the historical block tree?
      - How to query data from any of the trees (advanced)

FOR INSTRUCTIONS FOR BUILDING A WALLET, WE SHOULD WRITE DOCS HERE

ERRORS:

- Add any error explanations to [errors.md](../docs/developers/contracts/common_errors.md) (and break that file into multiple files if it's too unwieldy).

## Testing a dapp

Write about how to test a dapp using `aztec.js`. Maybe this overlaps with the e2e test stuff discussed above.
