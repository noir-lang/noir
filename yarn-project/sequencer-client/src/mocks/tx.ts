import { KernelCircuitPublicInputs, UInt8Vector } from '@aztec/circuits.js';
import { makeKernelPublicInputs, makeSignedTxRequest } from '@aztec/circuits.js/factories';
import { PrivateTx, PublicTx, Tx, UnverifiedData } from '@aztec/types';

function makeEmptyProof() {
  return new UInt8Vector(Buffer.alloc(0));
}

export function makeEmptyUnverifiedData(): UnverifiedData {
  const chunks = [Buffer.alloc(0)];
  return new UnverifiedData(chunks);
}

export function makeEmptyPrivateTx(): PrivateTx {
  return Tx.createPrivate(KernelCircuitPublicInputs.empty(), makeEmptyProof(), makeEmptyUnverifiedData());
}

export function makePrivateTx(seed = 0): PrivateTx {
  return Tx.createPrivate(makeKernelPublicInputs(seed), makeEmptyProof(), UnverifiedData.random(2));
}

export function makePublicTx(seed = 0): PublicTx {
  return Tx.createPublic(makeSignedTxRequest(seed));
}
