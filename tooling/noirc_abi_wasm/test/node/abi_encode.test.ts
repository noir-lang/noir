import { expect } from 'chai';
import { abiEncode, abiDecode, WitnessMap, Field } from '@noir-lang/noirc_abi';
import type { DecodedInputs } from '../types';
import { abi, inputs } from '../shared/abi_encode';

it('recovers original inputs when abi encoding and decoding', async () => {
  const initial_witness: WitnessMap = abiEncode(abi, inputs);
  const decoded_inputs: DecodedInputs = abiDecode(abi, initial_witness);

  const foo: Field = inputs.foo as Field;
  const bar: Field[] = inputs.bar as Field[];
  expect(BigInt(decoded_inputs.inputs.foo)).to.be.equal(BigInt(foo));
  expect(parseInt(decoded_inputs.inputs.bar[0])).to.be.equal(parseInt(bar[0].toString()));
  expect(parseInt(decoded_inputs.inputs.bar[1])).to.be.equal(parseInt(bar[1].toString()));
  expect(parseInt(decoded_inputs.inputs.bar[2])).to.be.equal(parseInt(bar[2].toString()));
  expect(decoded_inputs.return_value).to.be.null;
});
