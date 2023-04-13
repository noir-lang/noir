import { PrivateKernelPublicInputs, UInt8Vector } from '@aztec/circuits.js';
import { makePrivateKernelPublicInputs } from '@aztec/circuits.js/factories';
import { Tx, UnverifiedData } from '@aztec/types';

function makeEmptyProof() {
  return new UInt8Vector(Buffer.alloc(0));
}

export function makeEmptyUnverifiedData(): UnverifiedData {
  const chunks = [Buffer.alloc(0)];
  return new UnverifiedData(chunks);
}

export function makeEmptyTx(): Tx {
  const isEmpty = true;
  return new Tx(PrivateKernelPublicInputs.makeEmpty(), makeEmptyProof(), makeEmptyUnverifiedData(), isEmpty);
}

export function makeTx(seed = 0): Tx {
  return new Tx(makePrivateKernelPublicInputs(seed), makeEmptyProof(), UnverifiedData.random(2), false);
}
