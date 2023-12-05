import { BarretenbergSync } from '@aztec/bb.js';

import { EcdsaSignature } from './signature.js';

export * from './signature.js';

/**
 * ECDSA signature construction and helper operations.
 * TODO: Replace with codegen api on bb.js.
 */
export class Ecdsa {
  private wasm = BarretenbergSync.getSingleton().getWasm();

  /**
   * Computes a secp256k1 public key from a private key.
   * @param privateKey - Secp256k1 private key.
   * @returns A secp256k1 public key.
   */
  public computePublicKey(privateKey: Buffer): Buffer {
    this.wasm.writeMemory(0, privateKey);
    this.wasm.call('ecdsa__compute_public_key', 0, 32);
    return Buffer.from(this.wasm.getMemorySlice(32, 96));
  }

  /**
   * Constructs an ECDSA signature given a msg and a private key.
   * @param msg - Message over which the signature is constructed.
   * @param privateKey - The secp256k1 private key of the signer.
   * @returns An ECDSA signature of the form (r, s, v).
   */
  public constructSignature(msg: Uint8Array, privateKey: Buffer) {
    const mem = this.wasm.call('bbmalloc', msg.length);
    this.wasm.writeMemory(0, privateKey);
    this.wasm.writeMemory(mem, msg);
    this.wasm.call('ecdsa__construct_signature', mem, msg.length, 0, 32, 64, 96);

    return new EcdsaSignature(
      Buffer.from(this.wasm.getMemorySlice(32, 64)),
      Buffer.from(this.wasm.getMemorySlice(64, 96)),
      Buffer.from(this.wasm.getMemorySlice(96, 97)),
    );
  }

  /**
   * Recovers a secp256k1 public key from an ECDSA signature (similar to ecrecover).
   * @param msg - Message over which the signature was constructed.
   * @param sig - The ECDSA signature.
   * @returns The secp256k1 public key of the signer.
   */
  public recoverPublicKey(msg: Uint8Array, sig: EcdsaSignature): Buffer {
    const mem = this.wasm.call('bbmalloc', msg.length);
    this.wasm.writeMemory(0, sig.r);
    this.wasm.writeMemory(32, sig.s);
    this.wasm.writeMemory(64, sig.v);
    this.wasm.writeMemory(mem, msg);
    this.wasm.call('ecdsa__recover_public_key_from_signature', mem, msg.length, 0, 32, 64, 65);

    return Buffer.from(this.wasm.getMemorySlice(65, 129));
  }

  /**
   * Verifies and ECDSA signature given a secp256k1 public key.
   * @param msg - Message over which the signature was constructed.
   * @param pubKey - The secp256k1 public key of the signer.
   * @param sig - The ECDSA signature.
   * @returns True or false.
   */
  public verifySignature(msg: Uint8Array, pubKey: Buffer, sig: EcdsaSignature) {
    const mem = this.wasm.call('bbmalloc', msg.length);
    this.wasm.writeMemory(0, pubKey);
    this.wasm.writeMemory(64, sig.r);
    this.wasm.writeMemory(96, sig.s);
    this.wasm.writeMemory(128, sig.v);
    this.wasm.writeMemory(mem, msg);
    return this.wasm.call('ecdsa__verify_signature', mem, msg.length, 0, 64, 96, 128) ? true : false;
  }
}
