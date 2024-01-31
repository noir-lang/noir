import assert_lt_json from '../noir_compiled_examples/assert_lt/target/assert_lt.json' assert { type: 'json' };
import assert_msg_json from '../noir_compiled_examples/assert_msg_runtime/target/assert_msg_runtime.json' assert { type: 'json' };
import { Noir } from '@noir-lang/noir_js';
import { CompiledCircuit } from '@noir-lang/types';
import { expect } from 'chai';

const assert_lt_program = assert_lt_json as CompiledCircuit;
const assert_msg_runtime = assert_msg_json as CompiledCircuit;

it('returns the return value of the circuit', async () => {
  const inputs = {
    x: '2',
    y: '3',
  };
  const { returnValue } = await new Noir(assert_lt_program).execute(inputs);

  expect(returnValue).to.be.eq('0x05');
});

it('circuit with a dynamic assert message should fail on an assert failure not the foreign call handler', async () => {
  const inputs = {
    x: '10',
    y: '5',
  };
  try {
    await new Noir(assert_msg_runtime).execute(inputs);
  } catch (error) {
    const knownError = error as Error;
    expect(knownError.message).to.equal('Circuit execution failed: Error: Cannot satisfy constraint');
  }
});
