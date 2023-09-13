# Testing contracts

Please use the [TUTORIAL-TEMPLATE](../../TUTORIAL_TEMPLATE.md) for standalone guides / tutorials.

## Testing in Aztec.nr

Individual functions can be tested much like [how 'regular Noir' functions can be tested](https://noir-lang.org/nargo/testing).

But an Aztec.nr contract typically tracks state variables, so you'll likely need to write more complex tests in TypeScript, using Aztec.js