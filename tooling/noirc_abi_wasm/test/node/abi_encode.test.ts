import { expect } from 'chai';
import { abiEncode, abiDecode, WitnessMap, Field } from '@noir-lang/noirc_abi';
import { DecodedInputs } from '../types';

it('recovers original inputs when abi encoding and decoding', async () => {
  const { abi, inputs } = await import('../shared/abi_encode');

  const initial_witness: WitnessMap = abiEncode(abi, inputs);
  const decoded_inputs: DecodedInputs = abiDecode(abi, initial_witness);

  const foo: Field = inputs.foo as Field;
  const bar: Field[] = inputs.bar as Field[];
  expect(BigInt(decoded_inputs.inputs.foo)).to.be.equal(BigInt(foo));
  expect(BigInt(decoded_inputs.inputs.bar[0])).to.be.equal(BigInt(bar[0]));
  expect(BigInt(decoded_inputs.inputs.bar[1])).to.be.equal(BigInt(bar[1]));
  expect(decoded_inputs.return_value).to.be.null;
});
