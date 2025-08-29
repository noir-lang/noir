import assert_lt_json from '../noir_compiled_examples/assert_lt/target/assert_lt.json' assert { type: 'json' };
import assert_msg_json from '../noir_compiled_examples/assert_msg_runtime/target/assert_msg_runtime.json' assert { type: 'json' };
import fold_fibonacci_json from '../noir_compiled_examples/fold_fibonacci/target/fold_fibonacci.json' assert { type: 'json' };
import assert_raw_payload_json from '../noir_compiled_examples/assert_raw_payload/target/assert_raw_payload.json' assert { type: 'json' };
import databus_json from '../noir_compiled_examples/databus/target/databus.json' assert { type: 'json' };
import assert_inside_brillig_nested_json from '../noir_compiled_examples/assert_inside_brillig_nested/target/assert_inside_brillig_nested.json' assert { type: 'json' };

import { Noir, ErrorWithPayload } from '@noir-lang/noir_js';
import { CompiledCircuit } from '@noir-lang/types';
import { expect } from 'chai';

const assert_lt_program = assert_lt_json as CompiledCircuit;
const assert_msg_runtime = assert_msg_json as CompiledCircuit;
const fold_fibonacci_program = fold_fibonacci_json as CompiledCircuit;
const assert_raw_payload = assert_raw_payload_json as CompiledCircuit;
const databus_program = databus_json as CompiledCircuit;
const assert_inside_brillig_nested = assert_inside_brillig_nested_json as CompiledCircuit;

it('executes a single-ACIR program correctly', async () => {
  const inputs = {
    x: '2',
    y: '3',
  };
  const { returnValue } = await new Noir(assert_lt_program).execute(inputs);

  expect(returnValue).to.be.eq('0x05');
});

it('successfully executes a program with multiple acir circuits', async () => {
  const inputs = {
    x: '10',
  };
  expect(() => new Noir(fold_fibonacci_program).execute(inputs)).to.not.throw();
});

it('circuit with a fmt string assert message should fail with the resolved assertion information', async () => {
  const inputs = {
    x: '10',
    y: '5',
  };
  try {
    await new Noir(assert_msg_runtime).execute(inputs);
  } catch (error) {
    const knownError = error as ErrorWithPayload;
    expect(knownError.message).to.equal('Circuit execution failed: Expected x < y but got 10 < 5');
  }
});

it('circuit with a nested assertion should fail with the resolved call stack', async () => {
  try {
    await new Noir(assert_msg_runtime).execute({
      x: '10',
      y: '5',
    });
  } catch (error) {
    const knownError = error as ErrorWithPayload;
    expect(knownError.noirCallStack).to.have.lengthOf(2);
    expect(knownError.noirCallStack![0]).to.match(
      /^at make_assertion\(x, y\) \(.*assert_msg_runtime\/src\/main.nr:2:5\)$/,
    );
    expect(knownError.noirCallStack![1]).to.match(/^at x < y \(.*assert_msg_runtime\/src\/main.nr:7:12\)$/);
  }
});

it('circuit with a nested assertion inside brillig should fail with the resolved call stack', async () => {
  try {
    await new Noir(assert_inside_brillig_nested).execute({
      x: '10',
    });
  } catch (error) {
    const knownError = error as ErrorWithPayload;
    const expectedStack = ['acir_wrapper(x)', 'brillig_entrypoint(x)', 'brillig_nested(x)', 'x < 10'];
    expect(knownError.noirCallStack).to.have.lengthOf(expectedStack.length);

    for (let i = 0; i < expectedStack.length; i++) {
      expect(knownError.noirCallStack![i]).to.contain(expectedStack[i]);
    }
  }
});

it('circuit with a raw assert payload should fail with the decoded payload', async () => {
  const inputs = {
    x: '7',
    y: '5',
  };
  try {
    await new Noir(assert_raw_payload).execute(inputs);
  } catch (error) {
    console.log(error);
    const knownError = error as ErrorWithPayload;
    const invalidXYErrorSelector = Object.keys(assert_raw_payload_json.abi.error_types)[0];
    expect(knownError.rawAssertionPayload!.selector).to.equal(invalidXYErrorSelector);
    expect(knownError.decodedAssertionPayload).to.deep.equal({
      x: '0x07',
      y: '0x05',
    });
  }
});

it('successfully decodes the return values from a program using the databus', async () => {
  const inputs = {
    x: '3',
    y: '4',
    z: [1, 2, 3, 4],
  };
  const { returnValue } = await new Noir(databus_program).execute(inputs);
  expect(returnValue).to.be.eq('0x09');
});
