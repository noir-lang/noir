import { expect } from "chai";
import { abiEncode, abiDecode, WitnessMap } from "../../../../result/";
import { DecodedInputs } from "../types";

it("recovers original inputs when abi encoding and decoding", async () => {
  const { abi, inputs } = await import("../shared/abi_encode");

  const initial_witness: WitnessMap = abiEncode(abi, inputs, null);
  const decoded_inputs: DecodedInputs = abiDecode(abi, initial_witness);

  expect(BigInt(decoded_inputs.inputs.foo)).to.be.equal(BigInt(inputs.foo));
  expect(BigInt(decoded_inputs.inputs.bar[0])).to.be.equal(
    BigInt(inputs.bar[0])
  );
  expect(BigInt(decoded_inputs.inputs.bar[1])).to.be.equal(
    BigInt(inputs.bar[1])
  );
  expect(decoded_inputs.return_value).to.be.null;
});