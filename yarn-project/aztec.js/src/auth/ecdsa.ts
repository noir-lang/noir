import { secp256k1 } from '@noble/curves/secp256k1';
import { EcdsaSignature } from '@aztec/circuits.js/barretenberg';

import { AztecAddress } from '../index.js';
import { AuthPayload } from './index.js';
import { EntrypointPayload } from '../account_impl/account_contract.js';

/**
 * An ecdsa implementation of auth provider.
 */
export class EcdsaAuthProvider {
  constructor(private privKey: Buffer) {}
  authenticateTx(payload: EntrypointPayload, payloadHash: Buffer, _address: AztecAddress): Promise<AuthPayload> {
    const sig = secp256k1.sign(payloadHash, this.privKey);
    if (sig.recovery === undefined) throw new Error(`Missing recovery from signature`);
    return Promise.resolve(EcdsaSignature.fromBigInts(sig.r, sig.s, sig.recovery));
  }
}
