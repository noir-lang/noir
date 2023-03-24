import { expectSerializeToMatchSnapshot } from "../tests/expectSerialize.js";
import { fr } from "../tests/factories.js";
import { range } from "../utils/jsUtils.js";
import { numToUInt32BE } from "../utils/serialize.js";
import { CallContext } from "./call_context.js";
import {
  ARGS_LENGTH,
  EMITTED_EVENTS_LENGTH,
  RETURN_VALUES_LENGTH,
  NEW_COMMITMENTS_LENGTH,
  NEW_NULLIFIERS_LENGTH,
  PRIVATE_CALL_STACK_LENGTH,
  PUBLIC_CALL_STACK_LENGTH,
  L1_MSG_STACK_LENGTH,
} from "./constants.js";
import { PrivateCircuitPublicInputs } from "./private_circuit_public_inputs.js";
import { EthAddress, Fr } from "./shared.js";
import { ContractDeploymentData } from "./tx.js";

/**
 * Create sequential test data for ContractDeploymentData.
 * @returns Test data.
 */
function contractDeploymentData() {
  return new ContractDeploymentData(
    new Fr(numToUInt32BE(1, 32)),
    new Fr(numToUInt32BE(2, 32)),
    new Fr(numToUInt32BE(3, 32)),
    new EthAddress(numToUInt32BE(4, 20))
  );
}

/**
 * Create sequential test data for PrivateCircuitPublicInputs.
 * @returns Test data.
 */
function privateCircuitPublicInputs() {
  return PrivateCircuitPublicInputs.from({
    callContext: new CallContext(
      fr(1),
      fr(2),
      new EthAddress(numToUInt32BE(3, /* eth address is 20 bytes */ 20)),
      true,
      true,
      true
    ),
    args: range(ARGS_LENGTH, 0x100).map(fr),
    emittedEvents: range(EMITTED_EVENTS_LENGTH, 0x200).map(fr), // TODO not in spec
    returnValues: range(RETURN_VALUES_LENGTH, 0x300).map(fr),
    newCommitments: range(NEW_COMMITMENTS_LENGTH, 0x400).map(fr),
    newNullifiers: range(NEW_NULLIFIERS_LENGTH, 0x500).map(fr),
    privateCallStack: range(PRIVATE_CALL_STACK_LENGTH, 0x600).map(fr),
    publicCallStack: range(PUBLIC_CALL_STACK_LENGTH, 0x700).map(fr),
    l1MsgStack: range(L1_MSG_STACK_LENGTH, 0x800).map(fr),
    historicContractTreeRoot: new Fr(numToUInt32BE(0x900, 32)), // TODO not in spec
    historicPrivateDataTreeRoot: new Fr(numToUInt32BE(0x1000, 32)),
    historicPrivateNullifierTreeRoot: new Fr(numToUInt32BE(0x1100, 32)), // TODO not in spec
    contractDeploymentData: contractDeploymentData(),
  });
}

describe("basic PrivateCircuitPublicInputs serialization", () => {
  it(`serializes a trivial PrivateCircuitPublicInputs and prints it`, async () => {
    // Test the data case: writing (mostly) sequential numbers
    await expectSerializeToMatchSnapshot(
      privateCircuitPublicInputs().toBuffer(),
      "abis__test_roundtrip_serialize_private_circuit_public_inputs"
    );
  });
});
