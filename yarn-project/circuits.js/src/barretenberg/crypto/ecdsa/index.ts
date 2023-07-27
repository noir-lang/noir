import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { numToUInt32BE } from '@aztec/foundation/serialize';
import { IWasmModule } from '@aztec/foundation/wasm';

import { secp256k1 } from '@noble/curves/secp256k1';

import { CircuitsWasm, Point, PrivateKey, PublicKey } from '../../../index.js';
import { Signer } from '../index.js';
import { EcdsaSignature } from './signature.js';

export * from './signature.js';

/**
 * ECDSA signature construction and helper operations.
 */
export class Ecdsa implements Signer {
  /**
   * Creates a new Ecdsa instance.
   * @returns New Ecdsa instance.
   */
  public static async new() {
    return new this(await CircuitsWasm.get());
  }

  constructor(private wasm: IWasmModule) {}

  /**
   * Computes a secp256k1 public key from a private key.
   * @param privateKey - Secp256k1 private key.
   * @returns A secp256k1 public key.
   */
  public computePublicKey(privateKey: PrivateKey): PublicKey {
    this.wasm.writeMemory(0, privateKey.value);
    this.wasm.call('ecdsa__compute_public_key', 0, 32);
    return Point.fromBuffer(Buffer.from(this.wasm.getMemorySlice(32, 96)));
  }

  /**
   * Constructs an ECDSA signature given a msg and a private key.
   * @param msg - Message over which the signature is constructed.
   * @param privateKey - The secp256k1 private key of the signer.
   * @returns An ECDSA signature of the form (r, s, v).
   */
  public constructSignature(msg: Uint8Array, privateKey: PrivateKey) {
    const mem = this.wasm.call('bbmalloc', msg.length);
    this.wasm.writeMemory(0, privateKey.value);
    this.wasm.writeMemory(mem, msg);
    this.wasm.call('ecdsa__construct_signature', mem, msg.length, 0, 32, 64, 96);

    // TODO(#913): Understand why this doesn't work
    // const sig = new EcdsaSignature(
    //   Buffer.from(this.wasm.getMemorySlice(32, 64)),
    //   Buffer.from(this.wasm.getMemorySlice(64, 96)),
    //   Buffer.from(this.wasm.getMemorySlice(96, 97)),
    // );

    const signature = secp256k1.sign(msg, privateKey.value);
    return new EcdsaSignature(
      toBufferBE(signature.r, 32),
      toBufferBE(signature.s, 32),
      numToUInt32BE(signature.recovery!).subarray(3, 4),
    );
  }

  /**
   * Recovers a secp256k1 public key from an ECDSA signature (similar to ecrecover).
   * @param msg - Message over which the signature was constructed.
   * @param sig - The ECDSA signature.
   * @returns The secp256k1 public key of the signer.
   */
  public recoverPublicKey(msg: Uint8Array, sig: EcdsaSignature): PublicKey {
    const mem = this.wasm.call('bbmalloc', msg.length);
    this.wasm.writeMemory(0, sig.r);
    this.wasm.writeMemory(32, sig.s);
    this.wasm.writeMemory(64, sig.v);
    this.wasm.writeMemory(mem, msg);
    this.wasm.call('ecdsa__recover_public_key_from_signature', mem, msg.length, 0, 32, 64, 65);

    return Point.fromBuffer(Buffer.from(this.wasm.getMemorySlice(65, 129)));
  }

  /**
   * Verifies and ECDSA signature given a secp256k1 public key.
   * @param msg - Message over which the signature was constructed.
   * @param pubKey - The secp256k1 public key of the signer.
   * @param sig - The ECDSA signature.
   * @returns True or false.
   */
  public verifySignature(msg: Uint8Array, pubKey: PublicKey, sig: EcdsaSignature) {
    const mem = this.wasm.call('bbmalloc', msg.length);
    this.wasm.writeMemory(0, pubKey.toBuffer());
    this.wasm.writeMemory(64, sig.r);
    this.wasm.writeMemory(96, sig.s);
    this.wasm.writeMemory(128, sig.v);
    this.wasm.writeMemory(mem, msg);
    return this.wasm.call('ecdsa__verify_signature', mem, msg.length, 0, 64, 96, 128) ? true : false;
  }
}
