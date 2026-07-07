import { expect } from 'chai';
import { abiEncode, abiDecode, WitnessMap } from '@noir-lang/noirc_abi';
import { MyNestedStruct, MyStruct } from '../shared/structs';
import { DecodedInputs } from '../types';
import { abi, inputs } from '../shared/structs';

it('correctly handles struct inputs', async () => {
  const initial_witness: WitnessMap = abiEncode(abi, inputs);
  const decoded_inputs: DecodedInputs = abiDecode(abi, initial_witness);

  const struct_arg: MyStruct = inputs.struct_arg as MyStruct;
  const struct_array_arg: MyStruct[] = inputs.struct_array_arg as MyStruct[];
  const nested_struct_arg: MyNestedStruct = inputs.nested_struct_arg as MyNestedStruct;

  expect(BigInt(decoded_inputs.inputs.struct_arg.foo)).to.be.equal(BigInt(struct_arg.foo));
  expect(BigInt(decoded_inputs.inputs.struct_array_arg[0].foo)).to.be.equal(BigInt(struct_array_arg[0].foo));
  expect(BigInt(decoded_inputs.inputs.struct_array_arg[1].foo)).to.be.equal(BigInt(struct_array_arg[1].foo));
  expect(BigInt(decoded_inputs.inputs.struct_array_arg[2].foo)).to.be.equal(BigInt(struct_array_arg[2].foo));
  expect(BigInt(decoded_inputs.inputs.nested_struct_arg.foo.foo)).to.be.equal(BigInt(nested_struct_arg.foo.foo));
  expect(decoded_inputs.return_value).to.be.null;
});
