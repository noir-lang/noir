import { secp256k1 } from '@noble/curves/secp256k1';
import { AztecAddress } from '../index.js';
import { AuthPayload, TxAuthProvider } from './index.js';

import { EntrypointPayload } from '../account_impl/account_contract.js';
import { EcdsaSignature } from '@aztec/circuits.js/barretenberg';

/**
 * An ecdsa implementation of TxAuthProvider.
 */
export class EcdsaAuthProvider implements TxAuthProvider {
  constructor(private privKey: Buffer) {}
  authenticateTx(payload: EntrypointPayload, payloadHash: Buffer, _address: AztecAddress): Promise<AuthPayload> {
    const sig = secp256k1.sign(payloadHash, this.privKey);
    if (sig.recovery === undefined) throw new Error(`Missing recovery from signature`);
    return Promise.resolve(EcdsaSignature.fromBigInts(sig.r, sig.s, sig.recovery));
  }
}
