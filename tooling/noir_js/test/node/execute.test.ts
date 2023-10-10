import assert_lt_json from '../noir_compiled_examples/assert_lt/target/assert_lt.json' assert { type: 'json' };
import { Noir } from '@noir-lang/noir_js';
import { CompiledCircuit } from '@noir-lang/types';
import { expect } from 'chai';

const assert_lt_program = assert_lt_json as CompiledCircuit;

it('returns the return value of the circuit', async () => {
  const inputs = {
    x: '2',
    y: '3',
  };
  const { returnValue } = await new Noir(assert_lt_program).execute(inputs);

  expect(returnValue).to.be.eq('0x05');
});
