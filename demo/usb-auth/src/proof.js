import { Barretenberg, UltraHonkBackend } from '@aztec/bb.js';
import { Noir } from '@noir-lang/noir_js';
import { computeCommitment, computeNullifier, fieldToString, randomField, userIdToField } from './fields.js';

export async function createAuthInputs({ deviceSecret, userId, usbSerial = 0, challenge = randomField() }) {
  const userIdHash = await userIdToField(userId);
  return {
    privateInputs: {
      device_secret: deviceSecret,
      usb_serial: usbSerial,
      commitment: computeCommitment(deviceSecret, userIdHash),
      challenge,
      user_id_hash: userIdHash,
    },
    publicInputs: {
      usb_serial: usbSerial,
      commitment: computeCommitment(deviceSecret, userIdHash),
      challenge,
      user_id_hash: userIdHash,
      expected_nullifier: computeNullifier(deviceSecret, challenge, userIdHash, usbSerial),
    },
  };
}

export async function generateAndVerifyProof(circuit, authInputs) {
  const barretenberg = await Barretenberg.new();
  try {
    const noir = new Noir(circuit);
    const backend = new UltraHonkBackend(circuit.bytecode, barretenberg);
    const { witness, returnValue } = await noir.execute(authInputs.privateInputs);
    const proof = await backend.generateProof(witness);
    const verified = await backend.verifyProof(proof);
    return {
      proof,
      verified,
      nullifier: fieldToString(returnValue ?? authInputs.publicInputs.expected_nullifier),
      publicInputs: authInputs.publicInputs,
    };
  } finally {
    await barretenberg.destroy();
  }
}

export function proofToJson(result) {
  return {
    verified: result.verified,
    nullifier: result.nullifier,
    publicInputs: result.publicInputs,
    proof: Array.from(result.proof.proof),
    proofPublicInputs: result.proof.publicInputs?.map(String) ?? [],
  };
}
