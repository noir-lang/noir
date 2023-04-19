import { EthAddress } from '@aztec/foundation';
import { numToUInt8 } from '../serialize/index.js';
import { keccak256 } from '../crypto/index.js';
import elliptic from 'elliptic';
import { hexToBuffer } from '../hex_string/index.js';

const secp256k1 = new elliptic.ec('secp256k1');

/**
 * The EthSignature class represents an Ethereum signature consisting of 'r', 's', and 'v' components.
 * It provides methods to convert signatures between string, buffer, and object formats for easy manipulation
 * and usage in signing and recovering operations. This class is particularly useful when working with
 * Ethereum transactions that require signing and validation processes.
 */
export class EthSignature {
  constructor(
    /**
     * The 'r' value of an ECDSA signature.
     */
    public r: Buffer,
    /**
     * The 's' value of the ECDSA signature.
     */ public s: Buffer,
    /**
     * The recovery parameter used in ECDSA signatures.
     */ public v: number,
  ) {}

  /**
   * Create an EthSignature instance from a given buffer.
   * The input 'buf' should be a Buffer containing the 'r', 's', and 'v' values of the signature.
   * 'r' and 's' values are each 32 bytes long, while 'v' is a single byte.
   * Throws an error if the input buffer length is not exactly 65 bytes.
   *
   * @param buf - The Buffer containing the 'r', 's', and 'v' values of the signature.
   * @returns An EthSignature instance.
   */
  static fromBuffer(buf: Buffer) {
    return new EthSignature(buf.subarray(0, 32), buf.subarray(32, 64), buf[64]);
  }

  /**
   * Create an EthSignature instance from a hex-encoded string.
   * The input 'hex' should be prefixed with '0x', followed by 128 hex characters (for r, s) and 2 hex characters for the `v` value.
   * Throws an error if the input length is invalid or any of the r, s, v values are out of range.
   *
   * @param hex - The hex-encoded string representing the Ethereum signature.
   * @returns An EthSignature instance.
   */
  static fromString(hex: string) {
    return EthSignature.fromBuffer(hexToBuffer(hex));
  }

  /**
   * Converts the EthSignature instance to a Buffer representation.
   * The resulting buffer contains the concatenated 'r', 's', and 'v' values of the signature.
   * This function is useful when working with raw binary data or when storing the signature.
   *
   * @returns A Buffer containing the concatenated 'r', 's', and 'v' values of the EthSignature instance.
   */
  toBuffer() {
    return Buffer.concat([this.r, this.s, numToUInt8(this.v)]);
  }

  /**
   * Convert the EthSignature instance into a hex-encoded string.
   * The resulting string is prefixed with '0x' and represents the concatenated r, s, and v values of the signature.
   *
   * @returns A hex-encoded string representing the EthSignature instance.
   */
  toString() {
    return '0x' + this.toBuffer().toString('hex');
  }
}

/**
 * Sign a message hash using the provided private key and add 27 to the recovery value.
 * This function produces an Ethereum-compatible signature, which can be used for verifying
 * the signer's address. It returns an EthSignature object containing the 'r', 's', and 'v' values.
 *
 * @param messageHash - The Buffer containing the hashed message to be signed.
 * @param privateKey - The Buffer containing the private key used for signing the message hash.
 * @returns An EthSignature instance with the signature components (r, s, v).
 */
export function signMessage(messageHash: Buffer, privateKey: Buffer) {
  return sign(messageHash, privateKey, 27);
}

/**
 * Generate an Ethereum signature for a given message hash and private key.
 * The 'sign' function takes a message hash (Buffer), a private key (Buffer), and an optional addToV parameter,
 * and returns an EthSignature object containing the r, s, and v components of the signature.
 * The 'addToV' parameter can be used to adjust the recovery ID of the signature (default is 0).
 *
 * @param messageHash - The message hash to be signed, as a Buffer.
 * @param privateKey - The signer's private key, as a Buffer.
 * @param addToV - Optional value to add to the recovery ID of the signature (default is 0).
 * @returns An instance of EthSignature containing the r, s, and v components of the signed message.
 */
export function sign(messageHash: Buffer, privateKey: Buffer, addToV = 0): EthSignature {
  const signature = secp256k1.keyFromPrivate(privateKey).sign(messageHash, { canonical: true });
  const v = signature.recoveryParam! + addToV;
  const r = signature.r.toBuffer('be', 32);
  const s = signature.s.toBuffer('be', 32);
  return new EthSignature(r, s, v);
}

/**
 * Recover the Ethereum address from a signature and message hash.
 * This function takes the message hash and an EthSignature object, which contains r, s, and v values,
 * and returns the corresponding Ethereum address that signed the message.
 * The recovered address is returned as an EthAddress instance.
 *
 * @param messageHash - The hash of the message that was signed.
 * @param signature - An EthSignature object containing r, s, and v values.
 * @returns An EthAddress instance representing the address that signed the message.
 */
export function recoverFromSignature(messageHash: Buffer, { v, r, s }: EthSignature) {
  return recoverFromVRS(messageHash, v, r, s);
}

/**
 * Recover the Ethereum address from a message hash, using the provided signature parameters (v, r, s).
 * The function uses elliptic curve cryptography (secp256k1) to recover the public key and then derives
 * the Ethereum address by hashing the public key with keccak256.
 *
 * @param messageHash - The hashed message that was signed.
 * @param v - The recovery identifier value, used to determine which of the two possible keys was used for signing.
 * @param r - The 'r' component of the ECDSA signature.
 * @param s - The 's' component of the ECDSA signature.
 * @returns An EthAddress instance representing the recovered Ethereum address.
 */
export function recoverFromVRS(messageHash: Buffer, v: number, r: Buffer, s: Buffer) {
  const ecPublicKey = secp256k1.recoverPubKey(
    messageHash,
    {
      r,
      s,
    },
    v < 2 ? v : 1 - (v % 2),
  );
  const publicKey = Buffer.from(ecPublicKey.encode('hex', false).slice(2), 'hex');
  const publicHash = keccak256(publicKey);
  return new EthAddress(publicHash.subarray(-20));
}

/**
 * Recover an Ethereum address from a given message hash and signature buffer.
 * This function uses the EthSignature.fromBuffer() method to convert the signature buffer into an
 * EthSignature instance, then calls the recoverFromSignature() function with the message hash and
 * the EthSignature instance to recover the Ethereum address.
 *
 * @param messageHash - The Buffer containing the hash of the message that was signed.
 * @param signature - The Buffer containing the signature generated from signing the message.
 * @returns An EthAddress instance representing the recovered Ethereum address.
 */
export function recoverFromSigBuffer(messageHash: Buffer, signature: Buffer) {
  return recoverFromSignature(messageHash, EthSignature.fromBuffer(signature));
}
