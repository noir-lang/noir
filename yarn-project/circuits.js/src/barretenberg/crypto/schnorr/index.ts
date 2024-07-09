import { BarretenbergSync } from '@aztec/bb.js';
import { type GrumpkinScalar, Point } from '@aztec/foundation/fields';
import { numToUInt32BE } from '@aztec/foundation/serialize';

import { type PublicKey } from '../../../types/public_key.js';
import { SchnorrSignature } from './signature.js';

export * from './signature.js';

/**
 * Schnorr signature construction and helper operations.
 */
export class Schnorr {
  private wasm = BarretenbergSync.getSingleton().getWasm();

  /**
   * Computes a grumpkin public key from a private key.
   * @param privateKey - The private key.
   * @returns A grumpkin public key.
   */
  public computePublicKey(privateKey: GrumpkinScalar): PublicKey {
    this.wasm.writeMemory(0, privateKey.toBuffer());
    this.wasm.call('schnorr_compute_public_key', 0, 32);
    return Point.fromBuffer(Buffer.from(this.wasm.getMemorySlice(32, 96)));
  }

  /**
   * Constructs a Schnorr signature given a msg and a private key.
   * @param msg - Message over which the signature is constructed.
   * @param privateKey - The private key of the signer.
   * @returns A Schnorr signature of the form (s, e).
   */
  public constructSignature(msg: Uint8Array, privateKey: GrumpkinScalar) {
    const mem = this.wasm.call('bbmalloc', msg.length + 4);
    this.wasm.writeMemory(0, privateKey.toBuffer());
    this.wasm.writeMemory(mem, Buffer.concat([numToUInt32BE(msg.length), msg]));
    this.wasm.call('schnorr_construct_signature', mem, 0, 32, 64);

    return new SchnorrSignature(Buffer.from(this.wasm.getMemorySlice(32, 96)));
  }

  /**
   * Verifies a Schnorr signature given a Grumpkin public key.
   * @param msg - Message over which the signature was constructed.
   * @param pubKey - The Grumpkin public key of the signer.
   * @param sig - The Schnorr signature.
   * @returns True or false.
   */
  public verifySignature(msg: Uint8Array, pubKey: PublicKey, sig: SchnorrSignature) {
    const mem = this.wasm.call('bbmalloc', msg.length + 4);
    this.wasm.writeMemory(0, pubKey.toBuffer());
    this.wasm.writeMemory(64, sig.s);
    this.wasm.writeMemory(96, sig.e);
    this.wasm.writeMemory(mem, Buffer.concat([numToUInt32BE(msg.length), msg]));
    this.wasm.call('schnorr_verify_signature', mem, 0, 64, 96, 128);
    const result = this.wasm.getMemorySlice(128, 129);
    return !Buffer.alloc(1, 0).equals(result);
  }
}
