import assert_lt_json from '../noir_compiled_examples/assert_lt/target/assert_lt.json' assert { type: 'json' };
import assert_msg_json from '../noir_compiled_examples/assert_msg_runtime/target/assert_msg_runtime.json' assert { type: 'json' };
import fold_fibonacci_json from '../noir_compiled_examples/fold_fibonacci/target/fold_fibonacci.json' assert { type: 'json' };
import assert_raw_payload_json from '../noir_compiled_examples/assert_raw_payload/target/assert_raw_payload.json' assert { type: 'json' };

import { Noir, ErrorWithPayload } from '@noir-lang/noir_js';
import { CompiledCircuit } from '@noir-lang/types';
import { expect } from 'chai';

const assert_lt_program = assert_lt_json as CompiledCircuit;
const assert_msg_runtime = assert_msg_json as CompiledCircuit;
const fold_fibonacci_program = fold_fibonacci_json as CompiledCircuit;

it('executes a single-ACIR program correctly', async () => {
  const inputs = {
    x: '2',
    y: '3',
  };
  const { returnValue } = await new Noir(assert_lt_program).execute(inputs);

  expect(returnValue).to.be.eq('0x05');
});

it('circuit with a fmt string assert message should fail with the resolved assertion message', async () => {
  const inputs = {
    x: '10',
    y: '5',
  };
  try {
    await new Noir(assert_msg_runtime).execute(inputs);
  } catch (error) {
    const knownError = error as Error;
    expect(knownError.message).to.equal('Circuit execution failed: Expected x < y but got 10 < 5');
  }
});

it('circuit with a raw assert payload should fail with the decoded payload', async () => {
  const inputs = {
    x: '7',
    y: '5',
  };
  try {
    await new Noir(assert_raw_payload_json).execute(inputs);
  } catch (error) {
    const knownError = error as ErrorWithPayload;
    const invalidXYErrorSelector = Object.keys(assert_raw_payload_json.abi.error_types)[0];
    expect(knownError.rawAssertionPayload!.selector).to.equal(invalidXYErrorSelector);
    expect(knownError.decodedAssertionPayload).to.deep.equal({
      x: '0x07',
      y: '0x05',
    });
  }
});

it('successfully executes a program with multiple acir circuits', async () => {
  const inputs = {
    x: '10',
  };
  try {
    await new Noir(fold_fibonacci_program).execute(inputs);
  } catch (error) {
    const knownError = error as Error;
    expect(knownError.message).to.equal('Circuit execution failed: Expected x < y but got 10 < 5');
  }
});

it('circuit with a raw assert payload should fail with the decoded payload', async () => {
  const inputs = {
    x: '7',
    y: '5',
  };
  try {
    await new Noir(assert_raw_payload_json).execute(inputs);
  } catch (error) {
    const knownError = error as ErrorWithPayload;
    const invalidXYErrorSelector = Object.keys(assert_raw_payload_json.abi.error_types)[0];
    expect(knownError.rawAssertionPayload!.selector).to.equal(invalidXYErrorSelector);
    expect(knownError.decodedAssertionPayload).to.deep.equal({
      x: '0x07',
      y: '0x05',
    });
  }
});

it('successfully executes a program with multiple acir circuits', async () => {
  const inputs = {
    x: '10',
  };
  try {
    await new Noir(fold_fibonacci_program).execute(inputs);
  } catch (error) {
    const knownError = error as Error;
    expect(knownError.message).to.equal('Circuit execution failed: Error: Cannot satisfy constraint');
  }
});

it('successfully executes a program with multiple acir circuits', async () => {
  const inputs = {
    x: '10',
  };
  try {
    await new Noir(fold_fibonacci_program).execute(inputs);
  } catch (error) {
    const knownError = error as Error;
    expect(knownError.message).to.equal('Circuit execution failed: Error: Cannot satisfy constraint');
  }
});

it('successfully executes a program with multiple acir circuits', async () => {
  const inputs = {
    x: '10',
  };
  try {
    await new Noir(fold_fibonacci_program).execute(inputs);
  } catch (error) {
    const knownError = error as Error;
    expect(knownError.message).to.equal('Circuit execution failed: Error: Cannot satisfy constraint');
  }
});

it('successfully executes a program with multiple acir circuits', async () => {
  const inputs = {
    x: '10',
  };
  try {
    await new Noir(fold_fibonacci_program).execute(inputs);
  } catch (error) {
    const knownError = error as Error;
    expect(knownError.message).to.equal('Circuit execution failed: Error: Cannot satisfy constraint');
  }
});
