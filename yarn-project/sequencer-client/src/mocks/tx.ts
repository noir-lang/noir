import { KernelCircuitPublicInputs, UInt8Vector } from '@aztec/circuits.js';
import { makeKernelPublicInputs, makeSignedTxRequest } from '@aztec/circuits.js/factories';
import { PrivateTx, PublicTx, Tx, UnverifiedData } from '@aztec/types';

/**
 * Creates an empty proof.
 */
function makeEmptyProof() {
  return new UInt8Vector(Buffer.alloc(0));
}

/**
 * Testing utility to create empty unverified data composed by a single empty chunk.
 */
export function makeEmptyUnverifiedData(): UnverifiedData {
  const chunks = [Buffer.alloc(0)];
  return new UnverifiedData(chunks);
}

/**
 * Testing utility to create a tx with an empty kernel circuit output, empty proof, and empty unverified data.
 */
export function makeEmptyPrivateTx(): PrivateTx {
  return Tx.createPrivate(KernelCircuitPublicInputs.empty(), makeEmptyProof(), makeEmptyUnverifiedData());
}

/**
 * Testing utility to create a tx with gibberish kernel circuit output, random unverified data, and an empty proof.
 */
export function makePrivateTx(seed = 0): PrivateTx {
  return Tx.createPrivate(makeKernelPublicInputs(seed), makeEmptyProof(), UnverifiedData.random(2));
}

/**
 * Testing utility to create a tx with a request to execute a public function.
 */
export function makePublicTx(seed = 0): PublicTx {
  return Tx.createPublic(makeSignedTxRequest(seed));
}
