import { expect } from 'chai';
import assert_lt_json from '../noir_compiled_examples/assert_lt/target/assert_lt.json' assert { type: 'json' };
import { generateWitness } from '../../src/index.js';
import { CompiledCircuit } from '@noir-lang/types';

const assert_lt_program = assert_lt_json as CompiledCircuit;

it('generates witnesses successfully', async () => {
  const inputs = {
    x: '2',
    y: '3',
  };
  expect(() => generateWitness(assert_lt_program, inputs)).to.not.throw;
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
  const solvedWitnessString = await generateWitness(assert_lt_program, inputsString);
  const solvedWitnessNumber = await generateWitness(assert_lt_program, inputsNumber);
  expect(solvedWitnessString).to.deep.equal(solvedWitnessNumber);
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

  const solvedWitnessString = await generateWitness(assert_lt_program, inputsString);
  const solvedWitnessNumber = await generateWitness(assert_lt_program, inputsNumber);
  expect(solvedWitnessString).to.deep.equal(solvedWitnessNumber);
});

it('0x prefixed string input for inputs will throw', async () => {
  const inputsHexPrefix = {
    x: '0x2',
    y: '0x3',
  };

  try {
    await generateWitness(assert_lt_program, inputsHexPrefix);
    expect.fail('Expected generatedWitness to throw, due to inputs being prefixed with 0x. Currently not supported');
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
      await generateWitness(assert_lt_program, inputs);
      expect.fail('Expected generatedWitness to throw, due to x not being convertible to a uint64');
    } catch (error) {
      const knownError = error as Error;
      expect(knownError.message).to.equal(
        'Expected witness values to be integers, provided value causes `invalid digit found in string` error',
      );
    }
  });
});
