/* eslint-disable no-undef */
/* eslint-disable @typescript-eslint/no-var-requires */
const chai = require('chai');
const assert_lt_json = require('../noir_compiled_examples/assert_lt/target/assert_lt.json');
const { Noir } = require('@noir-lang/noir_js');

it('generates witnesses successfully', async () => {
  const inputs = {
    x: '2',
    y: '3',
  };
  const _solvedWitness = await new Noir(assert_lt_json).execute(inputs);
});

it('string input and number input are the same', async () => {
  const inputsString = {
    x: '2',
    y: '3',
  };
  const inputsNumber = {
    x: 2,
    y: 3,
  };
  const solvedWitnessString = await new Noir(assert_lt_json).execute(inputsString);
  const solvedWitnessNumber = await new Noir(assert_lt_json).execute(inputsNumber);
  chai.expect(solvedWitnessString).to.deep.equal(solvedWitnessNumber);
});

it('string input and number input are the same', async () => {
  const inputsString = {
    x: '2',
    y: '3',
  };
  const inputsNumber = {
    x: 2,
    y: 3,
  };

  const solvedWitnessString = await new Noir(assert_lt_json).execute(inputsString);
  const solvedWitnessNumber = await new Noir(assert_lt_json).execute(inputsNumber);
  chai.expect(solvedWitnessString).to.deep.equal(solvedWitnessNumber);
});

it('0x prefixed string input for inputs will throw', async () => {
  const inputsHexPrefix = {
    x: '0x2',
    y: '0x3',
  };

  try {
    await new Noir(assert_lt_json).execute(inputsHexPrefix);
    chai.expect.fail(
      'Expected generatedWitness to throw, due to inputs being prefixed with 0x. Currently not supported',
    );
  } catch (error) {
    // Successfully errored due to 0x not being supported. Update this test once/if we choose
    // to support 0x prefixed inputs.
  }
});

describe('input validation', () => {
  it('x should be a uint64 not a string', async () => {
    const inputs = {
      x: 'foo',
      y: '3',
    };

    try {
      await new Noir(assert_lt_json).execute(inputs);
      chai.expect.fail('Expected generatedWitness to throw, due to x not being convertible to a uint64');
    } catch (error) {
      const knownError = error;
      chai
        .expect(knownError.message)
        .to.equal(
          'The value passed for parameter `x` is invalid:\nExpected witness values to be integers, provided value causes `invalid digit found in string` error',
        );
    }
  });
});
