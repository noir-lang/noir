import { EthAddress } from '../eth_address/index.js';
import { numToUInt8 } from '../serialize/index.js';
import { keccak256 } from '../crypto/index.js';
import elliptic from 'elliptic';
import { hexToBuffer } from '../hex_string/index.js';

const secp256k1 = new elliptic.ec('secp256k1');

export class EthSignature {
  constructor(public r: Buffer, public s: Buffer, public v: number) {}

  static fromBuffer(buf: Buffer) {
    return new EthSignature(buf.subarray(0, 32), buf.subarray(32, 64), buf[64]);
  }

  static fromString(hex: string) {
    return EthSignature.fromBuffer(hexToBuffer(hex));
  }

  toBuffer() {
    return Buffer.concat([this.r, this.s, numToUInt8(this.v)]);
  }

  toString() {
    return '0x' + this.toBuffer().toString('hex');
  }
}

export function signMessage(messageHash: Buffer, privateKey: Buffer) {
  return sign(messageHash, privateKey, 27);
}

export function sign(messageHash: Buffer, privateKey: Buffer, addToV = 0): EthSignature {
  const signature = secp256k1.keyFromPrivate(privateKey).sign(messageHash, { canonical: true });
  const v = signature.recoveryParam! + addToV;
  const r = signature.r.toBuffer('be', 32);
  const s = signature.s.toBuffer('be', 32);
  return new EthSignature(r, s, v);
}

export function recoverFromSignature(messageHash: Buffer, { v, r, s }: EthSignature) {
  return recoverFromVRS(messageHash, v, r, s);
}

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

export function recoverFromSigBuffer(messageHash: Buffer, signature: Buffer) {
  return recoverFromSignature(messageHash, EthSignature.fromBuffer(signature));
}
