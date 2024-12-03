import { expect } from '@esm-bundle/chai';
import initNoirAbi, { abiEncode, abiDecode, WitnessMap, Field } from '@noir-lang/noirc_abi';
import { DecodedInputs } from '../types';

beforeEach(async () => {
  await initNoirAbi();
});

it('recovers original inputs when abi encoding and decoding', async () => {
  const { abi, inputs } = await import('../shared/abi_encode');

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
